//! The commands behind the project name: renaming a project and suggesting the name of a source
//! that is about to be added. The name is only a label; the id and the source never change.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectRepository;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId};
use codeskimr_storage::SqliteStore;
use serde::{Deserialize, Serialize};

use super::consts::{
    PROJECTS_RENAME, PROJECTS_SUGGEST_NAME, RENAME_DESCRIPTION, RENAME_TITLE,
    SUGGEST_NAME_DESCRIPTION, SUGGEST_NAME_TITLE,
};
use crate::source_arg::{SourceError, classify, default_name, display_name};

/// The input of `projects.rename`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RenameInput {
    /// Id of the project.
    pub project: u64,
    /// The new display name; empty goes back to the name that the source suggests.
    pub name: String,
}

/// What `projects.rename` reports back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Renamed {
    /// Id of the project.
    pub project: u64,
    /// The name it has now.
    pub name: String,
}

/// Gives a project another display name.
pub struct RenameProject {
    pub(super) store: Arc<SqliteStore>,
}

impl Command for RenameProject {
    type Input = RenameInput;
    type Output = Renamed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_RENAME),
        title_key: RENAME_TITLE,
        description_key: RENAME_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects, `PRJ-201` for an unknown project,
    /// `PRJ-102` for a name that is too long or has control characters, otherwise the failure of
    /// the store.
    fn execute(&self, request: Request<'_, Self>) -> Result<Renamed, CodedError> {
        let id = ProjectId::new(request.input.project);
        let wanted = request.input.name.trim();
        let name = if wanted.is_empty() {
            let project = self.store.project(id)?;
            default_name(&project.source).ok_or(SourceError::NameInvalid)?
        } else {
            display_name(wanted)?
        };
        let project = self.store.rename_project(id, name)?;
        Ok(Renamed {
            project: id.get(),
            name: project.name.to_string(),
        })
    }
}

/// The input of `projects.suggest_name`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SuggestInput {
    /// The folder or Git address that the user typed.
    pub source: String,
}

/// What `projects.suggest_name` reports back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggested {
    /// The name the project would get; `None` if the text is no source (yet).
    pub name: Option<String>,
}

/// Tells the name that a source would get, so the new-project form can show it before the project
/// is added.
pub struct SuggestName;

impl Command for SuggestName {
    type Input = SuggestInput;
    type Output = Suggested;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_SUGGEST_NAME),
        title_key: SUGGEST_NAME_TITLE,
        description_key: SUGGEST_NAME_DESCRIPTION,
        shortcut: None,
        // Not "read only": it looks the folder up on disk, so it needs the right to register.
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects: the folder is looked up on disk (the path
    /// is made canonical), which a reader may not make the machine do. Text that is no source is
    /// not an error.
    fn execute(&self, request: Request<'_, Self>) -> Result<Suggested, CodedError> {
        access::require(request.principal, Permission::RegisterProject)?;
        let name = classify(request.input.source.trim(), None)
            .ok()
            .map(|spec| spec.name.to_string());
        Ok(Suggested { name })
    }
}
