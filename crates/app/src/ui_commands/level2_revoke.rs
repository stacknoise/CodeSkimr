//! The command that takes the level 2 confirmation of a project back (ADR 0071, decision 2).
//!
//! Revoking only removes a right, so it needs no terminal and no typed name. It is the one caller
//! of `level2::revoke` outside the command line, and `tests/consent_origin.rs` allows exactly this
//! file for it. It does not stop an indexer that is running; the next start finds no consent.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::level2::{self, ConfirmationDesk, Status};
use codeskimr_application::ports::{ConsentStore, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId};

use super::consts::{LEVEL2_REVOKE, LEVEL2_REVOKE_DESCRIPTION, LEVEL2_REVOKE_TITLE};
use super::level2::Level2Input;

/// Forbids the indexers for a project again and answers where it stands then.
pub struct Level2Revoke {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) store: Arc<dyn ConsentStore>,
    pub(super) desk: Arc<ConfirmationDesk>,
}

impl Command for Level2Revoke {
    type Input = Level2Input;
    type Output = Status;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LEVEL2_REVOKE),
        title_key: LEVEL2_REVOKE_TITLE,
        description_key: LEVEL2_REVOKE_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RevokeIndexer,
    };

    /// # Errors
    /// `AUTH-401` without the right to revoke, `SEC-403` on a team server, `PRJ-201` for an
    /// unknown project, otherwise the failure of the store.
    fn execute(&self, request: Request<'_, Self>) -> Result<Status, CodedError> {
        access::require(request.principal, Permission::RevokeIndexer)?;
        if self.desk.is_locked() {
            return Err(CodedError::new(ErrorCode::IndexerNotPermitted));
        }
        let id = ProjectId::new(request.input.project);
        self.projects.project(id)?;
        level2::revoke(request.principal, self.store.as_ref(), id)?;
        level2::status(&self.desk, id)
    }
}
