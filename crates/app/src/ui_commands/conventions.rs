//! The command behind the tab "Conventions" of the screen Rules: the convention check of the
//! console (`codeskimr check`) for a project that is a folder on this machine (ADR 0116).
//!
//! The check reads the files as they are now, so it needs the right to read source, like the
//! MCP tool `check_conventions` (ADR 0035). It stores nothing and takes no path from the caller:
//! the folder is the one the project was registered with.

use std::sync::Arc;

use codeskimr_application::{
    access,
    error::CodedError,
    ports::ProjectRepository,
    registry::{Command, CommandDescriptor, CommandId, Request},
};
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, ProjectSource};
use codeskimr_mcp::conventions_answer;
use serde::Deserialize;
use serde_json::Value;

use super::consts::{CONVENTIONS_CHECK, CONVENTIONS_CHECK_DESCRIPTION, CONVENTIONS_CHECK_TITLE};
use crate::check::check_folder;
use crate::locations::Locations;

/// The input of `conventions.check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    /// Id of the project.
    pub project: u64,
}

/// What the check needs besides the project: the settings of the program.
pub struct ConventionsPorts {
    /// The registered projects.
    pub(super) projects: Arc<dyn ProjectRepository>,
    /// The options of the command line, which the configuration is resolved with.
    pub(super) global: GlobalOptions,
    /// Where the language packs and the settings are.
    pub(super) locations: Locations,
}

/// Checks the files of a folder project against the code rules.
pub struct ConventionsCheck {
    pub(super) ports: Arc<ConventionsPorts>,
}

impl Command for ConventionsCheck {
    type Input = CheckInput;
    type Output = Value;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CONVENTIONS_CHECK),
        title_key: CONVENTIONS_CHECK_TITLE,
        description_key: CONVENTIONS_CHECK_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    /// # Errors
    /// - `AUTH-401` without the right to read source.
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (its files live in the
    ///   cache of the scan, not in a folder of the person).
    /// - The coded failure of the configuration, the language packs or the file discovery.
    fn execute(&self, request: Request<'_, Self>) -> Result<Value, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let project = self
            .ports
            .projects
            .project(ProjectId::new(request.input.project))?;
        let ProjectSource::Folder(root) = &project.source else {
            return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
        };
        let report = check_folder(root, &[], (&self.ports.global, &self.ports.locations))?;
        conventions_answer(report).map_err(CodedError::caused_by)
    }
}
