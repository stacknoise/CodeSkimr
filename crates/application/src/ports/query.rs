//! Read access to the newest facts of a project, used by the driving adapters
//! (adapter: `codeskimr_storage`).

use codeskimr_domain::{
    LayerEdge, Metric, NamespaceGraph, ProjectId, QualifiedName, RelativePath, Snapshot,
    SnapshotId, Symbol, SymbolId, SymbolKind, SymbolName,
};

use crate::error::CodedError;

/// One result of a name search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    /// The symbol.
    pub symbol: SymbolId,
    /// Its simple name.
    pub name: SymbolName,
    /// Its qualified name.
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
    /// File that declares it.
    pub path: RelativePath,
    /// The enclosing symbol, `None` for top-level symbols.
    pub parent: Option<SymbolId>,
}

/// How a caller names one symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKey {
    /// By id within a snapshot.
    Id(SymbolId),
    /// By qualified name; if several symbols share it, the one with the lowest id answers.
    Qualified(QualifiedName),
}

/// A direct member of a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRef {
    /// The member.
    pub id: SymbolId,
    /// Its simple name.
    pub name: SymbolName,
    /// Its kind.
    pub kind: SymbolKind,
}

/// A symbol with everything one lookup needs.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolDetail {
    /// The symbol itself.
    pub symbol: Symbol,
    /// File that declares it.
    pub path: RelativePath,
    /// Direct members, ordered by id.
    pub members: Vec<MemberRef>,
    /// Measured values of the symbol itself.
    pub metrics: Vec<Metric>,
}

/// Queries on the newest snapshots.
pub trait CodeIndex: Send + Sync {
    /// The newest snapshot of a project, `None` before the first scan.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn latest_snapshot(&self, project: ProjectId) -> Result<Option<Snapshot>, CodedError>;

    /// Fuzzy search over symbol names, best match first, at most `limit` hits.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn search(
        &self,
        snapshot: SnapshotId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchHit>, CodedError>;

    /// One symbol with its members and metrics, `None` if the snapshot has no such symbol.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn symbol(
        &self,
        snapshot: SnapshotId,
        key: &SymbolKey,
    ) -> Result<Option<SymbolDetail>, CodedError>;

    /// The dependency map of a snapshot, aggregated per pair of layers.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn layer_edges(&self, snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError>;

    /// The namespaces of a snapshot with their type counts, and the references between them.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn namespace_graph(&self, snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError>;
}
