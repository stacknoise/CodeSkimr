//! Logging in to a Git remote with a credential from the store (blueprint 12, issue #35).
#![allow(missing_docs)]

mod common;

// Only used to switch on `http-client-insecure-credentials` for the local plain-HTTP server.
use gix_transport as _;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::SourceProvider;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{BranchName, CredentialId, GitUrl, ProjectSource};
use codeskimr_source::{GitCache, SourceFetcher};
use common::http::GitHttpServer;
use common::{TestResult, TestStore};

const CANARY: &str = "CANARY-5e2a1b9d";
const USER: &str = "anton";

/// A bare repository served over HTTP below `root`, plus an empty cache folder.
struct Remote {
    root: tempfile::TempDir,
    cache: tempfile::TempDir,
}

fn remote() -> TestResult<Remote> {
    let root = tempfile::tempdir()?;
    common::bare_repo(root.path())?;
    Ok(Remote {
        root,
        cache: tempfile::tempdir()?,
    })
}

fn source(url: &str, credential: Option<&str>) -> TestResult<ProjectSource> {
    Ok(ProjectSource::Git {
        url: GitUrl::new(url)?,
        branch: BranchName::new("main")?,
        credential: credential.map(CredentialId::new).transpose()?,
    })
}

fn fetcher(cache: &Path, store: Arc<TestStore>) -> SourceFetcher<TestStore> {
    SourceFetcher::new(GitCache::new(cache.to_path_buf()), store)
}

/// All files below `dir`, with `.git` folders included.
fn files(dir: &Path) -> TestResult<Vec<std::path::PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            found.extend(files(&path)?);
        } else {
            found.push(path);
        }
    }
    Ok(found)
}

/// Whether any file below `dir` contains the secret as it is, or inside a Basic header value.
fn contains_anywhere(dir: &Path, needle: &str) -> TestResult<bool> {
    let encoded = common::http::base64(format!("{USER}:{needle}").as_bytes());
    let needles = [needle.to_owned(), encoded];
    for path in files(dir)? {
        let content = std::fs::read(&path)?;
        if needles.iter().any(|n| {
            content
                .windows(n.len())
                .any(|window| window == n.as_bytes())
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[test]
fn a_lent_credential_logs_in_and_never_reaches_the_clone() -> TestResult {
    let Remote { root, cache } = remote()?;
    let server = GitHttpServer::start(root.path(), Some((USER, CANARY)))?;
    let store = Arc::new(TestStore::with("work", USER, CANARY));
    let url = format!("http://127.0.0.1:{}/repo.git", server.port);

    let fetched = fetcher(cache.path(), store.clone())
        .fetch(&source(&url, Some("work"))?, &CancelToken::new())?;

    assert!(Path::new(fetched.root.as_str()).join("lib.rs").exists());
    assert!(server.authorization_count() >= 1);
    assert!(store.lent.load(Ordering::Relaxed) >= 1);
    // The cache holds no secret: not in `.git/config`, not anywhere else (issue #35).
    assert!(!contains_anywhere(cache.path(), CANARY)?);
    let config = std::fs::read_to_string(Path::new(fetched.root.as_str()).join(".git/config"))?;
    assert!(config.contains(&url), "{config}");
    Ok(())
}

#[test]
fn a_credential_the_store_does_not_have_is_reported_by_its_code() -> TestResult {
    let Remote { root, cache } = remote()?;
    let server = GitHttpServer::start(root.path(), Some((USER, CANARY)))?;
    let url = format!("http://127.0.0.1:{}/repo.git", server.port);

    let error = fetcher(cache.path(), Arc::new(TestStore::default()))
        .fetch(&source(&url, Some("missing"))?, &CancelToken::new())
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::CredentialNotFound);
    Ok(())
}

/// The error of a clone from the local server on `port` with the credential `work`.
fn failed_login(cache: &Path, port: u16) -> TestResult<CodedError> {
    let store = Arc::new(TestStore::with("work", USER, CANARY));
    let url = format!("http://127.0.0.1:{port}/repo.git");
    fetcher(cache, store)
        .fetch(&source(&url, Some("work"))?, &CancelToken::new())
        .err()
        .ok_or_else(|| "the clone should fail".into())
}

#[test]
fn a_rejected_credential_is_an_authentication_failure_without_the_secret() -> TestResult {
    let Remote { root, cache } = remote()?;
    let server = GitHttpServer::start(root.path(), Some((USER, "something-else")))?;

    let error = failed_login(cache.path(), server.port)?;

    assert_eq!(error.code(), ErrorCode::CredentialRejected, "{error:?}");
    let mut texts = vec![error.to_string(), format!("{error:?}")];
    let mut next = std::error::Error::source(&error);
    while let Some(inner) = next {
        texts.push(format!("{inner} {inner:?}"));
        next = inner.source();
    }
    assert!(texts.iter().all(|text| !text.contains(CANARY)), "{texts:?}");
    assert!(!contains_anywhere(cache.path(), CANARY)?);
    Ok(())
}

#[test]
fn a_password_or_token_in_the_url_is_dropped_before_git_writes_the_url_down() -> TestResult {
    for userinfo in [format!("{USER}:{CANARY}"), CANARY.to_owned()] {
        let Remote { root, cache } = remote()?;
        let server = GitHttpServer::start(root.path(), None)?;
        let url = format!("http://{userinfo}@127.0.0.1:{}/repo.git", server.port);

        let fetched = fetcher(cache.path(), Arc::new(TestStore::default()))
            .fetch(&source(&url, None)?, &CancelToken::new())?;

        assert!(Path::new(fetched.root.as_str()).join("lib.rs").exists());
        assert!(!contains_anywhere(cache.path(), CANARY)?);
    }
    Ok(())
}

#[test]
fn a_redirect_to_another_host_never_gets_the_credential() -> TestResult {
    let Remote { root, cache } = remote()?;
    let target = GitHttpServer::start(root.path(), Some((USER, CANARY)))?;
    let redirector = GitHttpServer::redirecting_to(target.port)?;
    let store = Arc::new(TestStore::with("work", USER, CANARY));
    let url = format!("http://127.0.0.1:{}/repo.git", redirector.port);

    let result = fetcher(cache.path(), store.clone())
        .fetch(&source(&url, Some("work"))?, &CancelToken::new());

    assert!(result.is_err());
    assert_eq!(store.lent.load(Ordering::Relaxed), 0);
    assert!(!contains_anywhere(cache.path(), CANARY)?);
    assert!(target.authorizations.lock().is_ok_and(|seen| {
        seen.iter()
            .all(|h| !h.contains(&common::http::base64(format!("{USER}:{CANARY}").as_bytes())))
    }));
    Ok(())
}

#[test]
fn a_host_that_hides_the_repository_from_a_bad_token_is_a_credential_problem() -> TestResult {
    // GitHub answers an invalid token for a private repository with 403 or 404, not with 401.
    for status in [403, 404] {
        let Remote { cache, .. } = remote()?;
        let server = GitHttpServer::refusing_with(status)?;

        let error = failed_login(cache.path(), server.port)?;

        assert_eq!(
            error.code(),
            ErrorCode::CredentialRejected,
            "{status}: {error:?}"
        );
    }
    Ok(())
}
