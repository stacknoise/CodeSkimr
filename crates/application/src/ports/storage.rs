//! Persistence (adapter: `codeskimr_storage`).

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    FileId, LanguageId, Metric, Project, ProjectId, Reference, ReferenceId, RelativePath, Snapshot,
    SnapshotId, SourceFile, Symbol, SymbolId,
};

use super::{PendingReference, SymbolFingerprints};
use crate::error::CodedError;

/// Everything one scan produced, written as one unit.
///
/// In an incremental scan the lists hold only what is new: the files that were analysed again,
/// their symbols, the references that are new (also those that start in an unchanged file, because
/// the names of that file were resolved again) and the metrics that were computed again. The
/// store carries everything else over from the previous snapshot, drops the `removed` files with
/// their symbols and the `dropped_references`, and returns the totals of the whole snapshot. A
/// metric of this value replaces a carried metric of the same symbol and kind.
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
    /// Names as written in the analysed files, resolved or not. The store keeps them so that the
    /// next incremental scan can resolve them again against a changed symbol table; it carries
    /// those of the other files over, like their symbols.
    pub pending: Vec<PendingReference>,
    /// Carried references that the full resolution no longer produces, for example because a
    /// changed file now declares a closer candidate; the new snapshot does not contain them.
    /// Empty in a full scan.
    pub dropped_references: Vec<ReferenceId>,
    /// The naming rules of the language packs of this scan (ADR 0036); an incremental scan keeps
    /// the files of the previous snapshot only if its rules are these.
    pub naming_rules: String,
    /// Microseconds spent parsing the files of each language in a full scan; empty in an
    /// incremental scan, which keeps the figures of the previous snapshot.
    pub parse_micros: Vec<(LanguageId, u64)>,
    /// The warnings of the scan, one per code, counted over the whole snapshot.
    pub warnings: Vec<ScanWarning>,
}

/// A warning of a scan: how often something happened, not one message per occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanWarning {
    /// The registered code, a warning.
    pub code: ErrorCode,
    /// How many times it happened in the scan.
    pub count: u64,
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

/// Which of the names as written a snapshot holds to read (see [`ProjectRepository::stored_names`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredNames<'a> {
    /// The base lists of all types: what the resolver needs to know which type inherits from
    /// which.
    Bases,
    /// The imports of these files: what the resolver needs to know what a file sees.
    ImportsOf(&'a [FileId]),
    /// The names written inside these symbols, which are in these files.
    Callers(&'a [FileId], &'a [SymbolId]),
    /// The names, imports excepted, whose target or whose named receiver is one of these simple
    /// names: what the resolver looks up by them.
    Targeting(&'a [String]),
    /// The imports whose target is one of these qualified names.
    ImportsTargeting(&'a [String]),
    /// Every name written in these files.
    OfFiles(&'a [FileId]),
    /// Every name.
    All,
}

/// The newest snapshot of a project as one consistent read: files, symbols and references. Read in
/// one transaction, so a snapshot written in between cannot make the parts disagree. The names as
/// written are read from `snapshot` afterwards, as far as they are needed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreviousState {
    /// The files.
    pub files: Vec<SourceFile>,
    /// All symbols and references.
    pub index: PreviousIndex,
    /// The snapshot that was read, `None` if the project has none.
    pub snapshot: Option<SnapshotId>,
    /// Whether the snapshot stores the names as written. One written before the store did cannot
    /// be continued by an incremental scan, because it could not resolve the names again.
    pub names_stored: bool,
}

/// What level 2 stores for some files of a snapshot: the symbols it creates (ADR 0055) and the
/// references that replace the level 1 references of those files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2Replacement {
    /// The snapshot the references belong to.
    pub snapshot: SnapshotId,
    /// The files whose references are replaced; a file that is not here keeps its references.
    pub files: Vec<FileId>,
    /// The symbols of origin `index` that replace the ones of those files, all in those files,
    /// with ids no stored row has. A file without any gets none of its former ones back.
    pub symbols: Vec<Symbol>,
    /// The new references, all in those files, with ids no stored row has. They may point at
    /// `symbols`.
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

    /// Files and index of the newest snapshot in one read. The default calls `latest_files` and
    /// `latest_index` one after the other and knows no stored names, so a scan over it is always a
    /// full one; an adapter that can read them in one transaction overrides it.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn latest_state(&self, id: ProjectId) -> Result<PreviousState, CodedError> {
        Ok(PreviousState {
            files: self.latest_files(id)?,
            index: self.latest_index(id)?,
            snapshot: None,
            names_stored: false,
        })
    }

    /// The names as written in the files of `snapshot` (one of `latest_state`), as far as `which`
    /// asks. The rows belong to the files, which no scan changes, so the answer is the one of the
    /// state. The default holds none.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn stored_names(
        &self,
        snapshot: SnapshotId,
        which: StoredNames<'_>,
    ) -> Result<Vec<PendingReference>, CodedError> {
        let _ = (snapshot, which);
        Ok(Vec::new())
    }

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

    /// Replaces the symbols of origin `index` and the references of the files of `replacement` in one transaction, updates the
    /// totals and the read models and returns the snapshot. Rows are shared between snapshots
    /// that carry a file unchanged (ADR 0032), so older snapshots show the new references of such
    /// a file too, which is right: the content is the same.
    ///
    /// # Errors
    /// `STO-901` if the snapshot is not the newest of its project or the data does not fit it;
    /// nothing is changed then.
    fn replace_level2(&self, replacement: Level2Replacement) -> Result<Snapshot, CodedError>;
}
