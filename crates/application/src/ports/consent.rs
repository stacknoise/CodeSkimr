//! The user's consent to run indexers and the proof that it was checked (ADR 0033).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectId;

use crate::error::CodedError;

/// What the user's own project record says about level 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConsentRecord {
    /// Level 2 is enabled for this project.
    pub enabled: bool,
    /// The user confirmed once that indexers may run the build tools of this project. The store
    /// reports `true` only while the project still has the source (kind, location, branch) it had
    /// when the user confirmed; a project that was pointed at another repository or folder is
    /// unconfirmed again.
    pub confirmed: bool,
}

/// The consent of the user, kept with the project record (adapter: `codeskimr_storage`).
pub trait ConsentStore: Send + Sync {
    /// The record of `project`; without one nothing is enabled or confirmed.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn consent(&self, project: ProjectId) -> Result<ConsentRecord, CodedError>;

    /// Replaces the record of `project` and binds the confirmation to the source the project has
    /// now.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn store_consent(&self, project: ProjectId, record: ConsentRecord) -> Result<(), CodedError>;
}

/// Proof that `level2::authorize_run` let a run pass. Only that function can make one, so a run
/// cannot be started without the check, and the runner reads the consent again with
/// [`RunPermit::recheck`] right before it starts the process.
pub struct RunPermit {
    store: Arc<dyn ConsentStore>,
    project: ProjectId,
    project_root: PathBuf,
    enabled_in_config: bool,
}

impl RunPermit {
    pub(crate) fn new(
        store: Arc<dyn ConsentStore>,
        project: ProjectId,
        project_root: PathBuf,
        enabled_in_config: bool,
    ) -> Self {
        Self {
            store,
            project,
            project_root,
            enabled_in_config,
        }
    }

    /// The project this permit is for.
    #[must_use]
    pub const fn project(&self) -> ProjectId {
        self.project
    }

    /// The folder of the project; the indexer runs in it. It belongs to the permit, so a permit
    /// of one project cannot be used with the folder of another.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Reads the consent again.
    ///
    /// # Errors
    /// `SEC-301` if it was withdrawn or no longer fits the project in the meantime, otherwise the
    /// failure of the store.
    pub fn recheck(&self) -> Result<(), CodedError> {
        let record = self.store.consent(self.project)?;
        if (self.enabled_in_config || record.enabled) && record.confirmed {
            Ok(())
        } else {
            Err(CodedError::new(ErrorCode::IndexerNotConfirmed))
        }
    }
}

impl std::fmt::Debug for RunPermit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(std::any::type_name::<Self>())
            .field(&self.project)
            .finish()
    }
}
