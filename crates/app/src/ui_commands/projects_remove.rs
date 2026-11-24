//! The command that removes a project from the app again.
//!
//! It deletes what the app stored about the project: scans, layers, the level 2 confirmation, tool
//! reports and annotations. It never touches the files of a folder project and never the stored
//! logins, which other projects may use. The clone of a Git project lies in the app's own data
//! folder, named by the app after the address and branch, so it is removed with the project.

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ProjectRemoval, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId, ProjectSource};
use codeskimr_source::GitCache;
use serde::{Deserialize, Serialize};

use super::Context;
use super::consts::{PROJECTS_REMOVE, REMOVE_DESCRIPTION, REMOVE_TITLE};

/// The input of `projects.remove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct RemoveInput {
    /// Id of the project.
    pub project: u64,
}

/// What `projects.remove` reports back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Removed {
    /// Id of the project that is gone.
    pub project: u64,
}

/// Removes a project and everything stored about it.
pub struct RemoveProject {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) removal: Arc<dyn ProjectRemoval>,
    pub(super) context: Arc<Context>,
}

impl Command for RemoveProject {
    type Input = RemoveInput;
    type Output = Removed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_REMOVE),
        title_key: REMOVE_TITLE,
        description_key: REMOVE_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects, `PRJ-201` for an unknown project,
    /// `PRJ-302` while a scan of the project runs (a level 2 run is part of a scan), otherwise the
    /// failure of the store; nothing is deleted then.
    fn execute(&self, request: Request<'_, Self>) -> Result<Removed, CodedError> {
        let id = ProjectId::new(request.input.project);
        let project = self.projects.project(id)?;
        // The scan lock is held until the project is gone, so no scan can start in between.
        let _scanning = self.context.running.start(id)?;
        let _removing = self.context.running.mark_removing(id);
        self.removal.remove_project(id)?;
        self.forget_clone(id, &project.source);
        Ok(Removed { project: id.get() })
    }
}

impl RemoveProject {
    /// Deletes the clone of a Git project. The project is gone already, so a clone that cannot
    /// be deleted is logged and left behind, not reported as a failure of the removal.
    fn forget_clone(&self, id: ProjectId, source: &ProjectSource) {
        let ProjectSource::Git { url, branch, .. } = source else {
            return;
        };
        let cache = GitCache::new(self.context.locations.git_cache());
        if let Err(error) = cache.remove_checkout(url, branch) {
            tracing::warn!(project = id.get(), kind = ?error.kind());
        }
    }
}
