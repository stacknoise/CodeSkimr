//! Who may run indexers for level 2, and when (ADR 0033, decisions 9 and 10).
//!
//! An indexer runs the build tools of the project, which means project code. So it runs only if
//! the local owner enabled level 2 for the project in the **user configuration or the user's own
//! project record** and confirmed it once. The `level2` table of the project's `.codeskimr.toml`
//! never counts (see `config`). Confirming and starting need their own permissions; MCP, REST
//! tokens, watch mode and CI have no way to confirm, and a missing confirmation stops them like
//! everybody else.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, ProjectId};

use crate::config::Level2;
use crate::error::CodedError;
pub use crate::ports::{ConsentRecord, ConsentStore, RunPermit};

/// A consent store that lives in memory only: for tests and for callers without a database.
#[derive(Debug, Default)]
pub struct InMemoryConsent(Mutex<ConsentRecord>);

impl ConsentStore for InMemoryConsent {
    fn consent(&self, _project: ProjectId) -> Result<ConsentRecord, CodedError> {
        Ok(*self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn store_consent(&self, _project: ProjectId, record: ConsentRecord) -> Result<(), CodedError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = record;
        Ok(())
    }
}

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

/// Lets a run of indexers for `project`, whose files are in `project_root`, pass and returns the proof the runner needs: the principal
/// may start them, level 2 is enabled in the user configuration or the project record, and the
/// project record holds a confirmation for the source the project has now.
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
    project: ProjectId,
    project_root: &Path,
) -> Result<RunPermit, CodedError> {
    require(*principal, Permission::RunIndexer)?;
    let permit = RunPermit::new(
        Arc::clone(store),
        project,
        project_root.to_path_buf(),
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
