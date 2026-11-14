//! The events of the application layer.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ProjectId, Snapshot};

use crate::stage::StageKind;

/// Something that happened and that other parts may want to show.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A stage of a scan started or advanced.
    ScanProgressed(ScanProgressed),
    /// A snapshot was written.
    SnapshotCreated(SnapshotCreated),
    /// Files of a project changed on disk.
    FilesChanged(FilesChanged),
    /// A scan ended without a snapshot.
    ScanStopped(ScanStopped),
}

/// Progress inside one stage. `total` is zero while the amount of work is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanProgressed {
    /// Project being scanned.
    pub project: ProjectId,
    /// Stage that reports.
    pub stage: StageKind,
    /// Units finished so far.
    pub done: u32,
    /// Units in total, or zero if unknown.
    pub total: u32,
    /// Milliseconds since the stage started, so that a screen need not time the stream itself.
    pub elapsed_ms: u64,
}

/// A snapshot was persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCreated {
    /// The new snapshot.
    pub snapshot: Snapshot,
}

/// File-system changes were detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilesChanged {
    /// Project whose files changed.
    pub project: ProjectId,
    /// Number of changed files.
    pub files: u32,
    /// The operating system lost events; scan the whole folder instead of trusting the list of
    /// changed files.
    pub rescan: bool,
}

/// Why a scan ended without a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The user cancelled; nothing was written.
    Cancelled,
    /// A stage failed with this code.
    Failed(ErrorCode),
}

/// A scan ended without a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanStopped {
    /// Project that was scanned.
    pub project: ProjectId,
    /// Why it stopped.
    pub reason: StopReason,
}
