//! The folder dialog of the operating system (ADR 0075), opened on the machine the app runs on.
//!
//! The UI offers "Browse" next to the folder field only if the dialog exists here: Windows and
//! macOS have one without extra system libraries; Linux has no common one (like the tray, ADR
//! 0067) and keeps the typed path. The dialog only returns the chosen path; nothing is read in
//! the folder before the user registers the project.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Something that can ask the user for a folder.
pub trait FolderPicker: Send + Sync {
    /// Whether a dialog can be shown on this machine.
    fn available(&self) -> bool;

    /// Shows the dialog and waits for the answer: the chosen folder, `None` if the user cancelled,
    /// no dialog exists here, or another one is open already.
    fn pick(&self) -> Option<PathBuf>;
}

/// The dialog of the operating system, one at a time.
#[derive(Debug, Default)]
pub struct SystemPicker {
    open: AtomicBool,
}

impl FolderPicker for SystemPicker {
    fn available(&self) -> bool {
        cfg!(any(windows, target_os = "macos"))
    }

    fn pick(&self) -> Option<PathBuf> {
        if !self.available() || self.open.swap(true, Ordering::AcqRel) {
            return None;
        }
        let chosen = show();
        self.open.store(false, Ordering::Release);
        chosen
    }
}

#[cfg(any(windows, target_os = "macos"))]
fn show() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn show() -> Option<PathBuf> {
    None
}
