//! Who may run indexers for level 2, and when (ADR 0033, decisions 9 and 10).
//!
//! An indexer runs the build tools of the project, which means project code. So it runs only if
//! the local owner enabled level 2 for the project in the **user configuration or the user's own
//! project record** and confirmed it once. The `level2` table of the project's `.codeskimr.toml`
//! never counts (see `config`). Confirming and starting need their own permissions; MCP, REST
//! tokens, watch mode and CI have no way to confirm, and a missing confirmation stops them like
//! everybody else.

mod pass;
mod report;

use std::path::Path;
use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, Permission, Principal, Project, ProjectId, ProjectSource};

use crate::config::Level2;
use crate::error::CodedError;
pub use crate::ports::{ConsentRecord, ConsentStore, RunPermit};
pub use pass::Level2Pass;
pub use report::{LanguageReport, Level2Outcome, Level2Result, Skipped};

/// Enables level 2 for `project` and records the confirmation. The caller (an interactive screen
/// or command of the local owner) must have asked the user; nothing else calls this.
///
/// # Errors
/// `SEC-403` if the principal lacks [`Permission::ConfirmIndexer`], otherwise the failure of the
/// store.
pub fn confirm(
    principal: &Principal,
    store: &dyn ConsentStore,
    project: ProjectId,
) -> Result<(), CodedError> {
    require(*principal, Permission::ConfirmIndexer)?;
    store.store_consent(
        project,
        ConsentRecord {
            enabled: true,
            confirmed: true,
        },
    )
}

/// Withdraws enabling and confirmation of `project`.
///
/// # Errors
/// `SEC-403` if the principal lacks [`Permission::ConfirmIndexer`], otherwise the failure of the
/// store.
pub fn revoke(
    principal: &Principal,
    store: &dyn ConsentStore,
    project: ProjectId,
) -> Result<(), CodedError> {
    require(*principal, Permission::ConfirmIndexer)?;
    store.store_consent(project, ConsentRecord::default())
}

/// The folder an indexer may run in: for a project that was registered from a folder, the folder
/// of the project record; for a Git source, the clone folder the source adapter returned. Nothing
/// else is accepted, so a caller cannot point an indexer at another place.
fn run_folder<'a>(project: &'a Project, clone: &'a FolderPath) -> &'a Path {
    match &project.source {
        ProjectSource::Folder(folder) => Path::new(folder.as_str()),
        ProjectSource::Git { .. } => Path::new(clone.as_str()),
    }
}

/// Lets a run of indexers for `project` pass and returns the proof the runner needs: the principal
/// may start them, level 2 is enabled in the user configuration or the project record, and the
/// project record holds a confirmation for the source the project has now. The indexer runs in
/// the folder of the project record (a folder source) or in `clone`, the folder the source adapter
/// made of a Git source.
///
/// `settings` must come from a configuration whose project layer was resolved by
/// `Config::resolve`, which drops the `level2` table of the project file.
///
/// # Errors
/// `SEC-403` if the principal lacks [`Permission::RunIndexer`], `SEC-301` if level 2 is not
/// enabled or not confirmed, otherwise the failure of the store.
pub fn authorize_run(
    principal: &Principal,
    settings: &Level2,
    store: &Arc<dyn ConsentStore>,
    project: &Project,
    clone: &FolderPath,
) -> Result<RunPermit, CodedError> {
    require(*principal, Permission::RunIndexer)?;
    let permit = RunPermit::new(
        Arc::clone(store),
        project.id,
        run_folder(project, clone).to_path_buf(),
        settings.enabled,
    );
    permit.recheck()?;
    Ok(permit)
}

/// A restricted principal gets `SEC-403`, which says what was refused (unlike `AUTH-401`).
fn require(principal: Principal, permission: Permission) -> Result<(), CodedError> {
    if principal.holds(permission) {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::IndexerNotPermitted))
    }
}
