//! `SourceFetcher`: folders and local Git repositories.
#![allow(missing_docs)]

mod common;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::SourceProvider;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{BranchName, GitUrl, ProjectSource};
use codeskimr_source::{GitCache, SourceFetcher};
use common::{TestResult, folder, git, write};
use std::path::Path;

fn fetcher(cache: &Path) -> SourceFetcher {
    SourceFetcher::new(GitCache::new(cache.to_path_buf()))
}

fn origin(dir: &Path) -> TestResult<GitUrl> {
    git(dir, &["init", "-q", "-b", "main"])?;
    write(dir, "lib.rs", "fn one() {}")?;
    git(dir, &["add", "."])?;
    git(dir, &["commit", "-q", "-m", "one"])?;
    Ok(GitUrl::new(format!("file://{}", dir.display()))?)
}

fn git_source(url: &GitUrl, branch: &str) -> TestResult<ProjectSource> {
    Ok(ProjectSource::Git {
        url: url.clone(),
        branch: BranchName::new(branch)?,
    })
}

#[test]
fn a_folder_source_is_used_in_place() -> TestResult {
    let dir = tempfile::tempdir()?;
    let fetched = fetcher(dir.path()).fetch(
        &ProjectSource::Folder(folder(dir.path())?),
        &CancelToken::new(),
    )?;
    assert_eq!(fetched.root, folder(dir.path())?);
    assert!(fetched.commit.is_none());
    Ok(())
}

#[test]
fn a_missing_folder_is_reported() -> TestResult {
    let dir = tempfile::tempdir()?;
    let source = ProjectSource::Folder(folder(&dir.path().join("missing"))?);
    let error = fetcher(dir.path())
        .fetch(&source, &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SourcePathNotFound);
    Ok(())
}

#[test]
fn clones_a_branch_and_follows_new_commits() -> TestResult {
    let remote = tempfile::tempdir()?;
    let cache = tempfile::tempdir()?;
    // The cache folder does not exist yet and is created on first use.
    let cache_root = cache.path().join("nested").join("cache");
    let url = origin(remote.path())?;
    let source = git_source(&url, "main")?;
    let first = fetcher(&cache_root).fetch(&source, &CancelToken::new())?;
    let checked_out = std::path::PathBuf::from(first.root.as_str()).join("lib.rs");
    assert_eq!(std::fs::read_to_string(&checked_out)?, "fn one() {}");

    write(remote.path(), "lib.rs", "fn two() {}")?;
    git(remote.path(), &["commit", "-q", "-a", "-m", "two"])?;
    let second = fetcher(&cache_root).fetch(&source, &CancelToken::new())?;
    assert_eq!(second.root, first.root);
    assert_ne!(second.commit, first.commit);
    assert_eq!(std::fs::read_to_string(&checked_out)?, "fn two() {}");
    Ok(())
}

#[test]
fn a_missing_branch_is_reported() -> TestResult {
    let remote = tempfile::tempdir()?;
    let cache = tempfile::tempdir()?;
    let url = origin(remote.path())?;
    let error = fetcher(cache.path())
        .fetch(&git_source(&url, "nope")?, &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::GitBranchNotFound);
    Ok(())
}

#[test]
fn an_unreachable_remote_is_a_fetch_failure_and_keeps_the_old_copy() -> TestResult {
    let remote = tempfile::tempdir()?;
    let cache = tempfile::tempdir()?;
    let url = origin(remote.path())?;
    let source = git_source(&url, "main")?;
    let first = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    drop(remote);
    let error = fetcher(cache.path())
        .fetch(&source, &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::GitFetchFailed);
    assert!(
        std::path::Path::new(first.root.as_str())
            .join("lib.rs")
            .exists()
    );
    Ok(())
}
