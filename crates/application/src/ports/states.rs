//! What the newest snapshot says about the targets of annotations (adapter: `codeskimr_storage`).

use codeskimr_domain::{ContentHash, Loc, QualifiedName, SnapshotId, SymbolKind, SymbolName};

use crate::error::CodedError;

/// One symbol of a snapshot as the alert step compares it with a baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolState {
    /// Kind of the symbol.
    pub kind: SymbolKind,
    /// Its qualified name.
    pub name: QualifiedName,
    /// Its signature as stored, if it has one.
    pub signature: Option<String>,
    /// Hash over its tokens; `None` for a symbol that has none (ADR 0068).
    pub content_hash: Option<ContentHash>,
    /// Lines of code, 0 if the symbol is not measured.
    pub loc: Loc,
    /// Cyclomatic complexity, 0 if the symbol is not measured.
    pub cc: u32,
}

/// A symbol with the simple name of a target that is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameName {
    /// Kind of the symbol.
    pub kind: SymbolKind,
    /// Its qualified name.
    pub name: QualifiedName,
}

/// Looks up annotation targets in a snapshot.
pub trait SymbolStates: Send + Sync {
    /// Every symbol of `snapshot` whose qualified name is one of `names`, of any kind, in one read.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn states(
        &self,
        snapshot: SnapshotId,
        names: &[QualifiedName],
    ) -> Result<Vec<SymbolState>, CodedError>;

    /// The content hashes of the types and members below the namespace `name`, also below its
    /// nested namespaces, in no particular order.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn child_hashes(
        &self,
        snapshot: SnapshotId,
        name: &QualifiedName,
    ) -> Result<Vec<ContentHash>, CodedError>;

    /// How many names written in the files of `snapshot` still name a symbol that is gone, by its
    /// qualified name or by its simple name. Provisional, because the names are not resolved
    /// (blueprint 9.2).
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn remaining_references(
        &self,
        snapshot: SnapshotId,
        name: &QualifiedName,
        simple: &SymbolName,
    ) -> Result<u32, CodedError>;

    /// The symbols of `snapshot` with the simple name `simple`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn same_simple_name(
        &self,
        snapshot: SnapshotId,
        simple: &SymbolName,
    ) -> Result<Vec<SameName>, CodedError>;
}
