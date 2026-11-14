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

    /// Reserves `count` consecutive ids and returns the first one, so that several workers can
    /// hand out ids from blocks of their own and the result does not depend on their timing.
    #[must_use]
    pub fn reserve(&self, count: u64) -> u64 {
        self.next.fetch_add(count, Ordering::Relaxed)
    }

    /// Makes sure no id below `first` is handed out any more; a sequence that is already above
    /// it stays as it is. Shared sequences use this to catch up with what the store holds.
    pub fn advance_to(&self, first: u64) {
        self.next.fetch_max(first, Ordering::Relaxed);
    }

    /// The next id this sequence would hand out; the difference to the first id is the number of
    /// ids handed out so far.
    #[must_use]
    pub fn peek(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_to_only_moves_forward() {
        let ids = IdSequence::starting_at(10);
        ids.advance_to(5);
        assert_eq!(ids.file().get(), 10);
        ids.advance_to(50);
        assert_eq!(ids.file().get(), 50);
    }
}
