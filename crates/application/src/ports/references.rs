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
