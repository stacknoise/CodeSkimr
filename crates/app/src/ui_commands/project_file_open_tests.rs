//! `project_file.open`: the file is opened by the opener of the test, never by the system
//! (ADR 0111).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use codeskimr_application::error::CodedError;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ProjectSource;
use serde_json::json;

use super::project_file_open::open_project_file;
use super::tests::Fixture;
use crate::file_opener::FileOpener;
use crate::source_arg::folder_spec;

const FILE: &str = ".codeskimr.toml";

/// Remembers what it was asked to open; fails on request.
struct Recorder {
    opened: Mutex<Vec<PathBuf>>,
    fails: bool,
    away: bool,
}

impl Recorder {
    fn new(fails: bool) -> Self {
        Self {
            opened: Mutex::new(Vec::new()),
            fails,
            away: false,
        }
    }

    /// An opener on a machine where nobody sits.
    fn away() -> Self {
        Self {
            away: true,
            ..Self::new(false)
        }
    }

    fn opened(&self) -> Vec<PathBuf> {
        self.opened.lock().unwrap().clone()
    }
}

impl FileOpener for Recorder {
    fn available(&self) -> bool {
        !self.away
    }

    fn open(&self, file: &Path) -> Result<(), CodedError> {
        self.opened.lock().unwrap().push(file.to_path_buf());
        if self.fails {
            return Err(CodedError::new(ErrorCode::ProjectFileOpenFailed));
        }
        Ok(())
    }
}

fn folder(dir: &Path) -> ProjectSource {
    folder_spec(dir).unwrap().source
}

#[test]
fn an_existing_file_is_opened_and_left_as_it_is() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(FILE), "keep = 1\n").unwrap();
    let opener = Recorder::new(false);
    let done = open_project_file(&opener, &folder(dir.path())).unwrap();
    assert_eq!((done.file.as_str(), done.created), (FILE, false));
    assert_eq!(opener.opened().len(), 1);
    assert!(opener.opened()[0].ends_with(FILE));
    assert_eq!(
        fs::read_to_string(dir.path().join(FILE)).unwrap(),
        "keep = 1\n"
    );
}

#[test]
fn a_missing_file_is_created_empty_and_then_opened() {
    let dir = tempfile::tempdir().unwrap();
    let opener = Recorder::new(false);
    let done = open_project_file(&opener, &folder(dir.path())).unwrap();
    assert!(done.created);
    assert_eq!(fs::read_to_string(dir.path().join(FILE)).unwrap(), "");
    assert_eq!(opener.opened().len(), 1);
}

/// Opening in `dir` is refused with `CFG-102` and the opener is never asked.
fn assert_refused(dir: &Path) {
    let opener = Recorder::new(false);
    let refused = open_project_file(&opener, &folder(dir)).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ConfigParseFailed);
    assert_eq!(opener.opened().len(), 0);
}

#[cfg(unix)]
#[test]
fn a_link_is_refused_and_never_followed() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("elsewhere.txt");
    fs::write(&target, "secret").unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join(FILE)).unwrap();
    assert_refused(dir.path());
    assert_eq!(fs::read_to_string(&target).unwrap(), "secret");
}

#[test]
fn a_folder_in_place_of_the_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join(FILE)).unwrap();
    assert_refused(dir.path());
}

#[test]
fn a_failing_system_is_reported_with_its_own_code() {
    let dir = tempfile::tempdir().unwrap();
    let opener = Recorder::new(true);
    let failed = open_project_file(&opener, &folder(dir.path())).unwrap_err();
    assert_eq!(failed.code(), ErrorCode::ProjectFileOpenFailed);
}

#[test]
fn a_git_project_has_no_file_to_open() {
    let fixture = Fixture::installed(None);
    fixture.registered_git(7);
    assert_eq!(
        fixture.call("project_file.open", json!({ "project": 7 })),
        Err("PRJ-101")
    );
}

#[test]
fn a_reader_is_refused_before_anything_happens() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let refused = fixture.call_as(
        crate::principal::reader(),
        "project_file.open",
        json!({ "project": project }),
    );
    assert_eq!(refused, Err("AUTH-401"));
    assert!(!fixture.root.path().join("app").join(FILE).exists());
}

#[test]
fn nothing_is_created_where_nobody_sits_at_the_machine() {
    let dir = tempfile::tempdir().unwrap();
    let opener = Recorder::away();
    let failed = open_project_file(&opener, &folder(dir.path())).unwrap_err();
    assert_eq!(failed.code(), ErrorCode::ProjectFileOpenFailed);
    assert_eq!(opener.opened().len(), 0);
    assert!(!dir.path().join(FILE).exists());
}

#[cfg(unix)]
#[test]
fn an_executable_file_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(FILE);
    fs::write(&file, "").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    assert_refused(dir.path());
}
