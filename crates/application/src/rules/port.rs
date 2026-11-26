//! The question the doors ask: what do the rules of a project say about its newest scan.

use codeskimr_domain::{Project, SnapshotId};

use super::RulesReport;
use crate::error::CodedError;

/// Judges the architecture rules of a project against its held state (ADR 0104). The
/// implementation reads the project file and the held state, which stand next to the code and
/// not in the store.
pub trait RulesChecks: Send + Sync {
    /// The rules of `project` with their violations in `snapshot`.
    ///
    /// # Errors
    /// `CFG-101` / `CFG-102` if the project file or the held state cannot be read or holds an
    /// invalid value, or a coded failure of the `STO` group.
    fn check(&self, project: &Project, snapshot: SnapshotId) -> Result<RulesReport, CodedError>;
}
