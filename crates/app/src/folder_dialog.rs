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

/// The one place for an open dialog; it is free again when this is dropped, also if the dialog
/// panics.
struct Open<'a>(&'a AtomicBool);

impl Drop for Open<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl SystemPicker {
    /// Takes the place for a dialog; `None` if a dialog is open already.
    fn claim(&self) -> Option<Open<'_>> {
        (!self.open.swap(true, Ordering::AcqRel)).then_some(Open(&self.open))
    }
}

impl FolderPicker for SystemPicker {
    fn available(&self) -> bool {
        cfg!(any(windows, target_os = "macos"))
    }

    fn pick(&self) -> Option<PathBuf> {
        if !self.available() {
            return None;
        }
        let _open = self.claim()?;
        show()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_dialog_is_refused_while_one_is_open() {
        let picker = SystemPicker::default();
        let first = picker.claim();
        assert!(first.is_some());
        assert!(picker.claim().is_none(), "the second click gets no dialog");
        drop(first);
        assert!(
            picker.claim().is_some(),
            "the place is free after the dialog closed"
        );
    }
}
