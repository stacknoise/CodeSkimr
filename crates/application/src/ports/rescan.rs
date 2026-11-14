//! Starting an incremental scan from a driving adapter (implemented by the `Scanner`).

use codeskimr_domain::{Principal, ProjectId, Snapshot};

use crate::error::CodedError;

/// Scans a registered project again, taking over what did not change.
pub trait ProjectScanner: Send + Sync {
    /// Runs an incremental scan of `project` for `principal` and returns the snapshot it wrote.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not scan, otherwise the code of the failed stage;
    /// a cancelled scan is `PRJ-303`.
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError>;
}
