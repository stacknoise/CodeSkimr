//! Opens a file with the program that the operating system names for it (ADR 0111).
//!
//! Only the application hands over a path, and only one that it built itself from a registered
//! project folder; the opener never sees text from a request.

use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

use crate::browser;
use crate::consts::{DESKTOP_SESSION_VARIABLES, REMOTE_SESSION_VARIABLES};

/// Something that can open a file for the user on this machine.
pub trait FileOpener: Send + Sync {
    /// Whether a person sits at this machine: only then a program can open for them. Asked before
    /// anything is created.
    fn available(&self) -> bool;

    /// Asks the system to open `file`.
    ///
    /// # Errors
    /// `APP-605` if no program could be started.
    fn open(&self, file: &Path) -> Result<(), CodedError>;
}

/// The opener of the operating system, the one that also starts the browser (ADR 0028).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemOpener;

/// Whether the session is a desktop session that is not remote, decided from the session
/// variables like the window of the confirmation (ADR 0071).
pub(crate) fn is_local_desktop(has_variable: &dyn Fn(&str) -> bool) -> bool {
    let remote = REMOTE_SESSION_VARIABLES
        .iter()
        .any(|name| has_variable(name));
    let desktop = DESKTOP_SESSION_VARIABLES
        .iter()
        .any(|name| has_variable(name));
    desktop && !remote
}

/// The path in the form the system opener takes (a UNC path loses its verbatim prefix).
fn opener_path(file: &Path) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        std::path::PathBuf::from(crate::source_arg::opener_form(&file.to_string_lossy()))
    }
    #[cfg(not(windows))]
    {
        file.to_path_buf()
    }
}

impl FileOpener for SystemOpener {
    fn available(&self) -> bool {
        is_local_desktop(&|name| std::env::var_os(name).is_some())
    }

    fn open(&self, file: &Path) -> Result<(), CodedError> {
        browser::open(&opener_path(file))
            .map_err(|_| CodedError::new(ErrorCode::ProjectFileOpenFailed))
    }
}

#[cfg(test)]
mod tests {
    use super::is_local_desktop;
    use crate::consts::{DESKTOP_SESSION_VARIABLES, REMOTE_SESSION_VARIABLES};

    #[test]
    fn only_a_desktop_session_that_is_not_remote_counts() {
        let desktop = |name: &str| DESKTOP_SESSION_VARIABLES.contains(&name);
        assert!(is_local_desktop(&desktop));
        assert!(!is_local_desktop(&|_| false));
        for remote in REMOTE_SESSION_VARIABLES {
            assert!(!is_local_desktop(
                &|name| name == remote || DESKTOP_SESSION_VARIABLES.contains(&name)
            ));
        }
    }
}
