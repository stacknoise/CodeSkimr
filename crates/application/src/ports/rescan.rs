//! Starting an incremental scan from a driving adapter (implemented by the `Scanner`).

use codeskimr_domain::{Principal, ProjectId, Snapshot};

use crate::error::CodedError;

/// Scans a registered project again, taking over what did not change.
pub trait ProjectScanner: Send + Sync {
    /// Runs an incremental scan of `project` for `principal`, looking for changes only in the
    /// files and folders `paths` name (all of the project if there are none), and returns the
    /// snapshot it wrote. A file the paths do not name stays as the previous snapshot has it, also
    /// if it changed or is gone since. A path is only compared with the files found below the
    /// project root, never opened (blueprint S-05), so a path that leaves the project names nothing.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not scan, otherwise the code of the failed stage;
    /// a cancelled scan is `PRJ-303`.
    fn rescan_paths(
        &self,
        principal: &Principal,
        project: ProjectId,
        paths: &[String],
    ) -> Result<Snapshot, CodedError>;

    /// Runs an incremental scan of the whole `project`.
    ///
    /// # Errors
    /// As [`ProjectScanner::rescan_paths`].
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError> {
        self.rescan_paths(principal, project, &[])
    }
}
