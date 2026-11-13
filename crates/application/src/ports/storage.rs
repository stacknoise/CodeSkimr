//! Persistence (adapter: `codeskimr_storage`).

use codeskimr_domain::{Metric, Project, ProjectId, Reference, Snapshot, SourceFile, Symbol};

use crate::error::CodedError;

/// Everything one scan produced, written as one unit.
///
/// In an incremental scan the lists hold only what was re-analysed; the store carries the data of
/// unchanged files over from the previous snapshot and returns the totals of the whole snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotData {
    /// The snapshot header; `stats` count what is in this value.
    pub snapshot: Snapshot,
    /// Files that were analysed.
    pub files: Vec<SourceFile>,
    /// Symbols of those files.
    pub symbols: Vec<Symbol>,
    /// Resolved references.
    pub references: Vec<Reference>,
    /// Measured values.
    pub metrics: Vec<Metric>,
}

/// Reads registered projects and the state of their last snapshot.
pub trait ProjectRepository: Send + Sync {
    /// The project with this id.
    ///
    /// # Errors
    /// `PRJ-201` if it does not exist.
    fn project(&self, id: ProjectId) -> Result<Project, CodedError>;

    /// The files of the newest snapshot of the project, empty if it has none.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn latest_files(&self, id: ProjectId) -> Result<Vec<SourceFile>, CodedError>;
}

/// Writes snapshots. One snapshot is written atomically or not at all.
pub trait SnapshotStore: Send + Sync {
    /// Writes `data` in one transaction, updates the read models and returns the final snapshot.
    ///
    /// # Errors
    /// A coded failure such as `STO-302` or `STO-501`; nothing is written then.
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError>;
}
