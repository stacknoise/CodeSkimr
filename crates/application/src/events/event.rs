//! The events of the application layer.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, Level2Stage, ProjectId, Snapshot, SnapshotId};

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
    /// The alerts of a project changed (ADR 0068).
    AlertsChanged(AlertsChanged),
    /// A level 2 run moved on to a language or a stage (ADR 0072).
    Level2Progressed(Level2Progressed),
    /// A level 2 run ended, in any way (ADR 0072).
    Level2Finished(Level2Finished),
}

/// A level 2 run works on a language. It names ids and labels only, never a path or a tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2Progressed {
    /// Project whose run it is.
    pub project: ProjectId,
    /// The language the run works on.
    pub language: LanguageId,
    /// What it does for it.
    pub stage: Level2Stage,
}

/// A level 2 run ended: checked, failed, cancelled or skipped as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level2Finished {
    /// Project whose run it was.
    pub project: ProjectId,
    /// The snapshot the run worked on.
    pub snapshot: SnapshotId,
    /// Languages an indexer checked.
    pub checked: u32,
    /// Languages that were skipped, with a reason in the report.
    pub skipped: u32,
    /// Languages whose run failed.
    pub failed: u32,
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

/// The alert list of a project changed. It names no symbol and no text: events are not filtered
/// per user, so a subscriber asks for the alerts it may see (ADR 0068, decision 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlertsChanged {
    /// The project.
    pub project: ProjectId,
    /// How many alerts of the project are open now.
    pub open: u32,
}
