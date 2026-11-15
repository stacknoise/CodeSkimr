//! `SourceFetcher`: folders and local Git repositories.
#![allow(missing_docs)]

mod common;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::{FetchedSource, SourceProvider};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{BranchName, GitUrl, ProjectSource};
use codeskimr_source::{GitCache, SourceFetcher};
use common::{TestResult, TestStore, folder, git, write};
use std::path::{Path, PathBuf};
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
    let checked_out = PathBuf::from(first.root.as_str()).join("lib.rs");
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

/// Two fetches of the fixture's origin with `change` applied to the origin in between. The Git data
/// of the cached clone is marked after the first fetch, so that a test can tell an update (the
/// mark stays) from a new clone (the mark is gone).
struct Updated {
    fixture: Fixture,
    root: PathBuf,
    first: FetchedSource,
    second: Result<FetchedSource, codeskimr_application::error::CodedError>,
}

impl Updated {
    fn new(change: impl FnOnce(&Path) -> TestResult) -> TestResult<Self> {
        let fixture = fixture()?;
        let first = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
        let root = PathBuf::from(first.root.as_str());
        std::fs::write(root.join(".git").join(MARK), "x")?;
        change(fixture.remote.path())?;
        let second = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new());
        Ok(Self {
            fixture,
            root,
            first,
            second,
        })
    }

    fn was_updated(&self) -> bool {
        self.root.join(".git").join(MARK).exists()
    }

    fn content(&self, file: &str) -> TestResult<String> {
        Ok(std::fs::read_to_string(self.root.join(file))?)
    }

    /// The second fetch worked and led to another commit.
    fn moved(&self) -> TestResult<bool> {
        let second = self.second.as_ref().map_err(ToString::to_string)?;
        Ok(second.commit != self.first.commit)
    }
}

/// Name of the file that marks a clone as updated in place.
const MARK: &str = "kept-by-update";

#[test]
fn an_update_reuses_the_earlier_clone_and_mirrors_the_new_commit() -> TestResult {
    let updated = Updated::new(|remote| {
        write(remote, "old.rs", "fn old() {}")?;
        git(remote, &["add", "."])?;
        git(remote, &["commit", "-q", "-m", "old"])?;
        change_origin(remote)
    })?;
    assert!(updated.moved()? && updated.was_updated());
    assert_eq!(updated.content("lib.rs")?, "fn two() {}");
    assert_eq!(updated.content("new.rs")?, "fn new() {}");
    assert_eq!(entries(updated.fixture.cache.path())?, 1);
    Ok(())
}

#[test]
fn an_update_removes_files_that_are_gone() -> TestResult {
    let updated = Updated::new(|remote| {
        git(remote, &["rm", "-q", "lib.rs"])?;
        write(remote, "other.rs", "fn other() {}")?;
        git(remote, &["add", "."])?;
        git(remote, &["commit", "-q", "-m", "swap"])
    })?;
    assert!(updated.moved()? && updated.was_updated());
    assert!(!updated.root.join("lib.rs").exists());
    assert_eq!(updated.content("other.rs")?, "fn other() {}");
    Ok(())
}

#[test]
fn an_update_without_a_new_commit_changes_nothing() -> TestResult {
    let updated = Updated::new(|_| Ok(()))?;
    assert!(!updated.moved()? && updated.was_updated());
    assert_eq!(updated.content("lib.rs")?, "fn one() {}");
    Ok(())
}

#[test]
fn an_update_follows_a_force_push() -> TestResult {
    let updated = Updated::new(|remote| {
        write(remote, "lib.rs", "fn rewritten() {}")?;
        git(remote, &["commit", "-q", "-a", "--amend", "-m", "again"])
    })?;
    assert!(updated.moved()? && updated.was_updated());
    assert_eq!(updated.content("lib.rs")?, "fn rewritten() {}");
    Ok(())
}

#[test]
fn a_branch_that_is_gone_is_reported_and_keeps_the_old_copy() -> TestResult {
    let updated = Updated::new(|remote| git(remote, &["branch", "-m", "main", "renamed"]))?;
    let error = updated.second.as_ref().expect_err("the branch is gone");
    assert_eq!(error.code(), ErrorCode::GitBranchNotFound);
    assert_eq!(updated.content("lib.rs")?, "fn one() {}");
    assert_eq!(entries(updated.fixture.cache.path())?, 1);
    Ok(())
}

#[test]
fn a_broken_earlier_clone_is_replaced_by_a_new_clone() -> TestResult {
    let fixture = fixture()?;
    let first = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
    let root = PathBuf::from(first.root.as_str());
    std::fs::write(root.join(".git").join("HEAD"), "not a reference")?;
    change_origin(fixture.remote.path())?;
    let second = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
    assert_ne!(second.commit, first.commit);
    assert_eq!(std::fs::read_to_string(root.join("lib.rs"))?, "fn two() {}");
    assert_eq!(entries(fixture.cache.path())?, 1);
    Ok(())
}

/// Collects what the adapter logs while `work` runs.
fn logged<T>(work: impl FnOnce() -> TestResult<T>) -> TestResult<(T, String)> {
    #[derive(Clone, Default)]
    struct Captured(Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut held = self
                .0
                .lock()
                .map_err(|_| std::io::Error::other("poisoned"))?;
            held.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
        type Writer = Self;
        fn make_writer(&'a self) -> Self {
            self.clone()
        }
    }
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(captured.clone())
        .with_ansi(false)
        .finish();
    let result = tracing::subscriber::with_default(subscriber, work)?;
    let text = captured.0.lock().map_err(|_| "poisoned")?.clone();
    Ok((result, String::from_utf8(text)?))
}

#[test]
fn a_fallback_to_a_new_clone_logs_its_reason() -> TestResult {
    let fixture = fixture()?;
    let first = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
    std::fs::write(
        PathBuf::from(first.root.as_str()).join(".git").join("HEAD"),
        "not a reference",
    )?;
    change_origin(fixture.remote.path())?;
    let (second, log) =
        logged(|| Ok(fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?))?;
    assert_ne!(second.commit, first.commit);
    assert!(log.contains("WARN"), "{log}");
    assert!(log.contains("reason="), "{log}");
    assert!(log.contains("code=\"SRC-"), "{log}");
    Ok(())
}

#[test]
fn an_update_logs_nothing_and_a_first_clone_neither() -> TestResult {
    let fixture = fixture()?;
    let (_, log) = logged(|| {
        let first = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
        change_origin(fixture.remote.path())?;
        let second = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
        Ok(second.commit != first.commit)
    })?;
    assert_eq!(log, "");
    Ok(())
}

#[test]
fn a_clone_with_many_packs_is_cloned_again_and_holds_one_pack() -> TestResult {
    let fixture = fixture()?;
    let first = fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?;
    let root = PathBuf::from(first.root.as_str());
    let packs = root.join(".git").join("objects").join("pack");
    for number in 0..20 {
        std::fs::write(packs.join(format!("pack-extra{number}.pack")), "")?;
    }
    std::fs::write(root.join(".git").join(MARK), "x")?;
    change_origin(fixture.remote.path())?;
    let (second, log) =
        logged(|| Ok(fetcher(fixture.cache.path()).fetch(&fixture.source, &CancelToken::new())?))?;
    assert_ne!(second.commit, first.commit);
    assert!(
        !root.join(".git").join(MARK).exists(),
        "the clone was updated, not cloned again"
    );
    assert!(log.contains("INFO"), "{log}");
    assert!(log.contains("too many packs"), "{log}");
    let kept = std::fs::read_dir(&packs)?
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "pack"))
        .count();
    assert_eq!(kept, 1);
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
