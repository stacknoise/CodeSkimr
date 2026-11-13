//! Helpers shared by the integration tests.
#![allow(dead_code, missing_docs)]

pub mod http;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use codeskimr_application::ports::{AuthError, Credential, CredentialStore};
use codeskimr_domain::{CredentialId, FolderPath, LanguageId};
use codeskimr_source::LanguageMap;
use secrecy::SecretString;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn folder(path: &Path) -> TestResult<FolderPath> {
    Ok(FolderPath::new(path.to_string_lossy().into_owned())?)
}

pub fn languages() -> TestResult<LanguageMap> {
    Ok([
        ("rs", LanguageId::new("rust")?),
        ("py", LanguageId::new("python")?),
    ]
    .into_iter()
    .collect())
}

pub fn write(root: &Path, relative: &str, content: &str) -> TestResult {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, content)?)
}

/// Runs `git` in `dir` and fails if it fails.
pub fn git(dir: &Path, args: &[&str]) -> TestResult {
    let status = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {args:?} failed").into())
    }
}

/// A credential store in memory that counts how often a credential was lent.
#[derive(Default)]
pub struct TestStore {
    entries: HashMap<String, (String, String)>,
    pub lent: AtomicUsize,
}

impl TestStore {
    pub fn with(id: &str, user: &str, secret: &str) -> Self {
        let mut store = Self::default();
        store
            .entries
            .insert(id.to_owned(), (user.to_owned(), secret.to_owned()));
        store
    }
}

impl CredentialStore for TestStore {
    fn with_credential<T>(
        &self,
        id: &CredentialId,
        action: impl FnOnce(&Credential) -> Result<T, AuthError>,
    ) -> Result<T, AuthError> {
        let (user, secret) = self.entries.get(id.as_str()).ok_or(AuthError::NotFound)?;
        self.lent.fetch_add(1, Ordering::Relaxed);
        action(&Credential::new(
            SecretString::from(user.as_str()),
            SecretString::from(secret.as_str()),
        ))
    }

    fn store(&self, _id: &CredentialId, _credential: &Credential) -> Result<(), AuthError> {
        Err(AuthError::AccessDenied)
    }

    fn delete(&self, _id: &CredentialId) -> Result<(), AuthError> {
        Err(AuthError::AccessDenied)
    }
}
