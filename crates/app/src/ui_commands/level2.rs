//! The command that tells the screen where level 2 stands for a project, and the wiring of the
//! desk behind it and behind the route that opens the confirmation window (ADR 0071).
//!
//! The browser never gives the consent. This command only reads: the state of the switch in the
//! user configuration, whether the project is confirmed for its current source, and how the
//! confirmation is reached on this system.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::level2::{ConfirmationDesk, DeskParts, Launch, Status, status};
use codeskimr_application::ports::{ConfirmationWindow, Level2Switch};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::{Permission, ProjectId};
use codeskimr_storage::SqliteStore;
use serde::Deserialize;

use super::consts::{LEVEL2_STATUS, LEVEL2_STATUS_DESCRIPTION, LEVEL2_STATUS_TITLE};
use crate::config_files;
use crate::confirmation_window::ConsoleWindow;
use crate::locations::Locations;

/// The input of `level2.status` and `level2.revoke`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level2Input {
    /// Id of the project. `project_id` is accepted as well, the name the route of the confirmation
    /// window uses.
    #[serde(alias = "project_id")]
    pub project: u64,
}

/// Reports where level 2 stands for a project: `state`, `confirmed` and `launch`, no path and no
/// text of the project.
pub struct Level2StatusCommand {
    pub(super) desk: Arc<ConfirmationDesk>,
}

impl Command for Level2StatusCommand {
    type Input = Level2Input;
    type Output = Status;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LEVEL2_STATUS),
        title_key: LEVEL2_STATUS_TITLE,
        description_key: LEVEL2_STATUS_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<Status, CodedError> {
        access::require(request.principal, Permission::Read)?;
        status(&self.desk, ProjectId::new(request.input.project))
    }
}

/// The switch `[level2] enabled`, read from the user configuration files again at every call, so
/// that a switch the owner just edited counts without restarting the server. The project file
/// never counts. The program never writes it (ADR 0071, decision 8).
pub(super) struct UserSwitch {
    pub(super) global: GlobalOptions,
    pub(super) locations: Locations,
}

impl Level2Switch for UserSwitch {
    fn enabled(&self) -> Result<bool, CodedError> {
        Ok(config_files::load(&self.locations, &self.global, None)?
            .level2
            .enabled)
    }
}

/// How the desk reaches the window: the adapter and what this system can do.
pub(crate) struct Window {
    pub(crate) port: Arc<dyn ConfirmationWindow>,
    pub(crate) launch: Launch,
}

impl Window {
    /// The adapter of this program: a console window on Windows, Terminal on macOS, the command to
    /// copy elsewhere and in a remote session.
    pub(crate) fn of_this_system(global: &GlobalOptions) -> Self {
        let adapter = ConsoleWindow::new(global);
        let launch = adapter.launch(&|name| std::env::var_os(name).is_some());
        Self {
            port: Arc::new(adapter),
            launch,
        }
    }
}

/// The desk over the project store, the switch of the user configuration and `window`. The one
/// desk serves the command and the route, so there is one pause for both.
pub(crate) fn confirmation_desk(
    catalog: &Arc<SqliteStore>,
    config: &Config,
    (global, locations): (&GlobalOptions, &Locations),
    window: Window,
) -> Arc<ConfirmationDesk> {
    Arc::new(ConfirmationDesk::new(DeskParts {
        window: window.port,
        switch: Arc::new(UserSwitch {
            global: global.clone(),
            locations: locations.clone(),
        }),
        projects: catalog.clone(),
        consents: catalog.clone(),
        launch: window.launch,
        locked: false,
        cooldown: std::time::Duration::from_secs(u64::from(
            config.level2.confirm_window_cooldown_seconds,
        )),
    }))
}
