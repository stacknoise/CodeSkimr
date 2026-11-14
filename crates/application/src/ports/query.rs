//! Read access to the newest facts of a project, used by the driving adapters
//! (adapter: `codeskimr_storage`).

use codeskimr_domain::{
    LayerEdge, Metric, NamespaceGraph, ProjectId, QualifiedName, RelativePath, Snapshot,
    SnapshotId, Span, Symbol, SymbolId, SymbolKind, SymbolName,
};

use crate::error::CodedError;
use crate::ports::SymbolLabel;

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

/// One result of a search over file paths: a file that declares at least one type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHit {
    /// Path of the file.
    pub path: RelativePath,
    /// Lines of code of the file.
    pub loc: u32,
    /// The first type the file declares, which is where the explorer opens it.
    pub first_type: SymbolId,
}

/// A place in a file where a symbol is named, as the source column of the explorer shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeSpan {
    /// The symbol that is named there.
    pub to: SymbolId,
    /// Where the name stands.
    pub span: Span,
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
    /// How many other symbols of the snapshot have the same qualified name. Names are not keys
    /// (ADR 0036): a lookup by name gives the one with the lowest id.
    pub same_name: u32,
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

    /// Fuzzy search over the paths of the files of a snapshot that declare a type, best match
    /// first, at most `limit` hits.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn search_files(
        &self,
        snapshot: SnapshotId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<FileHit>, CodedError>;

    /// The places in the lines `first..=last` (zero-based) of a file where a symbol is named,
    /// in the order of the text. A file or line without references gives none.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn code_spans(
        &self,
        snapshot: SnapshotId,
        path: &RelativePath,
        lines: (u32, u32),
    ) -> Result<Vec<CodeSpan>, CodedError>;

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

    /// The types that use one type and the types it uses, with the number of references; `None`
    /// if the symbol is not a type of the snapshot.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn type_focus(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<TypeFocus>, CodedError>;
}

/// A type next to the focus, and the references between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNeighbour {
    /// The neighbouring type.
    pub label: SymbolLabel,
    /// Number of references behind the link.
    pub count: u32,
}

/// One type with the types that use it and the types it uses, the most referenced first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFocus {
    /// The type itself.
    pub focus: SymbolLabel,
    /// Types that use it.
    pub users: Vec<TypeNeighbour>,
    /// Types it uses.
    pub used: Vec<TypeNeighbour>,
}
