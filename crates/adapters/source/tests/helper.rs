//! A configured credential never asks the system's credential helper (issue #58, point 1).
//!
//! Git reads its global configuration from the environment, which a test must not change for
//! its own process. The test therefore starts itself as a child process with a `.gitconfig` whose
//! helper leaves a marker file, and looks for the marker afterwards.
#![allow(missing_docs)]

mod common;

use gix_transport as _;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::SourceProvider;
use codeskimr_domain::{BranchName, CredentialId, GitUrl, ProjectSource};
use codeskimr_source::{GitCache, SourceFetcher};
use common::http::GitHttpServer;
use common::{TestResult, TestStore};

const CHILD_URL: &str = "CODESKIMR_TEST_URL";
const CHILD_CACHE: &str = "CODESKIMR_TEST_CACHE";
const CHILD_CREDENTIAL: &str = "CODESKIMR_TEST_CREDENTIAL";
const USER: &str = "anton";
const SECRET: &str = "helper-test-secret";

/// The body of the child process; it does nothing when the test runs on its own.
#[test]
fn child_fetches_once() -> TestResult {
    let (Ok(url), Ok(cache)) = (std::env::var(CHILD_URL), std::env::var(CHILD_CACHE)) else {
        return Ok(());
    };
    let with_credential = std::env::var(CHILD_CREDENTIAL).is_ok();
    let store = if with_credential {
        TestStore::with("work", USER, SECRET)
    } else {
        TestStore::default()
    };
    let source = ProjectSource::Git {
        url: GitUrl::new(url)?,
        branch: BranchName::new("main")?,
        credential: with_credential
            .then(|| CredentialId::new("work"))
            .transpose()?,
    };
    let fetcher = SourceFetcher::new(GitCache::new(PathBuf::from(cache)), Arc::new(store));
    // The outcome is not the point; whether the helper was asked is.
    let _ = fetcher.fetch(&source, &CancelToken::new());
    Ok(())
}

/// Runs the child with a helper that creates `marker`; returns whether the helper was asked.
fn helper_was_asked(with_credential: bool) -> TestResult<bool> {
    let root = tempfile::tempdir()?;
    common::bare_repo(root.path())?;
    let server = GitHttpServer::start(root.path(), Some((USER, SECRET)))?;

    let home = tempfile::tempdir()?;
    let marker = home.path().join("helper-was-asked");
    let config = home.path().join("gitconfig");
    std::fs::write(
        &config,
        format!("[credential]\n\thelper = !touch {}\n", marker.display()),
    )?;
    let mut child = Command::new(std::env::current_exe()?);
    child
        .args(["--exact", "child_fetches_once", "--test-threads=1"])
        .env(
            CHILD_URL,
            format!("http://127.0.0.1:{}/repo.git", server.port),
        )
        .env(CHILD_CACHE, home.path().join("cache"))
        .env("GIT_CONFIG_GLOBAL", &config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // The helper is run as `touch <marker> get`, which also creates a file called `get`.
        .current_dir(home.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if with_credential {
        child.env(CHILD_CREDENTIAL, "1");
    }
    assert!(child.status()?.success());
    Ok(marker.exists())
}

#[test]
fn without_a_credential_the_helper_decides_and_a_configured_one_skips_it() -> TestResult {
    // The control: without a credential the helper is asked, so the setup does reach it.
    assert!(helper_was_asked(false)?);
    assert!(!helper_was_asked(true)?);
    Ok(())
}
