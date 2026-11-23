//! The facts the statistics screen ranks (ADR 0081; adapter: `codeskimr_storage`).

use codeskimr_domain::{SnapshotId, SymbolFacts};

use crate::error::CodedError;

/// Reads what the statistics need to know about the symbols of a snapshot.
pub trait CodeStatistics: Send + Sync {
    /// Every symbol of the snapshot with its usage counts and measured values. A symbol is
    /// marked as test code if its file holds a reference of kind `test`, which only test files
    /// do. A snapshot that is not stored has no symbols.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn symbol_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFacts>, CodedError>;

    /// The same symbols, with test code marked (by the files and by `test_names`, the names of
    /// folders and modules that hold tests), shared: they are loaded once and kept until the
    /// usages of the snapshot change, so every statistics view reads the same copy. A second
    /// caller that comes while they load waits for them.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn shared_facts(
        &self,
        snapshot: SnapshotId,
        test_names: &[String],
    ) -> Result<std::sync::Arc<Vec<SymbolFacts>>, CodedError>;
}
