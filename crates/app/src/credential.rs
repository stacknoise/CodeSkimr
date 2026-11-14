//! `codeskimr credential`: stores, replaces and removes the credentials of Git sources, and signs
//! in with the OAuth device flow (blueprint 12.2, issue #53).
//!
//! The secret never travels as an argument or an environment variable: it is read without echo
//! from the terminal, or from standard input if there is no terminal, and handed straight to the
//! credential store. It is not printed, not logged and not part of any error.

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Credential, CredentialStore};
use codeskimr_cli::{CredentialAction, ProviderChoice};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::CredentialId;
use codeskimr_secrets::{OsCredentialStore, OsKeystore, Prompt, Provider};
use secrecy::SecretString;
use zeroize::Zeroizing;

use crate::consts::{
    CARRIAGE_RETURN, LINE_BREAK, LINE_FEED, MAX_SECRET_BYTES, is_reserved_credential,
};
use crate::refresh::{self, Link};

/// What a credential command did, for the line that reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// The credential was stored or replaced.
    Stored(String),
    /// The credential was removed.
    Deleted(String),
}

/// What the command needs from the outside, so that tests do not need a terminal, a browser or
/// a clock.
pub struct Io<'a> {
    /// Reads the secret of `credential set`.
    pub read_secret: &'a dyn Fn() -> Result<SecretString, CodedError>,
    /// Shows the page and the code of a device sign-in.
    pub announce: &'a dyn Fn(&Prompt),
    /// Pauses between two polls of the device flow.
    pub wait: &'a dyn Fn(Duration),
}

/// Runs `action` against the credential store of the operating system.
///
/// # Errors
/// See [`execute`].
pub fn run(action: &CredentialAction, io: &Io<'_>) -> Result<Done, CodedError> {
    execute(&OsCredentialStore::new(OsKeystore), action, io)
}

/// Runs `action` against `store`.
///
/// # Errors
/// - `CLI-101` for the name of the installation token, which belongs to the program, or an empty
///   name or user.
/// - `CLI-103` if no secret can be read or it is empty.
/// - `NET-601` if the device flow fails.
/// - `SEC-402` or `SEC-601` if the keystore cannot be used.
pub fn execute(
    store: &impl CredentialStore,
    action: &CredentialAction,
    io: &Io<'_>,
) -> Result<Done, CodedError> {
    match action {
        CredentialAction::Set { id, user } => set(store, id, user, io),
        CredentialAction::Delete { id } => {
            let id = usable_id(id)?;
            refresh::remove(store, &id)?;
            Ok(Done::Deleted(id.to_string()))
        }
        CredentialAction::Login {
            id,
            provider,
            client_id,
            host,
        } => login(store, id, (*provider, client_id, host.as_deref()), io),
    }
}

fn set(
    store: &impl CredentialStore,
    id: &str,
    user: &str,
    io: &Io<'_>,
) -> Result<Done, CodedError> {
    let id = usable_id(id)?;
    if user.trim().is_empty() {
        return Err(invalid());
    }
    let secret = (io.read_secret)()?;
    refresh::forget(store, &id)?;
    store.store(&id, &Credential::new(SecretString::from(user), secret))?;
    Ok(Done::Stored(id.to_string()))
}

/// Signs in at `(provider, client id, host)` and stores the token as `id`.
fn login(
    store: &impl CredentialStore,
    id: &str,
    (provider, client_id, host): (ProviderChoice, &str, Option<&str>),
    io: &Io<'_>,
) -> Result<Done, CodedError> {
    let id = usable_id(id)?;
    let flow = provider_of(provider).flow(client_id, host)?;
    let signed =
        flow.sign_in_with_refresh(|prompt| (io.announce)(prompt), |pause| (io.wait)(pause))?;
    let link = Link {
        provider: provider_of(provider),
        client_id,
        host,
    };
    refresh::save(store, &id, signed, &link)?;
    Ok(Done::Stored(id.to_string()))
}

const fn provider_of(choice: ProviderChoice) -> Provider {
    match choice {
        ProviderChoice::GitHub => Provider::GitHub,
        ProviderChoice::GitLab => Provider::GitLab,
    }
}

/// The name of a credential that a user may manage. The installation token is not one of them:
/// replacing it would lock the browser out, deleting it would break the next `serve`.
fn usable_id(name: &str) -> Result<CredentialId, CodedError> {
    if is_reserved_credential(name) {
        return Err(invalid());
    }
    CredentialId::new(name).map_err(|_| invalid())
}

fn invalid() -> CodedError {
    CodedError::new(ErrorCode::CliArgumentsInvalid)
}

/// Reads a secret: hidden from the terminal when there is one, otherwise the first line of
/// standard input. `prompt` is shown on the terminal only.
///
/// # Errors
/// `CLI-103` if nothing can be read or the secret is empty.
pub fn read_secret(prompt: &str) -> Result<SecretString, CodedError> {
    let stdin = io::stdin();
    let text = if stdin.is_terminal() {
        let mut err = io::stderr();
        let _ = err.write_all(prompt.as_bytes());
        let _ = err.write_all(LINE_BREAK.as_bytes());
        Zeroizing::new(rpassword::read_password().map_err(|_| unreadable())?)
    } else {
        let mut line = Zeroizing::new(String::new());
        stdin
            .lock()
            .take(MAX_SECRET_BYTES)
            .read_line(&mut line)
            .map_err(|_| unreadable())?;
        line
    };
    let secret = text.trim_end_matches([LINE_FEED, CARRIAGE_RETURN]);
    if secret.is_empty() {
        return Err(unreadable());
    }
    Ok(SecretString::from(secret))
}

fn unreadable() -> CodedError {
    CodedError::new(ErrorCode::SecretInputInvalid)
}

#[cfg(test)]
mod tests {
    use codeskimr_codes::Coded;
    use codeskimr_secrets::KeystoreCredentials;
    use secrecy::ExposeSecret;

    use super::*;
    use crate::consts::TOKEN_CREDENTIAL_ID;
    use crate::memory_keystore::Memory;

    const CANARY: &str = "CANARY-secret-4d21";

    type Store = KeystoreCredentials<Memory>;

    /// Runs `action` on `store`; `secret` is what the terminal would deliver.
    fn run_on(
        store: &Store,
        action: &CredentialAction,
        secret: Option<&str>,
    ) -> Result<Done, CodedError> {
        let read = || secret.map(SecretString::from).ok_or_else(unreadable);
        let (announce, wait) = (|_: &Prompt| {}, |_: Duration| {});
        let io = Io {
            read_secret: &read,
            announce: &announce,
            wait: &wait,
        };
        execute(store, action, &io)
    }

    fn set(id: &str) -> CredentialAction {
        CredentialAction::Set {
            id: id.to_owned(),
            user: "anton".to_owned(),
        }
    }

    fn delete(id: &str) -> CredentialAction {
        CredentialAction::Delete { id: id.to_owned() }
    }

    fn code(result: Result<Done, CodedError>) -> ErrorCode {
        result.map_or_else(|e| e.code(), |_| ErrorCode::UnexpectedInternalError)
    }

    fn holds_canary(store: &Store, id: &str) -> bool {
        let id = CredentialId::new(id).unwrap();
        store
            .with_credential(&id, |c| Ok(c.secret().expose_secret() == CANARY))
            .unwrap_or(false)
    }

    #[test]
    fn a_credential_is_stored_and_removed() {
        let store = Store::new(Memory::default());
        let stored = run_on(&store, &set("work"), Some(CANARY));
        assert_eq!(stored.unwrap(), Done::Stored("work".to_owned()));
        assert!(holds_canary(&store, "work"));

        let removed = run_on(&store, &delete("work"), None);
        assert_eq!(removed.unwrap(), Done::Deleted("work".to_owned()));
        assert!(!holds_canary(&store, "work"));
    }

    #[test]
    fn the_installation_token_cannot_be_touched() {
        let store = Store::new(Memory::default());
        for action in [set(TOKEN_CREDENTIAL_ID), delete(TOKEN_CREDENTIAL_ID)] {
            let result = run_on(&store, &action, Some(CANARY));
            assert_eq!(code(result), ErrorCode::CliArgumentsInvalid);
        }
    }

    #[test]
    fn an_unreadable_secret_stores_nothing() {
        let store = Store::new(Memory::default());
        let result = run_on(&store, &set("work"), None);
        assert_eq!(code(result), ErrorCode::SecretInputInvalid);
        assert!(!holds_canary(&store, "work"));
    }

    #[test]
    fn a_login_without_https_is_refused_before_any_request() {
        let store = Store::new(Memory::default());
        let action = CredentialAction::Login {
            id: "work".to_owned(),
            provider: ProviderChoice::GitLab,
            client_id: "app".to_owned(),
            host: Some("http://git.example.com".to_owned()),
        };
        assert_eq!(
            code(run_on(&store, &action, None)),
            ErrorCode::OauthFlowFailed
        );
    }

    const DEVICE_ANSWER: &str = r#"{"device_code":"d","user_code":"u","verification_uri":"https://example.com/d","expires_in":900,"interval":1}"#;
    const TOKEN_ANSWER: &str =
        r#"{"access_token":"CANARY-access-1","refresh_token":"CANARY-refresh-1"}"#;

    fn login_action(host: String) -> CredentialAction {
        CredentialAction::Login {
            id: "work".to_owned(),
            provider: ProviderChoice::GitLab,
            client_id: "app".to_owned(),
            host: Some(host),
        }
    }

    /// The user and the secret of an entry.
    fn entry(store: &Store, name: &str) -> Option<(String, String)> {
        let id = CredentialId::new(name).unwrap();
        store
            .with_credential(&id, |c| {
                let user = c.user().expose_secret().to_owned();
                Ok((user, c.secret().expose_secret().to_owned()))
            })
            .ok()
    }

    #[test]
    fn a_login_keeps_the_refresh_token_with_the_place_to_renew_it() {
        let store = Store::new(Memory::default());
        let (host, _) = crate::oauth_stub::serve(&[DEVICE_ANSWER, TOKEN_ANSWER]);
        let done = run_on(&store, &login_action(host.clone()), None);
        assert_eq!(done.unwrap(), Done::Stored("work".to_owned()));
        let access = entry(&store, "work").unwrap();
        assert!(access.1 == "CANARY-access-1", "wrong access token");
        let (link, token) = entry(&store, "work.refresh").unwrap();
        assert_eq!(link, format!("gitlab\tapp\t{host}"));
        assert!(token == "CANARY-refresh-1", "wrong refresh token");
    }

    #[test]
    fn a_new_secret_and_a_delete_remove_the_refresh_entry() {
        let store = Store::new(Memory::default());
        let (host, _) =
            crate::oauth_stub::serve(&[DEVICE_ANSWER, TOKEN_ANSWER, DEVICE_ANSWER, TOKEN_ANSWER]);
        run_on(&store, &login_action(host.clone()), None).unwrap();
        run_on(&store, &set("work"), Some(CANARY)).unwrap();
        assert!(entry(&store, "work.refresh").is_none());
        assert!(holds_canary(&store, "work"));
        run_on(&store, &login_action(host), None).unwrap();
        run_on(&store, &delete("work"), None).unwrap();
        assert!(entry(&store, "work").is_none());
        assert!(entry(&store, "work.refresh").is_none());
    }

    #[test]
    fn the_refresh_entry_of_a_credential_cannot_be_managed_by_name() {
        let store = Store::new(Memory::default());
        for name in ["work.refresh", "Work.Refresh ", " work.REFRESH"] {
            for action in [set(name), delete(name)] {
                let result = run_on(&store, &action, Some(CANARY));
                assert_eq!(code(result), ErrorCode::CliArgumentsInvalid, "{name:?}");
            }
        }
    }
}
