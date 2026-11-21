//! What an incremental scan reads of the references instead of all of them (ADR 0085): those that
//! point at some symbols, those of some files, the files level 2 verified, the pairs for coupling
//! and the number of ambiguous places the snapshot stores.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    Level2Replacement, ProjectRepository, SnapshotData, SnapshotStore, StoredReferences,
};
use codeskimr_domain::{
    FileId, Reference, ReferenceKind, RelativePath, ScanMode, SnapshotId, SymbolId, SymbolKind,
};
use common::{PROJECT, data, file, header, reference, registered_store, symbol};

const FIRST: u64 = 10;
const SECOND: u64 = 11;

/// Three files with one function each: `one` (41 in file 31) calls `two` (42 in 32) twice and is
/// used by a test, `two` calls `three` (43 in 33), `three` calls `one`.
#[cfg(test)]
fn scan() -> SnapshotData {
    let mut scan = data(header(FIRST, ScanMode::Full));
    scan.files = vec![
        file(FIRST, 31, "one.rs", "h1"),
        file(FIRST, 32, "two.rs", "h2"),
        file(FIRST, 33, "three.rs", "h3"),
    ];
    scan.symbols = vec![
        symbol(41, 31, SymbolKind::Function, "m.one", None),
        symbol(42, 32, SymbolKind::Function, "m.two", None),
        symbol(43, 33, SymbolKind::Function, "m.three", None),
    ];
    scan.references = vec![
        reference(51, 31, 41, 42),
        reference(52, 31, 41, 42),
        Reference {
            kind: ReferenceKind::Test,
            ..reference(53, 31, 41, 41)
        },
        reference(54, 32, 42, 43),
        reference(55, 33, 43, 41),
    ];
    scan
}

#[cfg(test)]
fn ids(references: &[Reference]) -> Vec<u64> {
    references.iter().map(|found| found.id.get()).collect()
}

#[cfg(test)]
fn read(store: &codeskimr_storage::SqliteStore, which: StoredReferences<'_>) -> Vec<u64> {
    let snapshot = store.latest_state(PROJECT).unwrap().snapshot.unwrap();
    ids(&store.stored_references(snapshot, which).unwrap())
}

#[test]
fn references_are_read_by_target_by_start_by_file_or_all() {
    let store = registered_store();
    store.persist(scan()).unwrap();
    let to = [SymbolId::new(42), SymbolId::new(43)];
    assert_eq!(read(&store, StoredReferences::To(&to)), [51, 52, 54]);
    let of = [FileId::new(31), FileId::new(33)];
    assert_eq!(read(&store, StoredReferences::In(&of)), [51, 52, 53, 55]);
    let from = [SymbolId::new(42), SymbolId::new(43)];
    assert_eq!(read(&store, StoredReferences::From(&from)), [54, 55]);
    assert_eq!(read(&store, StoredReferences::All), [51, 52, 53, 54, 55]);
    assert_eq!(read(&store, StoredReferences::To(&[])), Vec::<u64>::new());
}

/// An incremental scan that removes `two.rs`; the rows of the other files are shared.
#[cfg(test)]
fn without_two() -> SnapshotData {
    let mut next = data(header(SECOND, ScanMode::Incremental));
    next.removed = vec![RelativePath::new("two.rs").unwrap()];
    next
}

#[test]
fn a_reference_whose_target_left_the_snapshot_is_not_read() {
    let store = registered_store();
    store.persist(scan()).unwrap();
    store.persist(without_two()).unwrap();
    let of = [FileId::new(31), FileId::new(32)];
    assert_eq!(read(&store, StoredReferences::In(&of)), [53]);
    assert_eq!(read(&store, StoredReferences::All), [53, 55]);
    let gone = [SymbolId::new(42)];
    assert_eq!(read(&store, StoredReferences::To(&gone)), Vec::<u64>::new());
}

#[test]
fn the_pairs_hold_each_start_and_target_once_without_test_references() {
    let store = registered_store();
    store.persist(scan()).unwrap();
    let snapshot = SnapshotId::new(FIRST);
    let pair = |from, to| (SymbolId::new(from), SymbolId::new(to));
    let of = [FileId::new(31), FileId::new(33)];
    assert_eq!(
        store.reference_pairs(snapshot, &of, &[]).unwrap(),
        [pair(41, 42), pair(43, 41)]
    );
    let one = [SymbolId::new(41)];
    assert_eq!(
        store.reference_pairs(snapshot, &of, &one).unwrap(),
        [pair(43, 41)]
    );
    assert_eq!(store.reference_pairs(snapshot, &[], &[]).unwrap(), []);
}

#[cfg(test)]
fn verified_in_two() -> Level2Replacement {
    Level2Replacement {
        snapshot: SnapshotId::new(FIRST),
        files: vec![FileId::new(32)],
        symbols: Vec::new(),
        references: vec![Reference {
            verified: true,
            ..reference(61, 32, 42, 43)
        }],
    }
}

#[test]
fn the_files_that_level_two_verified_are_named() {
    let store = registered_store();
    store.persist(scan()).unwrap();
    let snapshot = SnapshotId::new(FIRST);
    assert_eq!(store.verified_files(snapshot).unwrap(), []);
    store.replace_level2(verified_in_two()).unwrap();
    assert_eq!(store.verified_files(snapshot).unwrap(), [FileId::new(32)]);
}

#[test]
fn a_snapshot_stores_its_ambiguous_places_until_level_two_changes_its_references() {
    let store = registered_store();
    let mut counted = scan();
    counted.ambiguous_places = Some(4);
    store.persist(counted).unwrap();
    let places = || store.latest_state(PROJECT).unwrap().ambiguous_places;
    assert_eq!(places(), Some(4));
    store.replace_level2(verified_in_two()).unwrap();
    assert_eq!(places(), None, "verified references change the count");
}

#[test]
fn a_snapshot_whose_writer_did_not_count_has_no_number() {
    let store = registered_store();
    store.persist(scan()).unwrap();
    assert_eq!(store.latest_state(PROJECT).unwrap().ambiguous_places, None);
}
