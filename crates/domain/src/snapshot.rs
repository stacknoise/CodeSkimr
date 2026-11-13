//! One scan state of a project.

use crate::consts::{SCAN_FULL, SCAN_INCREMENTAL};
use crate::project::ProjectId;
use crate::vocabulary::{id_type, labeled_enum, text_type};

id_type!(
    /// Identifies a snapshot.
    SnapshotId
);
text_type!(
    /// Git commit hash a snapshot was taken at.
    CommitHash
);

/// Seconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

labeled_enum!(
    /// How much of the project a scan processed.
    ScanMode {
        /// Every file was processed.
        Full => SCAN_FULL,
        /// Only files with a changed hash were processed.
        Incremental => SCAN_INCREMENTAL,
    }
);

/// Totals of a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SnapshotStats {
    /// Number of files.
    pub files: u32,
    /// Number of symbols.
    pub symbols: u32,
    /// Number of references.
    pub references: u32,
}

/// One scan state of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Identity.
    pub id: SnapshotId,
    /// Owning project.
    pub project: ProjectId,
    /// Commit, if the source is a Git repository.
    pub commit: Option<CommitHash>,
    /// Creation time.
    pub created: Timestamp,
    /// Full or incremental scan.
    pub mode: ScanMode,
    /// Totals.
    pub stats: SnapshotStats,
}
