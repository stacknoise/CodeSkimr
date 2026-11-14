//! `WalkDiscovery`: ignore rules, languages, change detection, limits, cancellation.
#![allow(missing_docs)]

mod common;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::{DiscoveredFile, FileDiscovery};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{FileId, Loc, SnapshotId, SourceFile};
use codeskimr_source::{DiscoveryConfig, WalkDiscovery};
use common::{TestResult, folder, languages, write};

fn discovery(max_total_bytes: u64) -> TestResult<WalkDiscovery> {
    Ok(WalkDiscovery::new(
        DiscoveryConfig {
            excluded_folders: vec!["target".to_owned()],
            max_total_bytes,
        },
        languages()?,
    ))
}

fn paths(files: &[DiscoveredFile]) -> Vec<&str> {
    files.iter().map(|file| file.path.as_str()).collect()
}

fn stored(file: &DiscoveredFile) -> SourceFile {
    SourceFile {
        id: FileId::new(1),
        snapshot: SnapshotId::new(1),
        path: file.path.clone(),
        language: file.language.clone(),
        content_hash: file.content_hash.clone(),
        loc: Loc(1),
    }
}

#[test]
fn finds_known_languages_with_relative_paths() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "src/main.rs", "fn main() {}")?;
    write(dir.path(), "tools/run.PY", "print(1)")?;
    write(dir.path(), "notes.txt", "not code")?;
    let files = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    assert_eq!(paths(&files), ["src/main.rs", "tools/run.PY"]);
    assert_eq!(files[0].language.as_str(), "rust");
    assert_eq!(files[1].language.as_str(), "python");
    assert!(files.iter().all(|file| file.changed));
    Ok(())
}

#[test]
fn honours_gitignore_and_excluded_folders() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), ".gitignore", "generated.rs\n")?;
    write(dir.path(), "generated.rs", "fn a() {}")?;
    write(dir.path(), "kept.rs", "fn b() {}")?;
    write(dir.path(), "target/debug/build.rs", "fn c() {}")?;
    write(dir.path(), "a/target/deep.rs", "fn d() {}")?;
    let files = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    assert_eq!(paths(&files), ["kept.rs"]);
    Ok(())
}

#[test]
fn marks_only_changed_files() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "same.rs", "fn a() {}")?;
    write(dir.path(), "edited.rs", "fn b() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    let previous: Vec<SourceFile> = first.iter().map(stored).collect();
    write(dir.path(), "edited.rs", "fn b() { 1; }")?;
    write(dir.path(), "added.rs", "fn c() {}")?;
    let second =
        discovery(u64::MAX)?.discover(&folder(dir.path())?, &previous, &CancelToken::new())?;
    let changed: Vec<(&str, bool)> = second
        .iter()
        .map(|file| (file.path.as_str(), file.changed))
        .collect();
    assert_eq!(
        changed,
        [("added.rs", true), ("edited.rs", true), ("same.rs", false)]
    );
    Ok(())
}

#[test]
fn equal_content_has_equal_hash() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "a.rs", "fn x() {}")?;
    write(dir.path(), "b.rs", "fn x() {}")?;
    let files = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    assert_eq!(files[0].content_hash, files[1].content_hash);
    Ok(())
}

#[test]
fn rejects_a_source_over_the_size_limit() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "a.rs", "0123456789")?;
    write(dir.path(), "b.rs", "0123456789")?;
    let error = discovery(15)?
        .discover(&folder(dir.path())?, &[], &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SourceTooLarge);
    Ok(())
}

#[test]
fn missing_folder_is_reported() -> TestResult {
    let dir = tempfile::tempdir()?;
    let error = discovery(u64::MAX)?
        .discover(
            &folder(&dir.path().join("missing"))?,
            &[],
            &CancelToken::new(),
        )
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SourcePathNotFound);
    Ok(())
}

#[test]
fn a_cancelled_walk_returns_the_partial_list_as_documented() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "a.rs", "fn a() {}")?;
    let cancel = CancelToken::new();
    cancel.cancel();
    let files = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &cancel)?;
    assert_eq!(files, Vec::new());
    Ok(())
}

#[test]
fn scans_hidden_files_but_never_the_git_folder() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), ".hidden/secret.rs", "fn a() {}")?;
    write(dir.path(), ".git/hooks/x.rs", "fn b() {}")?;
    let files = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    assert_eq!(paths(&files), [".hidden/secret.rs"]);
    Ok(())
}

#[test]
fn ignore_files_of_parent_folders_do_not_count() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), ".gitignore", "*.rs\n")?;
    write(dir.path(), "project/code.rs", "fn a() {}")?;
    let root = dir.path().join("project");
    let files = discovery(u64::MAX)?.discover(&folder(&root)?, &[], &CancelToken::new())?;
    assert_eq!(paths(&files), ["code.rs"]);
    Ok(())
}

#[test]
fn files_without_a_known_language_do_not_count_against_the_limit() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "big.bin", &"x".repeat(1_000))?;
    write(dir.path(), "small.rs", "fn a() {}")?;
    let files = discovery(100)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    assert_eq!(paths(&files), ["small.rs"]);
    Ok(())
}

#[test]
fn a_changed_language_marks_the_file_changed() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "same.rs", "fn a() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    let mut previous: Vec<SourceFile> = first.iter().map(stored).collect();
    previous[0].language = codeskimr_domain::LanguageId::new("other")?;
    let second =
        discovery(u64::MAX)?.discover(&folder(dir.path())?, &previous, &CancelToken::new())?;
    assert!(second[0].changed);
    Ok(())
}

#[cfg(unix)]
#[test]
fn an_unreadable_file_fails_the_run() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir()?;
    write(dir.path(), "locked.rs", "fn a() {}")?;
    let locked = dir.path().join("locked.rs");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))?;
    if std::fs::read(&locked).is_ok() {
        // Running with rights that ignore file modes (for example as root): nothing to test.
        return Ok(());
    }
    let error = discovery(u64::MAX)?
        .discover(&folder(dir.path())?, &[], &CancelToken::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SourcePathNotFound);
    Ok(())
}

#[test]
fn only_the_listed_files_are_checked_and_a_changed_or_missing_one_is_left_out() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "same.rs", "fn a() {}")?;
    write(dir.path(), "edited.rs", "fn b() {}")?;
    write(dir.path(), "gone.rs", "fn c() {}")?;
    write(dir.path(), "other.rs", "fn d() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    // `other.rs` is not in the list: it is neither read nor reported.
    let listed: Vec<SourceFile> = first
        .iter()
        .filter(|file| file.path.as_str() != "other.rs")
        .map(stored)
        .collect();
    write(dir.path(), "edited.rs", "fn b() { 1; }")?;
    std::fs::remove_file(dir.path().join("gone.rs"))?;
    let unchanged =
        discovery(u64::MAX)?.unchanged_files(&folder(dir.path())?, &listed, &CancelToken::new())?;
    let names: Vec<&str> = unchanged
        .iter()
        .map(codeskimr_domain::RelativePath::as_str)
        .collect();
    assert_eq!(names, ["same.rs"]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_link_in_place_of_a_listed_file_counts_as_changed() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "real.rs", "fn a() {}")?;
    write(dir.path(), "linked.rs", "fn a() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    let listed: Vec<SourceFile> = first.iter().map(stored).collect();
    std::fs::remove_file(dir.path().join("linked.rs"))?;
    std::os::unix::fs::symlink(dir.path().join("real.rs"), dir.path().join("linked.rs"))?;
    let unchanged =
        discovery(u64::MAX)?.unchanged_files(&folder(dir.path())?, &listed, &CancelToken::new())?;
    let names: Vec<&str> = unchanged
        .iter()
        .map(codeskimr_domain::RelativePath::as_str)
        .collect();
    assert_eq!(names, ["real.rs"]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_named_pipe_in_place_of_a_listed_file_counts_as_changed_and_does_not_block() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "piped.rs", "fn a() {}")?;
    write(dir.path(), "kept.rs", "fn b() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    let listed: Vec<SourceFile> = first.iter().map(stored).collect();
    let pipe = dir.path().join("piped.rs");
    std::fs::remove_file(&pipe)?;
    if !std::process::Command::new("mkfifo")
        .arg(&pipe)
        .status()
        .is_ok_and(|status| status.success())
    {
        // No `mkfifo` here: nothing to test.
        return Ok(());
    }
    let root = folder(dir.path())?;
    let checker = discovery(u64::MAX)?;
    let (done, result) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        done.send(checker.unchanged_files(&root, &listed, &CancelToken::new()))
            .ok();
    });
    let unchanged = result
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("a named pipe must not block the check")?;
    let names: Vec<&str> = unchanged
        .iter()
        .map(codeskimr_domain::RelativePath::as_str)
        .collect();
    assert_eq!(names, ["kept.rs"]);
    Ok(())
}

#[test]
fn a_file_that_grew_or_that_is_over_the_total_limit_counts_as_changed() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "a.rs", "fn a() {}")?;
    write(dir.path(), "b.rs", "fn b() {}")?;
    let first = discovery(u64::MAX)?.discover(&folder(dir.path())?, &[], &CancelToken::new())?;
    let listed: Vec<SourceFile> = first.iter().map(stored).collect();
    // Room for one of the two files only: the second is not read and counts as changed.
    let room = "fn a() {}".len() as u64;
    let unchanged =
        discovery(room)?.unchanged_files(&folder(dir.path())?, &listed, &CancelToken::new())?;
    assert_eq!(unchanged.len(), 1);
    // A file that grew no longer matches, whatever the limit is.
    write(dir.path(), "a.rs", &"fn a() {}".repeat(1000))?;
    let unchanged =
        discovery(u64::MAX)?.unchanged_files(&folder(dir.path())?, &listed, &CancelToken::new())?;
    let names: Vec<&str> = unchanged
        .iter()
        .map(codeskimr_domain::RelativePath::as_str)
        .collect();
    assert_eq!(names, ["b.rs"]);
    Ok(())
}
