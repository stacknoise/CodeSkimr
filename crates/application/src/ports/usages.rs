//! Usages and impact of a symbol (adapter: `codeskimr_storage`), blueprint 9.

use codeskimr_domain::{
    Direction, QualifiedName, Reference, RelativePath, SnapshotId, SymbolId, SymbolKind,
    UsageQuery, Visibility,
};

use crate::error::CodedError;

/// A symbol as a result line: enough to show and to ask about it again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLabel {
    /// The symbol.
    pub id: SymbolId,
    /// Its qualified name.
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
}

/// One reference of a walk with the places on both ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageEntry {
    /// Step of the walk, starting at one.
    pub depth: u32,
    /// The reference with kind, location and confidence.
    pub reference: Reference,
    /// File of the occurrence.
    pub path: RelativePath,
    /// Symbol that contains the occurrence.
    pub from: SymbolLabel,
    /// Symbol that is referred to.
    pub to: SymbolLabel,
}

/// The references of a walk, cut to a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsagePage {
    /// Number of references the walk found, also those that were cut.
    pub total: usize,
    /// The first references, nearest step first.
    pub entries: Vec<UsageEntry>,
}

/// A symbol that a change can reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactedSymbol {
    /// The symbol.
    pub label: SymbolLabel,
    /// Step of the walk it was reached in, starting at one.
    pub depth: u32,
    /// Its visibility; a public one is part of an interface.
    pub visibility: Option<Visibility>,
    /// File that declares it.
    pub path: RelativePath,
}

/// What a change of one symbol can reach, cut to a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactPage {
    /// Number of symbols that are reached, also those that were cut.
    pub total: usize,
    /// The nearest symbols first.
    pub symbols: Vec<ImpactedSymbol>,
}

/// One symbol outside a scope that the scope depends on, or that depends on the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    /// The symbol on the far side of the edge.
    pub other: SymbolLabel,
    /// File that declares it.
    pub path: RelativePath,
    /// Number of references between the scope and this symbol.
    pub count: u32,
}

/// The dependencies of one symbol and everything it contains, cut to a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyPage {
    /// Number of symbols on the far side, also those that were cut.
    pub total: usize,
    /// The symbols with the most references first.
    pub edges: Vec<DependencyEdge>,
}

/// The question of [`UsageIndex::dependencies`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DependencyQuery {
    /// The scope: a symbol with everything it contains.
    pub symbol: SymbolId,
    /// Which way the edges go.
    pub direction: Direction,
    /// Whether references from tests count.
    pub include_tests: bool,
}

/// The reverse index of the newest snapshots. A symbol or snapshot that does not exist has no
/// usages; ask `CodeIndex::symbol` first to tell the two apart.
pub trait UsageIndex: Send + Sync {
    /// The references a walk from `query.symbol` finds, at most `limit` of them.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn usages(
        &self,
        snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> Result<UsagePage, CodedError>;

    /// The symbols that an incoming walk from `symbol` reaches, `depth` steps deep, at most
    /// `limit` of them. References from tests always count. The caller holds `depth` and `limit`
    /// to the configured maximum; the index does not.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn impact(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
        depth: u32,
        limit: u32,
    ) -> Result<ImpactPage, CodedError>;

    /// The symbols outside `symbol` and its members that it uses (`Direction::Outgoing`) or that
    /// use it (`Direction::Incoming`), with the number of references to each, at most `limit`
    /// of them. A namespace or type is a scope that contains its members. References from
    /// tests count only with `include_tests`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn dependencies(
        &self,
        snapshot: SnapshotId,
        query: &DependencyQuery,
        limit: u32,
    ) -> Result<DependencyPage, CodedError>;
}
