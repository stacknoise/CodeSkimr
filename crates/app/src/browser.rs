//! Starts the default browser for a file (ADR 0028).
//!
//! The system's own opener is asked to open the launch file, so the file extension decides which
//! application takes it and nothing is guessed here. Only the path of the file is handed over; the
//! launch code is inside the file and never on a command line.

use std::path::Path;
use std::process::{Command, Stdio};

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

use crate::consts::{OPENER_MACOS, OPENER_UNIX, OPENER_WINDOWS, OS_MACOS};

/// The program that opens a file with its default application on this system.
#[must_use]
pub fn opener() -> &'static str {
    if cfg!(windows) {
        OPENER_WINDOWS
    } else if std::env::consts::OS == OS_MACOS {
        OPENER_MACOS
    } else {
        OPENER_UNIX
    }
}

/// Asks the system to open `file`. The opener runs on its own; its exit status is not read,
/// because several openers report failure although the file was opened.
///
/// # Errors
/// `APP-601` if the opener cannot be started.
pub fn open(file: &Path) -> Result<(), CodedError> {
    Command::new(opener())
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| CodedError::new(ErrorCode::BrowserStartFailed))
}
