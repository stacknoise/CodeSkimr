//! The commands behind "Keep on purpose" and "Lift": they write an entry into the exception list
//! `[[unused.ignore]]` of the project file of a folder project, or take it out again (ADR 0101).

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::{
    PROJECT_CONFIG_FILE, UnusedIgnore, unused_ignore_in, with_unused_ignore,
};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, ProjectSource, has_wildcard};
use serde::{Deserialize, Serialize};

use super::consts::{
    UNUSED_KEEP, UNUSED_KEEP_DESCRIPTION, UNUSED_KEEP_TITLE, UNUSED_LIFT, UNUSED_LIFT_DESCRIPTION,
    UNUSED_LIFT_TITLE,
};
use super::explorer::ExplorerPorts;
use super::layers::write_project_file;
use crate::project_file::read_project_file;

/// The input of `unused.keep`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KeepInput {
    /// Id of the project.
    pub project: u64,
    /// Qualified name of the symbol that stays.
    pub symbol: String,
    /// Why it stays; mandatory.
    pub reason: String,
}

/// The input of `unused.lift`: the entry as the answer of the statistics shows it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LiftInput {
    /// Id of the project.
    pub project: u64,
    /// Globs on the path of the entry.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Globs on the qualified name of the entry.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// The reason of the entry.
    pub reason: String,
}

/// The answer of both commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Listed {
    /// How many entries the exception list has now.
    pub entries: usize,
}

/// The project file of a folder project.
fn project_file(ports: &ExplorerPorts, project: u64) -> Result<std::path::PathBuf, CodedError> {
    let project = ports.projects.project(ProjectId::new(project))?;
    let ProjectSource::Folder(folder) = &project.source else {
        return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
    };
    Ok(Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE))
}

/// Reads the list of the project file, lets `change` edit it and writes it back.
fn edit(
    ports: &ExplorerPorts,
    project: u64,
    change: impl FnOnce(&mut Vec<UnusedIgnore>) -> Result<(), CodedError>,
) -> Result<Listed, CodedError> {
    let path = project_file(ports, project)?;
    let text = read_project_file(&path)?;
    let mut entries = unused_ignore_in(&text);
    change(&mut entries)?;
    let written = with_unused_ignore(&text, &entries).map_err(CodedError::caused_by)?;
    write_project_file(&path, &written)?;
    Ok(Listed {
        entries: entries.len(),
    })
}

/// Keeps a symbol on purpose.
pub struct UnusedKeep {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for UnusedKeep {
    type Input = KeepInput;
    type Output = Listed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(UNUSED_KEEP),
        title_key: UNUSED_KEEP_TITLE,
        description_key: UNUSED_KEEP_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (it has no project file).
    /// - `CFG-101` if the symbol or the reason is empty, or the name holds a wildcard (an entry
    ///   with it would cover more than this symbol; write such an entry by hand).
    /// - `CFG-102` if the file cannot be read or is not TOML, `CFG-104` if it cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<Listed, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let input = request.input;
        let entry = UnusedIgnore {
            paths: Vec::new(),
            symbols: vec![input.symbol.clone()],
            reason: input.reason.trim().to_owned(),
        };
        if !entry.is_valid() || has_wildcard(&input.symbol) {
            return Err(CodedError::new(ErrorCode::ConfigInvalidValue));
        }
        edit(&self.ports, input.project, |entries| {
            if !entries.contains(&entry) {
                entries.push(entry);
            }
            Ok(())
        })
    }
}

/// Lifts an exception again.
pub struct UnusedLift {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for UnusedLift {
    type Input = LiftInput;
    type Output = Listed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(UNUSED_LIFT),
        title_key: UNUSED_LIFT_TITLE,
        description_key: UNUSED_LIFT_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project.
    /// - `CFG-101` if no such entry is in the project file (it changed since the list was read).
    /// - `CFG-102` if the file cannot be read or is not TOML, `CFG-104` if it cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<Listed, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let input = request.input;
        let gone = UnusedIgnore {
            paths: input.paths.clone(),
            symbols: input.symbols.clone(),
            reason: input.reason.clone(),
        };
        edit(&self.ports, input.project, |entries| {
            let at = entries.iter().position(|entry| *entry == gone);
            let at = at.ok_or_else(|| CodedError::new(ErrorCode::ConfigInvalidValue))?;
            entries.remove(at);
            Ok(())
        })
    }
}
