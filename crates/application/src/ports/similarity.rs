//! Looking up similar functions among the stored fingerprints (adapter: `codeskimr_storage`).

use codeskimr_domain::{
    Fingerprints, QualifiedName, RelativePath, SnapshotId, Span, SymbolId, SymbolKind,
};

use crate::error::CodedError;

/// A stored function that resembles the code that was asked about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarHit {
    /// The function, method or constructor.
    pub symbol: SymbolId,
    /// Its qualified name.
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
    /// File that declares it.
    pub path: RelativePath,
    /// Declaration range, zero-based.
    pub span: Span,
    /// Jaccard similarity of the fingerprints in whole percent, rounded down. It is the measure of
    /// `Fingerprints::similarity` and of the duplicate check.
    pub percent: u32,
}

/// What `SimilarityIndex::similar` is asked.
#[derive(Debug, Clone, Copy)]
pub struct SimilarQuery<'a> {
    /// The fingerprints to look for.
    pub fingerprints: &'a Fingerprints,
    /// Smallest similarity in percent that counts.
    pub minimum_percent: u32,
    /// A symbol that is left out, so a function does not find itself.
    pub exclude: Option<SymbolId>,
    /// At most this many hits.
    pub limit: u32,
}

/// Queries on the fingerprints of the newest snapshots (blueprint 10.4: the same measure as the
/// duplicate check of `check_conventions`).
pub trait SimilarityIndex: Send + Sync {
    /// The stored fingerprints of one symbol, `None` if it has none (not a callable, or too
    /// short).
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn fingerprints_of(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<Fingerprints>, CodedError>;

    /// The stored functions that are at least as similar to the query as it asks, most similar
    /// first (lowest symbol id first on a tie).
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn similar(
        &self,
        snapshot: SnapshotId,
        query: &SimilarQuery<'_>,
    ) -> Result<Vec<SimilarHit>, CodedError>;
}
