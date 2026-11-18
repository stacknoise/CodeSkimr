//! The state of level 2 per project and the console window of the confirmation (ADR 0071).
//!
//! The browser never gives the consent: this module only reports the state, and asks the system
//! to open the window in which the owner types the name of the project (`codeskimr level2
//! confirm`). Whatever the window does, the page learns the result by asking for the state again.

use std::sync::Arc;
use std::time::{Duration, Instant};

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, ProjectId};
use serde::Serialize;

use super::disguise::is_disguised;
use super::pause::Pause;
use crate::access;
use crate::consts::{COPY_COMMAND_PROGRAM, COPY_COMMAND_SEPARATOR, COPY_COMMAND_WORDS};
use crate::error::CodedError;
use crate::ports::{
    ConfirmationWindow, ConsentStore, Level2Switch, ProjectRepository, WindowOutcome,
};

/// Where the project stands, as the screen shows it (ADR 0071, decision 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// The switch is on and the project is confirmed for its current source.
    On,
    /// The switch is on, the project is not confirmed (or its source changed since).
    Off,
    /// `[level2] enabled` is missing or false in the user configuration.
    NotEnabled,
    /// A team server: nothing can be switched.
    Locked,
}

/// How the screen gets the owner to the confirmation (ADR 0071, decisions 5 and 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Launch {
    /// The server can open a console window.
    Window,
    /// The screen shows the command to copy.
    Copy,
}

/// The answer of `level2.status`: no path and no text of the project, and no path of the
/// configuration file (the command needs no token, and a path holds the user name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Status {
    /// The project the answer is for.
    pub project: u64,
    /// Where the project stands.
    pub state: State,
    /// Whether a confirmation exists for the current source, whatever the switch says.
    pub confirmed: bool,
    /// How the confirmation is reached on this system.
    pub launch: Launch,
}

/// What `open_confirmation` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// A window is open (or was, and still is): the screen waits for the confirmation.
    Window,
    /// No window: the screen shows `command` to copy.
    Copy {
        /// `codeskimr level2 confirm <id>`, with the id and never the name of the project.
        command: String,
    },
}

/// Everything the desk is made of; the composition root fills it.
pub struct DeskParts {
    /// Opens the window.
    pub window: Arc<dyn ConfirmationWindow>,
    /// Reads `[level2] enabled` of the user configuration.
    pub switch: Arc<dyn Level2Switch>,
    /// The registered projects.
    pub projects: Arc<dyn ProjectRepository>,
    /// The stored consent, read only.
    pub consents: Arc<dyn ConsentStore>,
    /// What this system can do, decided once by the adapter of the window.
    pub launch: Launch,
    /// A team server: nothing can be switched and no window is opened.
    pub locked: bool,
    /// Pause after a window was opened (`level2.confirm_window_cooldown_seconds`).
    pub cooldown: Duration,
}

/// The state of level 2 and the window of the confirmation, with the pause that belongs to it.
pub struct ConfirmationDesk {
    parts: DeskParts,
    pause: Pause,
}

impl ConfirmationDesk {
    /// A desk over `parts`; the pause starts with the first window.
    #[must_use]
    pub const fn new(parts: DeskParts) -> Self {
        let pause = Pause::new(parts.cooldown);
        Self { parts, pause }
    }

    /// Whether this is a team server.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.parts.locked
    }

    /// How long a request has to wait now before a window may open; zero outside the pause. The
    /// server tells it to the client in `Retry-After`.
    #[must_use]
    pub fn retry_after(&self) -> Duration {
        self.pause.remaining(Instant::now())
    }
}

/// Where `project` stands. The caller has checked the right to read (the registry does, for the
/// command), and the answer holds no path and no text of the project.
///
/// # Errors
/// `PRJ-201` if the project is unknown, otherwise a failure of the storage or of the
/// configuration files.
pub fn status(desk: &ConfirmationDesk, project: ProjectId) -> Result<Status, CodedError> {
    desk.parts.projects.project(project)?;
    let confirmed = desk.parts.consents.consent(project)?.confirmed;
    let state = if desk.parts.locked {
        State::Locked
    } else if !desk.parts.switch.enabled()? {
        State::NotEnabled
    } else if confirmed {
        State::On
    } else {
        State::Off
    };
    Ok(Status {
        project: project.get(),
        state,
        confirmed,
        launch: desk.parts.launch,
    })
}

/// Opens the console window of the confirmation for `project`, or tells the screen to show the
/// command. The caller (the route) has passed the guard and read the body; the order from here is
/// the one of ADR 0071, decision 4: the right, the project, the switch, the pause, the launch.
/// The pause starts when the program tries to start a window, also when that fails, so that a
/// holder of the token cannot start the terminal program as often as it likes. A request refused
/// before the launch starts no pause.
///
/// # Errors
/// `AUTH-401` without [`Permission::OpenConfirmation`]; `SEC-403` on a team server or for a
/// project whose name, location or branch holds control or invisible characters; `PRJ-201` for an
/// unknown project; `CFG-105` if the switch of the user configuration is off; `API-504` inside
/// the pause; `APP-604` if the window could not be started.
pub fn open_confirmation(
    principal: &Principal,
    desk: &ConfirmationDesk,
    project: ProjectId,
) -> Result<Opened, CodedError> {
    access::require(principal, Permission::OpenConfirmation)?;
    if desk.parts.locked {
        return Err(CodedError::new(ErrorCode::IndexerNotPermitted));
    }
    let shown = desk.parts.projects.project(project)?;
    if is_disguised(&shown) {
        return Err(CodedError::new(ErrorCode::IndexerNotPermitted));
    }
    if !desk.parts.switch.enabled()? {
        return Err(CodedError::new(ErrorCode::Level2SwitchOff));
    }
    if desk.parts.launch == Launch::Copy {
        return Ok(copy(project));
    }
    let before = desk
        .pause
        .begin(Instant::now())
        .map_err(|_| CodedError::new(ErrorCode::ConfirmWindowRateLimited))?;
    // A failed start keeps the pause; a request that started nothing gives it back.
    match desk.parts.window.open(project)? {
        WindowOutcome::Opened => Ok(Opened::Window),
        WindowOutcome::AlreadyOpen => {
            desk.pause.undo(before);
            Ok(Opened::Window)
        }
        WindowOutcome::Unavailable => {
            desk.pause.undo(before);
            Ok(copy(project))
        }
    }
}

/// The answer that shows the command: the id, never the name.
fn copy(project: ProjectId) -> Opened {
    let words = [COPY_COMMAND_PROGRAM]
        .into_iter()
        .chain(COPY_COMMAND_WORDS)
        .map(str::to_owned)
        .chain([project.get().to_string()]);
    Opened::Copy {
        command: words
            .collect::<Vec<_>>()
            .join(&COPY_COMMAND_SEPARATOR.to_string()),
    }
}
