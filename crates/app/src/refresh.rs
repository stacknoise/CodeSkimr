//! Renewing OAuth access tokens (blueprint 12.2): a GitLab token lives for two hours.
//!
//! A sign-in with the device flow keeps the refresh token in the keystore next to the access
//! token, under the name of the credential plus [`REFRESH_SUFFIX`]. The user field of that entry
//! says where to renew (platform, application, host); it holds no secret. When a clone is
//! refused with a rejected credential, the source is fetched again once with a fresh token.

use std::sync::Arc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::SourceProvider;
use codeskimr_application::ports::{AuthError, Credential, CredentialStore, FetchedSource};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{CredentialId, ProjectSource};
use codeskimr_secrets::{Provider, SignedIn};
use secrecy::{ExposeSecret, SecretString};

use crate::consts::{REFRESH_LINK_SEPARATOR, REFRESH_SUFFIX, is_token_credential};

/// Where a credential is renewed: the platform, the application and, for a self-managed
/// installation, the host.
pub struct Link<'a> {
    /// The platform.
    pub provider: Provider,
    /// The application id of the device flow.
    pub client_id: &'a str,
    /// The host of a self-managed installation.
    pub host: Option<&'a str>,
}

/// The name of the refresh entry of `id`.
///
/// # Errors
/// `SEC-401` if the name is not valid, which cannot happen for a valid `id`.
pub fn refresh_id(id: &CredentialId) -> Result<CredentialId, CodedError> {
    CredentialId::new(format!("{id}{REFRESH_SUFFIX}"))
        .map_err(|_| CodedError::from(AuthError::Damaged))
}

/// Stores a signed-in credential and its refresh token. Without a refresh token an older one is
/// removed: it would renew to a token of another sign-in.
///
/// # Errors
/// `SEC-401`, `SEC-402` or `SEC-601` from the keystore.
pub fn save(
    store: &impl CredentialStore,
    id: &CredentialId,
    signed: SignedIn,
    link: &Link<'_>,
) -> Result<(), CodedError> {
    let entry = refresh_id(id)?;
    match signed.refresh {
        Some(token) => {
            let user = [
                link.provider.name(),
                link.client_id,
                link.host.unwrap_or_default(),
            ]
            .join(&REFRESH_LINK_SEPARATOR.to_string());
            store.store(&entry, &Credential::new(SecretString::from(user), token))?;
        }
        None => store.delete(&entry)?,
    }
    store.store(id, &signed.credential)?;
    Ok(())
}

/// Removes the refresh entry of a credential that was replaced by hand: it would renew to a
/// token of the old sign-in and overwrite the new one.
///
/// # Errors
/// `SEC-402` or `SEC-601` from the keystore.
pub fn forget(store: &impl CredentialStore, id: &CredentialId) -> Result<(), CodedError> {
    store.delete(&refresh_id(id)?)?;
    Ok(())
}

/// Removes a credential together with its refresh entry.
///
/// # Errors
/// `SEC-402` or `SEC-601` from the keystore.
pub fn remove(store: &impl CredentialStore, id: &CredentialId) -> Result<(), CodedError> {
    store.delete(&refresh_id(id)?)?;
    store.delete(id)?;
    Ok(())
}

/// What a refresh entry holds.
struct Entry {
    provider: Provider,
    client_id: String,
    host: Option<String>,
    token: SecretString,
}

fn read_entry(
    store: &impl CredentialStore,
    id: &CredentialId,
) -> Result<Option<Entry>, CodedError> {
    let found = store.with_credential(&refresh_id(id)?, |credential| {
        let user = credential.user().expose_secret().to_owned();
        let token = SecretString::from(credential.secret().expose_secret().to_owned());
        Ok((user, token))
    });
    let (user, token) = match found {
        Ok(entry) => entry,
        Err(AuthError::NotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut parts = user.splitn(3, REFRESH_LINK_SEPARATOR);
    let provider = parts.next().and_then(Provider::from_name);
    let client_id = parts.next().unwrap_or_default().to_owned();
    let host = parts
        .next()
        .filter(|host| !host.is_empty())
        .map(str::to_owned);
    let Some(provider) = provider else {
        return Err(AuthError::Damaged.into());
    };
    Ok(Some(Entry {
        provider,
        client_id,
        host,
        token,
    }))
}

/// Renews the access token of a credential with its refresh token.
pub struct Refresher<S> {
    store: Arc<S>,
}

impl<S: CredentialStore> Refresher<S> {
    /// A refresher that reads and writes `store`.
    #[must_use]
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    /// Trades the refresh token of `id` for a new access token and stores both. Returns whether
    /// there was anything to renew: a credential without refresh entry, such as a pasted token,
    /// stays as it is.
    ///
    /// # Errors
    /// `NET-601` if the platform refuses the refresh token, `SEC-*` from the keystore.
    pub fn refresh(&self, id: &CredentialId) -> Result<bool, CodedError> {
        if is_token_credential(id.as_str()) {
            return Ok(false);
        }
        let Some(entry) = read_entry(&*self.store, id)? else {
            return Ok(false);
        };
        let flow = entry
            .provider
            .flow(&entry.client_id, entry.host.as_deref())?;
        let mut signed = flow.refresh(&entry.token)?;
        // A platform that does not rotate the token sends none: the old one stays valid.
        signed.refresh.get_or_insert(entry.token);
        let link = Link {
            provider: entry.provider,
            client_id: &entry.client_id,
            host: entry.host.as_deref(),
        };
        save(&*self.store, id, signed, &link)?;
        Ok(true)
    }
}

/// A source provider that renews a rejected token once and tries again.
pub struct RefreshingSource<P, S> {
    inner: P,
    refresher: Refresher<S>,
}

impl<P, S> RefreshingSource<P, S> {
    /// Wraps `inner`; tokens are renewed through `refresher`.
    #[must_use]
    pub fn new(inner: P, refresher: Refresher<S>) -> Self {
        Self { inner, refresher }
    }
}

impl<P: SourceProvider, S: CredentialStore> SourceProvider for RefreshingSource<P, S> {
    fn fetch(
        &self,
        source: &ProjectSource,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, CodedError> {
        let first = self.inner.fetch(source, cancel);
        let Err(error) = &first else {
            return first;
        };
        let ProjectSource::Git {
            credential: Some(id),
            ..
        } = source
        else {
            return first;
        };
        if error.code() == ErrorCode::CredentialRejected
            && matches!(self.refresher.refresh(id), Ok(true))
        {
            return self.inner.fetch(source, cancel);
        }
        first
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Mutex;

    use codeskimr_domain::{BranchName, FolderPath, GitUrl};

    use super::*;
    use crate::memory_keystore::secrets;
    use crate::shared_keystore::Secrets;

    const OLD_ACCESS: &str = "CANARY-access-old-31ac";
    const OLD_REFRESH: &str = "CANARY-refresh-old-9e07";
    const NEW_ACCESS: &str = "CANARY-access-new-6b12";
    const NEW_REFRESH: &str = "CANARY-refresh-new-c4d8";

    /// Answers every request with `answer` and keeps the request bodies.
    fn oauth_server(answer: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let keep = Arc::clone(&seen);
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buffer = [0_u8; 8192];
                let count = stream.read(&mut buffer).unwrap_or(0);
                let text = String::from_utf8_lossy(&buffer[..count]).into_owned();
                keep.lock().unwrap().push(text);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
                    answer.len()
                );
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        (host, seen)
    }

    /// A fetcher that is rejected as often as the script says, then succeeds.
    struct Scripted {
        rejections: Mutex<Vec<ErrorCode>>,
        calls: Mutex<u32>,
    }

    impl Scripted {
        fn new(rejections: Vec<ErrorCode>) -> Self {
            Self {
                rejections: Mutex::new(rejections),
                calls: Mutex::new(0),
            }
        }
    }

    impl SourceProvider for Scripted {
        fn fetch(&self, _: &ProjectSource, _: &CancelToken) -> Result<FetchedSource, CodedError> {
            *self.calls.lock().unwrap() += 1;
            match self.rejections.lock().unwrap().pop() {
                Some(code) => Err(CodedError::new(code)),
                None => Ok(FetchedSource {
                    root: FolderPath::new("/tmp").unwrap(),
                    commit: None,
                }),
            }
        }
    }

    fn source(id: &CredentialId) -> ProjectSource {
        ProjectSource::Git {
            url: GitUrl::new("https://example.com/org/repo.git").unwrap(),
            branch: BranchName::new("main").unwrap(),
            credential: Some(id.clone()),
        }
    }

    fn signed_in(store: &Secrets, id: &CredentialId, host: &str) {
        let credential =
            Credential::new(SecretString::from("oauth2"), SecretString::from(OLD_ACCESS));
        let signed = SignedIn {
            credential,
            refresh: Some(SecretString::from(OLD_REFRESH)),
        };
        let link = Link {
            provider: Provider::GitLab,
            client_id: "app 1",
            host: Some(host),
        };
        save(store, id, signed, &link).unwrap();
    }

    fn read(store: &Secrets, id: &str) -> Result<String, AuthError> {
        let id = CredentialId::new(id).unwrap();
        store.with_credential(&id, |credential| {
            Ok(credential.secret().expose_secret().to_owned())
        })
    }

    fn setup(
        host: &str,
        rejections: Vec<ErrorCode>,
    ) -> (
        Arc<Secrets>,
        CredentialId,
        RefreshingSource<Scripted, Secrets>,
    ) {
        let store = secrets();
        let id = CredentialId::new("work").unwrap();
        signed_in(&store, &id, host);
        let fetcher = RefreshingSource::new(
            Scripted::new(rejections),
            Refresher::new(Arc::clone(&store)),
        );
        (store, id, fetcher)
    }

    #[test]
    fn a_rejected_token_is_renewed_once_and_the_fetch_goes_on() {
        let answer = r#"{"access_token":"CANARY-access-new-6b12","refresh_token":"CANARY-refresh-new-c4d8"}"#;
        let (host, requests) = oauth_server(answer);
        let (store, id, fetcher) = setup(&host, vec![ErrorCode::CredentialRejected]);
        let cancel = CancelToken::default();
        assert!(fetcher.fetch(&source(&id), &cancel).is_ok());
        assert_eq!(fetcher.inner.calls.lock().unwrap().to_owned(), 2);
        assert!(
            read(&store, "work").unwrap() == NEW_ACCESS,
            "token mismatch"
        );
        assert!(
            read(&store, "work.refresh").unwrap() == NEW_REFRESH,
            "token mismatch"
        );
        let body = requests.lock().unwrap().join("");
        assert!(body.contains("grant_type=refresh_token"));
        assert!(body.contains(OLD_REFRESH));
        assert!(body.contains("client_id=app%201"));
    }

    #[test]
    fn a_platform_that_does_not_rotate_keeps_the_old_refresh_token() {
        let (host, _) = oauth_server(r#"{"access_token":"CANARY-access-new-6b12"}"#);
        let (store, id, fetcher) = setup(&host, vec![ErrorCode::CredentialRejected]);
        assert!(fetcher.fetch(&source(&id), &CancelToken::default()).is_ok());
        assert!(
            read(&store, "work").unwrap() == NEW_ACCESS,
            "token mismatch"
        );
        assert!(
            read(&store, "work.refresh").unwrap() == OLD_REFRESH,
            "token mismatch"
        );
    }

    #[test]
    fn a_refused_refresh_leaves_the_tokens_and_reports_the_rejection() {
        let (host, _) = oauth_server(r#"{"error":"invalid_grant"}"#);
        let (store, id, fetcher) = setup(&host, vec![ErrorCode::CredentialRejected]);
        let error = fetcher
            .fetch(&source(&id), &CancelToken::default())
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::CredentialRejected);
        assert_eq!(fetcher.inner.calls.lock().unwrap().to_owned(), 1);
        assert!(
            read(&store, "work").unwrap() == OLD_ACCESS,
            "token mismatch"
        );
        assert!(
            read(&store, "work.refresh").unwrap() == OLD_REFRESH,
            "token mismatch"
        );
    }

    #[test]
    fn a_second_rejection_is_not_renewed_again() {
        let answer = r#"{"access_token":"CANARY-access-new-6b12"}"#;
        let (host, requests) = oauth_server(answer);
        let rejections = vec![ErrorCode::CredentialRejected; 2];
        let (_, id, fetcher) = setup(&host, rejections);
        let result = fetcher.fetch(&source(&id), &CancelToken::default());
        assert_eq!(result.unwrap_err().code(), ErrorCode::CredentialRejected);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn other_failures_and_credentials_without_refresh_entry_are_not_renewed() {
        let (host, requests) = oauth_server(r#"{"access_token":"x"}"#);
        let (store, id, fetcher) = setup(&host, vec![ErrorCode::GitAuthRequired]);
        assert!(
            fetcher
                .fetch(&source(&id), &CancelToken::default())
                .is_err()
        );
        // A pasted token has no refresh entry: the rejection stays.
        forget(&*store, &id).unwrap();
        let rejected = Scripted::new(vec![ErrorCode::CredentialRejected]);
        let fetcher = RefreshingSource::new(rejected, Refresher::new(Arc::clone(&store)));
        let error = fetcher.fetch(&source(&id), &CancelToken::default());
        assert_eq!(error.unwrap_err().code(), ErrorCode::CredentialRejected);
        assert!(requests.lock().unwrap().is_empty());
    }

    #[test]
    fn removing_a_credential_removes_its_refresh_entry_and_a_new_secret_forgets_it() {
        let store = secrets();
        let id = CredentialId::new("work").unwrap();
        signed_in(&store, &id, "http://127.0.0.1:9");
        forget(&*store, &id).unwrap();
        assert!(matches!(
            read(&store, "work.refresh"),
            Err(AuthError::NotFound)
        ));
        assert!(read(&store, "work").is_ok());
        signed_in(&store, &id, "http://127.0.0.1:9");
        remove(&*store, &id).unwrap();
        assert!(matches!(read(&store, "work"), Err(AuthError::NotFound)));
        assert!(matches!(
            read(&store, "work.refresh"),
            Err(AuthError::NotFound)
        ));
    }
}
