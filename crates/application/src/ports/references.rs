//! The stored references that leave the functions of some files (adapter: `codeskimr_storage`).

use codeskimr_domain::{QualifiedName, RelativePath, SnapshotId, Span};

use crate::error::CodedError;

/// One stored reference that a symbol makes to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSite {
    /// File of the referring symbol.
    pub path: RelativePath,
    /// The referring symbol: usually a function, method or constructor.
    pub from: QualifiedName,
    /// Declaration range of the referring symbol, zero-based.
    pub span: Span,
    /// The symbol it refers to.
    pub to: QualifiedName,
}

/// Queries on the references of the newest snapshots (blueprint 10.4: layering for
/// `check_conventions`).
pub trait ReferenceSites: Send + Sync {
    /// The certain references that start in one of `files`, without those of kind `test` and
    /// without those whose name matches several symbols (the filter of the layer graph).
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn references_from(
        &self,
        snapshot: SnapshotId,
        files: &[RelativePath],
    ) -> Result<Vec<ReferenceSite>, CodedError>;
}

/// One place where a symbol refers to another: the line of the reference, not the declaration
/// of the referring symbol (ADR 0104).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePlace {
    /// File that holds the reference.
    pub path: RelativePath,
    /// Range of the reference, zero-based.
    pub span: Span,
    /// The referring symbol.
    pub from: QualifiedName,
    /// The symbol it refers to.
    pub to: QualifiedName,
}

/// Every certain reference of a snapshot, one after the other, so that a rule can look at them
/// without holding millions of them (adapter: `codeskimr_storage`).
pub trait ReferencePlaces: Send + Sync {
    /// Hands each certain reference that is not of kind `test` and does not match several
    /// symbols (the filter of the layer graph) to `visit`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn for_each_reference(
        &self,
        snapshot: SnapshotId,
        visit: &mut dyn FnMut(ReferencePlace),
    ) -> Result<(), CodedError>;
}
