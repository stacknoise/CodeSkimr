//! Allocation of ids for one scan. Ids are handed out before persisting, so a snapshot can be
//! written in one transaction.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_domain::{FileId, ProjectId, ReferenceId, SnapshotId, SymbolId};

use crate::consts::{COUNTER_BITS, MAX_PROJECT_NUMBER};

/// The first snapshot id of `project` (ADR 0095): the project number sits in the bits above the
/// counter, so a snapshot id says which project file holds its rows and stays below 2^53, the
/// largest integer a browser reads exactly. The ids of files, symbols and references are not
/// unique across projects; they are only used together with a snapshot.
#[must_use]
pub const fn first_snapshot_of(project: ProjectId) -> u64 {
    (project.get() << COUNTER_BITS) + 1
}

/// The project whose file holds the snapshot with `id`.
#[must_use]
pub const fn project_of_snapshot(id: SnapshotId) -> ProjectId {
    project_of_raw(id.get())
}

/// The project whose file holds the row with the raw id `id` of a snapshot, an annotation or an
/// alert; their counters start above the project number.
#[must_use]
pub const fn project_of_raw(id: u64) -> ProjectId {
    ProjectId::new(id >> COUNTER_BITS)
}

/// Whether `project` is a number that snapshot ids can carry.
#[must_use]
pub const fn is_project_number(project: ProjectId) -> bool {
    project.get() >= 1 && project.get() <= MAX_PROJECT_NUMBER
}

/// One id sequence per project, shared by the scanners of a process: scans of different projects
/// run side by side and each hands out ids for the file of its own project.
#[derive(Debug, Default)]
pub struct ProjectIds {
    all: Mutex<HashMap<ProjectId, Arc<IdSequence>>>,
}

impl ProjectIds {
    /// The sequence of `project`, moved up to what its file holds: `first_row` for the ids of
    /// files, symbols and references and `first_snapshot` for the snapshots (never below the
    /// first snapshot id of the project).
    #[must_use]
    pub fn of(&self, project: ProjectId, (first_row, first_snapshot): (u64, u64)) -> Arc<IdSequence> {
        let mut all = self.all.lock().unwrap_or_else(PoisonError::into_inner);
        let ids = all
            .entry(project)
            .or_insert_with(|| Arc::new(IdSequence::of_project(project)));
        ids.advance_to(first_row);
        ids.advance_snapshots_to(first_snapshot.max(first_snapshot_of(project)));
        Arc::clone(ids)
    }

    /// Forgets the sequence of a removed project.
    pub fn forget(&self, project: ProjectId) {
        self.all
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&project);
    }
}

/// Hands out unique ids. Share it by reference; it never repeats a value.
#[derive(Debug)]
pub struct IdSequence {
    next: AtomicU64,
    /// The snapshot ids of a project, which carry its number; `None` draws them from `next`.
    snapshots: Option<AtomicU64>,
}

impl IdSequence {
    /// Creates a sequence whose first id is `first`, for example one above the highest stored id.
    #[must_use]
    pub const fn starting_at(first: u64) -> Self {
        Self {
            next: AtomicU64::new(first),
            snapshots: None,
        }
    }

    /// Creates the sequence of one project: rows from 1, snapshots from the first id of the
    /// project.
    #[must_use]
    pub const fn of_project(project: ProjectId) -> Self {
        Self {
            next: AtomicU64::new(1),
            snapshots: Some(AtomicU64::new(first_snapshot_of(project))),
        }
    }

    /// Makes sure no snapshot id below `first` is handed out any more.
    pub fn advance_snapshots_to(&self, first: u64) {
        if let Some(snapshots) = &self.snapshots {
            snapshots.fetch_max(first, Ordering::Relaxed);
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
        SnapshotId::new(
            self.snapshots
                .as_ref()
                .map_or_else(|| self.take(), |next| next.fetch_add(1, Ordering::Relaxed)),
        )
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

    #[test]
    fn a_snapshot_id_names_its_project_and_stays_exact_in_a_browser() {
        let project = ProjectId::new(MAX_PROJECT_NUMBER);
        let last = first_snapshot_of(project) + (1 << COUNTER_BITS) - 2;
        assert_eq!(project_of_snapshot(SnapshotId::new(first_snapshot_of(project))), project);
        assert_eq!(project_of_snapshot(SnapshotId::new(last)), project);
        assert!(last < (1 << 53));
        assert!(is_project_number(project));
        assert!(!is_project_number(ProjectId::new(MAX_PROJECT_NUMBER + 1)));
        assert!(!is_project_number(ProjectId::new(0)));
    }

    #[test]
    fn a_project_sequence_hands_out_snapshots_in_the_range_of_its_project() {
        let ids = ProjectIds::default();
        let project = ProjectId::new(3);
        let sequence = ids.of(project, (7, 0));
        assert_eq!(sequence.snapshot().get(), first_snapshot_of(project));
        assert_eq!(sequence.file().get(), 7);
        let again = ids.of(project, (5, first_snapshot_of(project) + 10));
        assert_eq!(again.snapshot().get(), first_snapshot_of(project) + 10);
        assert_eq!(again.file().get(), 8);
        let other = ids.of(ProjectId::new(4), (1, 0));
        assert_eq!(project_of_snapshot(other.snapshot()), ProjectId::new(4));
    }
}
