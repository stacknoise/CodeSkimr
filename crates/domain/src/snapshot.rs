//! One scan state of a project.

use crate::consts::{
    ORIGIN_BROWSER, ORIGIN_MCP, ORIGIN_TERMINAL, ORIGIN_WATCH, SCAN_FULL, SCAN_INCREMENTAL,
};
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

labeled_enum!(
    /// Which channel started a scan. The server decides it from the channel that received the
    /// request, never from a field of the request (ADR 0072, decision 2).
    ScanOrigin {
        /// `codeskimr scan` at the terminal of the owner.
        Terminal => ORIGIN_TERMINAL,
        /// A command of the HTTP layer of `codeskimr serve`, the page in the browser.
        Browser => ORIGIN_BROWSER,
        /// A later scan of `codeskimr scan --watch`.
        Watch => ORIGIN_WATCH,
        /// A rescan through MCP.
        Mcp => ORIGIN_MCP,
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
