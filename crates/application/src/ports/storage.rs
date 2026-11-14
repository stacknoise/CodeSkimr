//! Persistence (adapter: `codeskimr_storage`).

use codeskimr_domain::{
    FileId, Metric, Project, ProjectId, Reference, RelativePath, Snapshot, SnapshotId, SourceFile,
    Symbol,
};

use super::SymbolFingerprints;
use crate::error::CodedError;

/// Everything one scan produced, written as one unit.
///
/// In an incremental scan the lists hold only what is new: the files that were analysed again,
/// their symbols, the references that were resolved again (also those that start in an unchanged
/// file but pointed into a changed or removed one) and the metrics that were computed again. The
/// store carries everything else over from the previous snapshot, drops the `removed` files with
/// their symbols, and returns the totals of the whole snapshot. A metric of this value replaces a
/// carried metric of the same symbol and kind.
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
    /// Syntax fingerprints of the callables of the analysed files. The store carries those of
    /// the other files over, like their symbols.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// Paths of files that were in the previous snapshot and no longer exist; empty in a full scan.
    pub removed: Vec<RelativePath>,
    /// The naming rules of the language packs of this scan (ADR 0036); an incremental scan keeps
    /// the files of the previous snapshot only if its rules are these.
    pub naming_rules: String,
}

/// Symbols and references of the newest snapshot of a project, the starting point of an
/// incremental scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreviousIndex {
    /// All symbols.
    pub symbols: Vec<Symbol>,
    /// All references.
    pub references: Vec<Reference>,
}

/// Level 2 references that replace the level 1 references of some files of a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceReplacement {
    /// The snapshot the references belong to.
    pub snapshot: SnapshotId,
    /// The files whose references are replaced; a file that is not here keeps its references.
    pub files: Vec<FileId>,
    /// The new references, all in those files, with ids no stored row has.
    pub references: Vec<Reference>,
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

    /// The symbols and references of the newest snapshot of the project, empty if it has none.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn latest_index(&self, id: ProjectId) -> Result<PreviousIndex, CodedError>;

    /// The naming rules the newest snapshot of the project was scanned with, `None` if it has no
    /// snapshot or none from after the record existed (ADR 0036). The default knows no record.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn latest_naming_rules(&self, id: ProjectId) -> Result<Option<String>, CodedError> {
        let _ = id;
        Ok(None)
    }
}

/// Writes snapshots. One snapshot is written atomically or not at all.
pub trait SnapshotStore: Send + Sync {
    /// Writes `data` in one transaction, updates the read models and returns the final snapshot.
    ///
    /// # Errors
    /// A coded failure such as `STO-302` or `STO-501`; nothing is written then.
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError>;

    /// Replaces the references of the files of `replacement` in one transaction, updates the
    /// totals and the read models and returns the snapshot. Rows are shared between snapshots
    /// that carry a file unchanged (ADR 0032), so older snapshots show the new references of such
    /// a file too, which is right: the content is the same.
    ///
    /// # Errors
    /// `STO-901` if the snapshot is not the newest of its project or the data does not fit it;
    /// nothing is changed then.
    fn replace_references(&self, replacement: ReferenceReplacement)
    -> Result<Snapshot, CodedError>;
}
