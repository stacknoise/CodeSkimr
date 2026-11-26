//! Opens a file with the program that the operating system names for it (ADR 0111).
//!
//! Only the application hands over a path, and only one that it built itself from a registered
//! project folder; the opener never sees text from a request.

use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

use crate::browser;

/// Something that can open a file for the user on this machine.
pub trait FileOpener: Send + Sync {
    /// Asks the system to open `file`.
    ///
    /// # Errors
    /// `APP-605` if no program could be started.
    fn open(&self, file: &Path) -> Result<(), CodedError>;
}

/// The opener of the operating system, the one that also starts the browser (ADR 0028).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemOpener;

impl FileOpener for SystemOpener {
    fn open(&self, file: &Path) -> Result<(), CodedError> {
        browser::open(file).map_err(|_| CodedError::new(ErrorCode::ProjectFileOpenFailed))
    }
}
