//! Who may run indexers for level 2, and when (ADR 0033, decisions 9 and 10).
//!
//! An indexer runs the build tools of the project, which means project code. So it runs only if
//! the local owner enabled level 2 for the project in the **user configuration or the user's own
//! project record** and confirmed it once. The `level2` table of the project's `.codeskimr.toml`
//! never counts (see `config`). Confirming and starting need their own permissions; MCP, REST
//! tokens, watch mode and CI have no way to confirm, and a missing confirmation stops them like
//! everybody else.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, ProjectId};

use crate::config::Level2;
use crate::error::CodedError;

/// What the user's own project record says about level 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConsentRecord {
    /// Level 2 is enabled for this project.
    pub enabled: bool,
    /// The user confirmed once that indexers may run the build tools of this project.
    pub confirmed: bool,
}

/// The consent of the user, kept with the project record (adapter: `codeskimr_storage`).
pub trait ConsentStore: Send + Sync {
    /// The record of `project`; without one nothing is enabled or confirmed.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn consent(&self, project: ProjectId) -> Result<ConsentRecord, CodedError>;

    /// Replaces the record of `project`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn store_consent(&self, project: ProjectId, record: ConsentRecord) -> Result<(), CodedError>;
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

/// Lets a run of indexers for `project` pass: the principal may start them, level 2 is enabled in
/// the user configuration or the project record, and the project record holds the confirmation.
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
    store: &dyn ConsentStore,
    project: ProjectId,
) -> Result<(), CodedError> {
    require(*principal, Permission::RunIndexer)?;
    let record = store.consent(project)?;
    if (settings.enabled || record.enabled) && record.confirmed {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::IndexerNotConfirmed))
    }
}

/// A restricted principal gets `SEC-403`, which says what was refused (unlike `AUTH-401`).
fn require(principal: Principal, permission: Permission) -> Result<(), CodedError> {
    if principal.holds(permission) {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::IndexerNotPermitted))
    }
}
