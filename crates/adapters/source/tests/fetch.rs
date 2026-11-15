//! `SourceFetcher`: folders and local Git repositories.
#![allow(missing_docs)]

mod common;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::SourceProvider;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{BranchName, GitUrl, ProjectSource};
use codeskimr_source::{GitCache, SourceFetcher};
use common::{TestResult, TestStore, folder, git, write};
use std::path::Path;
use std::sync::Arc;

fn fetcher(cache: &Path) -> SourceFetcher<TestStore> {
    SourceFetcher::new(
        GitCache::new(cache.to_path_buf()),
        Arc::new(TestStore::default()),
    )
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
        credential: None,
    })
}

/// A local origin repository, an empty cache and the Git source that points at the origin.
struct Fixture {
    remote: tempfile::TempDir,
    cache: tempfile::TempDir,
    source: ProjectSource,
}

fn fixture() -> TestResult<Fixture> {
    let remote = tempfile::tempdir()?;
    let url = origin(remote.path())?;
    Ok(Fixture {
        source: git_source(&url, "main")?,
        remote,
        cache: tempfile::tempdir()?,
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

/// Commits a change to the origin: `lib.rs` changes, `old.rs` goes, `new.rs` comes.
fn change_origin(remote: &Path) -> TestResult {
    write(remote, "lib.rs", "fn two() {}")?;
    write(remote, "new.rs", "fn new() {}")?;
    git(remote, &["rm", "-q", "--cached", "-r", "."])?;
    git(remote, &["add", "-A"])?;
    git(remote, &["commit", "-q", "-m", "two"])
}

#[test]
fn an_update_reuses_the_earlier_clone_and_mirrors_the_new_commit() -> TestResult {
    let Fixture {
        remote,
        cache,
        source,
    } = fixture()?;
    write(remote.path(), "old.rs", "fn old() {}")?;
    git(remote.path(), &["add", "."])?;
    git(remote.path(), &["commit", "-q", "-m", "old"])?;
    let first = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    let root = std::path::PathBuf::from(first.root.as_str());
    // A file in the Git data that only an update takes over; a new clone would not have it.
    std::fs::write(root.join(".git").join("kept-by-update"), "x")?;
    std::fs::remove_file(remote.path().join("old.rs"))?;
    change_origin(remote.path())?;
    let second = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    assert_ne!(second.commit, first.commit);
    assert!(root.join(".git").join("kept-by-update").exists());
    assert_eq!(std::fs::read_to_string(root.join("lib.rs"))?, "fn two() {}");
    assert_eq!(std::fs::read_to_string(root.join("new.rs"))?, "fn new() {}");
    assert!(!root.join("old.rs").exists());
    assert_eq!(entries(cache.path())?, 1);
    Ok(())
}

#[test]
fn a_broken_earlier_clone_is_replaced_by_a_new_clone() -> TestResult {
    let Fixture {
        remote,
        cache,
        source,
    } = fixture()?;
    let first = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    let root = std::path::PathBuf::from(first.root.as_str());
    std::fs::write(root.join(".git").join("HEAD"), "not a reference")?;
    change_origin(remote.path())?;
    let second = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    assert_ne!(second.commit, first.commit);
    assert_eq!(std::fs::read_to_string(root.join("lib.rs"))?, "fn two() {}");
    assert_eq!(entries(cache.path())?, 1);
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
    let Fixture {
        remote,
        cache,
        source,
    } = fixture()?;
    let first = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    drop(remote);
    let error = fetcher(cache.path())
        .fetch(&source, &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::GitFetchFailed);
    assert_eq!(entries(cache.path())?, 1);
    assert!(Path::new(first.root.as_str()).join("lib.rs").exists());
    Ok(())
}

fn entries(dir: &Path) -> TestResult<usize> {
    Ok(std::fs::read_dir(dir)?.count())
}

#[test]
fn a_cancelled_fetch_does_not_touch_the_old_copy() -> TestResult {
    let Fixture {
        remote: _remote,
        cache,
        source,
    } = fixture()?;
    let first = fetcher(cache.path()).fetch(&source, &CancelToken::new())?;
    let cancel = CancelToken::new();
    cancel.cancel();
    assert!(fetcher(cache.path()).fetch(&source, &cancel).is_err());
    assert_eq!(entries(cache.path())?, 1);
    assert!(Path::new(first.root.as_str()).join("lib.rs").exists());
    Ok(())
}

#[test]
fn credentials_in_a_url_never_show_up_in_a_failure() -> TestResult {
    const CANARY: &str = "CANARY-7c1f9e";
    let cache = tempfile::tempdir()?;
    let url = GitUrl::new(format!("https://user:{CANARY}@127.0.0.1:1/repo.git"))?;
    let error = fetcher(cache.path())
        .fetch(&git_source(&url, "main")?, &CancelToken::new())
        .unwrap_err();
    let mut texts = vec![error.to_string(), format!("{error:?}")];
    let mut next = std::error::Error::source(&error);
    while let Some(inner) = next {
        texts.push(inner.to_string());
        texts.push(format!("{inner:?}"));
        next = inner.source();
    }
    assert!(texts.iter().all(|text| !text.contains(CANARY)), "{texts:?}");
    Ok(())
}
