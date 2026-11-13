//! Stored fingerprints: written with the snapshot, carried over, found by similarity.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    SimilarQuery, SimilarityIndex, SnapshotStore, SymbolFingerprints,
};
use codeskimr_domain::{Fingerprint, Fingerprints, ScanMode, SnapshotId, SymbolId, SymbolKind};
use common::{PROJECT, data, file, header, project, store, symbol};

#[cfg(test)]
fn prints(hashes: &[u64]) -> Fingerprints {
    let mut entries = Vec::new();
    for (position, hash) in (0..).zip(hashes) {
        entries.push(Fingerprint {
            hash: *hash,
            position,
        });
    }
    Fingerprints::from_entries(entries)
}

#[cfg(test)]
fn of(symbol: u64, hashes: &[u64]) -> SymbolFingerprints {
    SymbolFingerprints {
        symbol: SymbolId::new(symbol),
        fingerprints: prints(hashes),
    }
}

/// Snapshot 10: `a.cs` with the functions 21 (hashes 1 to 4), 22 (hashes 1 to 3, 9) and 23 (all
/// above `i64::MAX`, to see that the bit pattern survives); `b.cs` with 24 (hashes 7 to 9).
#[cfg(test)]
fn first_scan() -> codeskimr_application::ports::SnapshotData {
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "a.cs", "ha"), file(10, 12, "b.cs", "hb")];
    scan.symbols = vec![
        symbol(21, 11, SymbolKind::Method, "App.A.One", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Two", None),
        symbol(23, 11, SymbolKind::Method, "App.A.Big", None),
        symbol(24, 12, SymbolKind::Method, "App.B.Run", None),
    ];
    scan.fingerprints = vec![
        of(21, &[1, 2, 3, 4]),
        of(22, &[1, 2, 3, 9]),
        of(23, &[u64::MAX, u64::MAX - 1]),
        of(24, &[7, 8, 9]),
    ];
    scan
}

#[cfg(test)]
fn registered() -> codeskimr_storage::SqliteStore {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store.persist(first_scan()).unwrap();
    store
}

#[cfg(test)]
fn ask(
    store: &codeskimr_storage::SqliteStore,
    snapshot: SnapshotId,
    hashes: &[u64],
    minimum_percent: u32,
) -> Vec<codeskimr_application::ports::SimilarHit> {
    ask_with(store, snapshot, hashes, (minimum_percent, None, 10))
}

/// The same with a symbol to leave out and a limit: `(minimum percent, excluded, limit)`.
#[cfg(test)]
fn ask_with(
    store: &codeskimr_storage::SqliteStore,
    snapshot: SnapshotId,
    hashes: &[u64],
    (minimum_percent, exclude, limit): (u32, Option<SymbolId>, u32),
) -> Vec<codeskimr_application::ports::SimilarHit> {
    let fingerprints = prints(hashes);
    let query = SimilarQuery {
        fingerprints: &fingerprints,
        minimum_percent,
        exclude,
        limit,
    };
    store.similar(snapshot, &query).unwrap()
}

#[cfg(test)]
const FIRST: SnapshotId = SnapshotId::new(10);

#[test]
fn the_stored_fingerprints_read_back_unchanged() {
    let store = registered();
    let back = store
        .fingerprints_of(FIRST, SymbolId::new(23))
        .unwrap()
        .unwrap();
    assert_eq!(back, prints(&[u64::MAX, u64::MAX - 1]));
    assert!(
        store
            .fingerprints_of(FIRST, SymbolId::new(99))
            .unwrap()
            .is_none()
    );
}

#[test]
fn similar_functions_come_most_similar_first_with_their_place() {
    let store = registered();
    let hits = ask(&store, FIRST, &[1, 2, 3, 4], 30);
    let found: Vec<(u64, u32)> = hits
        .iter()
        .map(|hit| (hit.symbol.get(), hit.percent))
        .collect();
    // 21 is identical, 22 shares 3 of 5 distinct hashes, 24 shares none.
    assert_eq!(found, [(21, 100), (22, 60)]);
    assert_eq!(hits[1].qualified_name.as_str(), "App.A.Two");
    assert_eq!(hits[1].path.as_str(), "a.cs");
}

#[test]
fn the_excluded_symbol_and_the_limit_are_honoured() {
    let store = registered();
    let without = ask_with(
        &store,
        FIRST,
        &[1, 2, 3, 4],
        (1, Some(SymbolId::new(21)), 10),
    );
    assert_eq!(without.len(), 1);
    assert_eq!(without[0].symbol.get(), 22);
    assert_eq!(
        ask_with(&store, FIRST, &[1, 2, 3, 4], (1, None, 1)).len(),
        1
    );
}

#[test]
fn the_percent_matches_the_domain_measure() {
    let store = registered();
    let query = prints(&[1, 2, 3, 4]);
    let hit = &ask(&store, FIRST, &[1, 2, 3, 4], 1)[1];
    let other = store.fingerprints_of(FIRST, hit.symbol).unwrap().unwrap();
    let expected = (query.similarity(&other) * 100.0).floor();
    assert!((f64::from(hit.percent) - expected).abs() < f64::EPSILON);
}

#[test]
fn an_incremental_scan_carries_the_fingerprints_of_unchanged_files() {
    let store = registered();
    let mut next = data(header(40, ScanMode::Incremental));
    next.files = vec![file(40, 41, "b.cs", "hb2")];
    next.symbols = vec![symbol(42, 41, SymbolKind::Method, "App.B.Run", None)];
    next.fingerprints = vec![of(42, &[7, 8, 9, 10])];
    store.persist(next).unwrap();
    let second = SnapshotId::new(40);
    assert!(
        store
            .fingerprints_of(second, SymbolId::new(21))
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .fingerprints_of(second, SymbolId::new(24))
            .unwrap()
            .is_none()
    );
    let hits = ask(&store, second, &[7, 8, 9, 10], 90);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].symbol.get(), 42);
}

#[test]
fn removed_files_leave_no_fingerprints_behind() {
    let store = registered();
    let mut next = data(header(40, ScanMode::Incremental));
    next.removed = vec![codeskimr_domain::RelativePath::new("a.cs").unwrap()];
    store.persist(next).unwrap();
    let hits = ask(&store, SnapshotId::new(40), &[1, 2, 3, 4], 1);
    assert_eq!(hits, Vec::new());
    assert_eq!(PROJECT.get(), 1);
}
