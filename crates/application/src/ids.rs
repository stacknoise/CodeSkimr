//! Allocation of ids for one scan. Ids are handed out before persisting, so a snapshot can be
//! written in one transaction.

use std::sync::atomic::{AtomicU64, Ordering};

use codeskimr_domain::{FileId, ReferenceId, SnapshotId, SymbolId};

/// Hands out unique ids. Share it by reference; it never repeats a value.
#[derive(Debug)]
pub struct IdSequence {
    next: AtomicU64,
}

impl IdSequence {
    /// Creates a sequence whose first id is `first`, for example one above the highest stored id.
    #[must_use]
    pub const fn starting_at(first: u64) -> Self {
        Self {
            next: AtomicU64::new(first),
        }
    }

    fn take(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }

    /// The next free snapshot id.
    #[must_use]
    pub fn snapshot(&self) -> SnapshotId {
        SnapshotId::new(self.take())
    }

    /// The next free file id.
    #[must_use]
    pub fn file(&self) -> FileId {
        FileId::new(self.take())
    }

    /// The next free symbol id.
    #[must_use]
    pub fn symbol(&self) -> SymbolId {
        SymbolId::new(self.take())
    }

    /// The next free reference id.
    #[must_use]
    pub fn reference(&self) -> ReferenceId {
        ReferenceId::new(self.take())
    }
}
