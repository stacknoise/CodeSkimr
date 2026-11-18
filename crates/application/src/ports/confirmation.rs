//! The console window of the level 2 confirmation and the master switch it depends on
//! (ADR 0071, decisions 5, 7 and 8).

use codeskimr_domain::ProjectId;

use crate::error::CodedError;

/// What the system did when it was asked to open the confirmation window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowOutcome {
    /// A new console window with the confirmation is open now.
    Opened,
    /// The window of an earlier request for this project is still open; nothing new was started.
    AlreadyOpen,
    /// This system has no way to open the window; the screen shows the command to copy instead.
    Unavailable,
}

/// Opens the console window in which the owner confirms level 2 for a project. The adapter lives
/// in the composition root, next to the browser start and the tray, because it starts a process.
/// The window runs the confirmation command of the program itself, which asks the owner to type
/// the name of the project; nothing the caller passes can carry a consent.
pub trait ConfirmationWindow: Send + Sync {
    /// Opens the window for the confirmation of `project`. The adapter writes the numeric id into
    /// a fixed argument list; it never passes a name or text of the request on.
    ///
    /// # Errors
    /// `APP-604` if the window could not be started.
    fn open(&self, project: ProjectId) -> Result<WindowOutcome, CodedError>;
}

/// The master switch `[level2] enabled` of the user configuration. The program never writes it
/// (ADR 0071, decision 8); the adapter reads the configuration files again on every call, so that
/// a switch the owner just edited counts without restarting the server.
pub trait Level2Switch: Send + Sync {
    /// Whether level 2 is enabled in the user configuration.
    ///
    /// # Errors
    /// A coded failure of the configuration (`CFG-102`, `CFG-103`, `CFG-101`).
    fn enabled(&self) -> Result<bool, CodedError>;
}
