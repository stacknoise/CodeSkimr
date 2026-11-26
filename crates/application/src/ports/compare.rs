//! The reads that the comparison of two scans needs (ADR 0097; adapter: `codeskimr_storage`).

use codeskimr_domain::{SnapshotId, SymbolFact};

use crate::error::CodedError;

/// What the comparison reads of a retained snapshot, not only of the newest one. Each call is one
/// read transaction and holds no lock a scan waits for.
pub trait ComparisonSource: Send + Sync {
    /// The symbols of `snapshot` that take part in a comparison (`SymbolFact::takes_part`), of
    /// origin level 1 only (what only level 2 knows is not compared), with the file that declares each, ordered by id. Empty for a snapshot that
    /// does not exist or that retention has removed.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn comparison_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFact>, CodedError>;

    /// The naming rules `snapshot` was scanned with, `None` for a snapshot that has no record
    /// (written before ADR 0036) or that does not exist.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn snapshot_naming_rules(&self, snapshot: SnapshotId) -> Result<Option<String>, CodedError>;
}
