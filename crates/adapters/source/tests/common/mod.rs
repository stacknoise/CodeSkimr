//! Helpers shared by the integration tests.
#![allow(dead_code, missing_docs)]

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use codeskimr_domain::{FolderPath, LanguageId};
use codeskimr_source::LanguageMap;

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
