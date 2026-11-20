//! The folder dialog commands, with a dialog that answers as told instead of showing a window.

use std::path::PathBuf;
use std::sync::Arc;

use codeskimr_application::registry::CommandExt;
use serde_json::Value;

use super::projects_pick::{FolderDialogInfo, PickFolder};
use super::tests::Fixture;
use crate::folder_dialog::FolderPicker;
use crate::principal;

/// A dialog that is there or not and always answers the same.
struct Told {
    available: bool,
    answer: Option<PathBuf>,
}

impl FolderPicker for Told {
    fn available(&self) -> bool {
        self.available
    }

    fn pick(&self) -> Option<PathBuf> {
        self.answer.clone()
    }
}

fn told(available: bool, answer: Option<&str>) -> Arc<dyn FolderPicker> {
    Arc::new(Told {
        available,
        answer: answer.map(PathBuf::from),
    })
}

#[test]
fn the_info_says_whether_a_dialog_exists() {
    for available in [true, false] {
        let info = FolderDialogInfo {
            picker: told(available, None),
        };
        let out = info.run(&principal::resolve(), ()).unwrap();
        assert_eq!(out.available, available);
    }
}

#[test]
fn the_chosen_folder_comes_back_as_text() {
    let pick = PickFolder {
        picker: told(true, Some("/work/shop")),
    };
    let out = pick.run(&principal::resolve(), ()).unwrap();
    assert_eq!(out.path.as_deref(), Some("/work/shop"));
}

#[test]
fn a_cancelled_dialog_is_no_error() {
    let pick = PickFolder {
        picker: told(true, None),
    };
    let out = pick.run(&principal::resolve(), ()).unwrap();
    assert_eq!(out.path, None);
}

#[test]
fn the_registry_knows_both_commands_and_no_dialog_opens_without_one() {
    // The browser sends no body, which the server reads as `null` (not `{}`); a body of `{}` was
    // refused for a long time and the button never showed (found in the first test of the app).
    let fixture = Fixture::installed(None);
    let info = fixture.call("projects.folder_dialog", Value::Null).unwrap();
    // The system dialog exists only on Windows and macOS; the test never opens it.
    assert_eq!(info["available"], cfg!(any(windows, target_os = "macos")));
    if !cfg!(any(windows, target_os = "macos")) {
        let picked = fixture.call("projects.pick_folder", Value::Null).unwrap();
        assert!(picked["path"].is_null());
    }
}
