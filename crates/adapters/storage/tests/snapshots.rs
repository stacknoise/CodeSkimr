//! Writing snapshots: totals, atomicity, incremental carry-over and retention.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    PreviousIndex, ProjectRepository, SnapshotData, SnapshotStore, SymbolKey,
};
use codeskimr_domain::{FileId, MetricKind, RelativePath, ScanMode, SnapshotId, SymbolKind};
use common::{PROJECT, code_of, data, file, header, metric, project, reference, store, symbol};

/// Two files: `a.cs` declares `App.A` with method `Run`, `b.cs` declares `App.B`; `Run` calls `B`.
#[cfg(test)]
fn first_scan(snapshot: u64) -> SnapshotData {
    let mut scan = data(header(snapshot, ScanMode::Full));
    scan.files = vec![
        file(snapshot, 11, "a.cs", "ha"),
        file(snapshot, 12, "b.cs", "hb"),
    ];
    scan.symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.A", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Run", Some(21)),
        symbol(23, 12, SymbolKind::Class, "App.B", None),
    ];
    scan.references = vec![reference(31, 11, 22, 23)];
    scan.metrics = vec![
        metric(21, MetricKind::Loc, 10.0),
        metric(22, MetricKind::CyclomaticComplexity, 3.0),
    ];
    scan
}

/// An incremental scan that analyses `b.cs` again as snapshot 40, with its class `name` (id 42).
#[cfg(test)]
fn b_rewritten(name: &str, references: Vec<codeskimr_domain::Reference>) -> SnapshotData {
    let mut scan = data(header(40, ScanMode::Incremental));
    scan.files = vec![file(40, 41, "b.cs", "hb2")];
    scan.symbols = vec![symbol(42, 41, SymbolKind::Class, name, None)];
    scan.references = references;
    scan
}

#[cfg(test)]
fn registered() -> codeskimr_storage::SqliteStore {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store
}

#[test]
fn persist_returns_the_totals() {
    let store = registered();
    let snapshot = store.persist(first_scan(10)).unwrap();
    assert_eq!(
        (
            snapshot.stats.files,
            snapshot.stats.symbols,
            snapshot.stats.references
        ),
        (2, 3, 1)
    );
    assert_eq!(store.latest_snapshot(PROJECT).unwrap(), Some(snapshot));
}

#[test]
fn a_failing_snapshot_leaves_nothing_behind() {
    let store = registered();
    let mut broken = first_scan(10);
    broken.references.push(reference(32, 11, 22, 999));
    let error = store.persist(broken).unwrap_err();
    assert_eq!(code_of(&error), "STO-901");
    assert!(store.latest_snapshot(PROJECT).unwrap().is_none());
    assert_eq!(store.latest_files(PROJECT).unwrap(), Vec::new());
    assert_eq!(store.next_free_id().unwrap(), 1);
}

#[test]
fn incremental_scan_keeps_unchanged_files_and_replaces_changed_ones() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let snapshot = store.persist(b_rewritten("App.B2", Vec::new())).unwrap();
    assert_eq!(
        (
            snapshot.stats.files,
            snapshot.stats.symbols,
            snapshot.stats.references
        ),
        (2, 3, 0),
        "a.cs carried over (2 symbols), b.cs replaced (1 symbol); the call to the old B is gone"
    );
    let files = store.latest_files(PROJECT).unwrap();
    let hashes: Vec<_> = files
        .iter()
        .map(|f| (f.path.as_str(), f.content_hash.as_str()))
        .collect();
    assert_eq!(hashes, vec![("a.cs", "ha"), ("b.cs", "hb2")]);
    assert!(files.iter().all(|f| f.snapshot == SnapshotId::new(40)));
}

#[test]
fn incremental_scan_carries_references_inside_unchanged_files() {
    let store = registered();
    let mut first = data(header(10, ScanMode::Full));
    first.files = vec![file(10, 11, "a.cs", "ha")];
    first.symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.A", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Run", Some(21)),
    ];
    first.references = vec![reference(31, 11, 22, 21)];
    store.persist(first).unwrap();
    let mut second = data(header(40, ScanMode::Incremental));
    second.files = vec![file(40, 41, "b.cs", "hb")];
    second.symbols = vec![symbol(42, 41, SymbolKind::Class, "App.B", None)];
    let snapshot = store.persist(second).unwrap();
    assert_eq!(snapshot.stats.references, 1);
    assert_eq!(
        store
            .usage_count(snapshot.id, symbol(21, 11, SymbolKind::Class, "x", None).id)
            .unwrap(),
        1
    );
}

#[test]
fn full_scan_does_not_carry_anything() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let mut scan = data(header(40, ScanMode::Full));
    scan.files = vec![file(40, 41, "c.cs", "hc")];
    let snapshot = store.persist(scan).unwrap();
    assert_eq!(snapshot.stats.files, 1);
}

#[test]
fn only_the_newest_snapshots_are_kept() {
    let store = common::store(2);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    for snapshot in [10, 20, 30] {
        store.persist(first_scan_with_ids(snapshot)).unwrap();
    }
    let kept = |snapshot: u64| {
        store
            .usage_count(
                SnapshotId::new(snapshot),
                symbol(first_id(snapshot) + 1, 0, SymbolKind::Class, "x", None).id,
            )
            .unwrap()
    };
    assert_eq!(kept(10), 0, "pruned snapshots lose their read models");
    assert_eq!(kept(20), 1);
    assert_eq!(kept(30), 1);
}

#[cfg(test)]
fn first_id(snapshot: u64) -> u64 {
    snapshot * 100
}

/// A scan whose ids are unique per snapshot: the target symbol is always `first_id + 1`.
#[cfg(test)]
fn first_scan_with_ids(snapshot: u64) -> SnapshotData {
    let base = first_id(snapshot);
    let mut scan = data(header(snapshot, ScanMode::Full));
    scan.files = vec![file(snapshot, base + 10, "a.cs", "ha")];
    scan.symbols = vec![
        symbol(base, base + 10, SymbolKind::Method, "App.A.Run", None),
        symbol(base + 1, base + 10, SymbolKind::Class, "App.A", None),
    ];
    scan.references = vec![reference(base + 20, base + 10, base, base + 1)];
    scan
}

#[test]
fn next_free_id_is_above_every_stored_id() {
    let store = registered();
    assert_eq!(store.next_free_id().unwrap(), 1);
    store.persist(first_scan(10)).unwrap();
    assert_eq!(store.next_free_id().unwrap(), 32);
}

/// Incremental scan 40 that analyses only `c.cs` and adds the given references.
#[cfg(test)]
fn scan_of_c(references: Vec<codeskimr_domain::Reference>) -> SnapshotData {
    let mut scan = data(header(40, ScanMode::Incremental));
    scan.files = vec![file(40, 41, "c.cs", "hc")];
    scan.references = references;
    scan
}

#[cfg(test)]
#[test]
fn new_references_may_point_at_carried_symbols() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let mut scan = scan_of_c(vec![reference(43, 41, 42, 21)]);
    scan.symbols = vec![symbol(42, 41, SymbolKind::Method, "App.C.Go", None)];
    let snapshot = store.persist(scan).unwrap();
    assert_eq!(snapshot.stats.references, 2);
    assert_eq!(
        store
            .usage_count(snapshot.id, codeskimr_domain::SymbolId::new(21))
            .unwrap(),
        1
    );
}

#[cfg(test)]
#[test]
fn a_failing_incremental_scan_keeps_the_previous_snapshot_intact() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    assert!(
        store
            .persist(scan_of_c(vec![reference(43, 41, 998, 999)]))
            .is_err()
    );
    let latest = store.latest_snapshot(PROJECT).unwrap().unwrap();
    assert_eq!((latest.id, latest.stats.files), (SnapshotId::new(10), 2));
    assert_eq!(store.latest_files(PROJECT).unwrap().len(), 2);
}

#[test]
fn removed_files_leave_the_snapshot_with_their_symbols_and_references() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let mut scan = data(header(40, ScanMode::Incremental));
    scan.removed = vec![RelativePath::new("b.cs").unwrap()];
    let snapshot = store.persist(scan).unwrap();
    assert_eq!(
        (
            snapshot.stats.files,
            snapshot.stats.symbols,
            snapshot.stats.references
        ),
        (1, 2, 0),
        "b.cs and App.B are gone, and with them the call from Run to B"
    );
    let files = store.latest_files(PROJECT).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path.as_str(), "a.cs");
}

#[test]
fn resolved_again_references_of_unchanged_files_replace_the_dropped_ones() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let again = vec![reference(43, 11, 22, 42)];
    let snapshot = store.persist(b_rewritten("App.B", again)).unwrap();
    assert_eq!(snapshot.stats.references, 1);
    let index = store.latest_index(PROJECT).unwrap();
    assert_eq!(index.references.len(), 1);
    assert_eq!(
        index.references[0].to,
        symbol(42, 41, SymbolKind::Class, "x", None).id
    );
}

#[test]
fn new_metrics_replace_carried_ones_of_the_same_symbol_and_kind() {
    let store = registered();
    store.persist(first_scan(10)).unwrap();
    let mut scan = data(header(40, ScanMode::Incremental));
    scan.metrics = vec![metric(21, MetricKind::Loc, 99.0)];
    store.persist(scan).unwrap();
    let detail = store
        .symbol_detail(
            SnapshotId::new(40),
            &SymbolKey::Id(symbol(21, 11, SymbolKind::Class, "x", None).id),
        )
        .unwrap()
        .unwrap();
    let loc: Vec<f64> = detail
        .metrics
        .iter()
        .filter(|m| m.kind == MetricKind::Loc)
        .map(|m| m.value.get())
        .collect();
    assert_eq!(loc, vec![99.0]);
}

#[test]
fn the_latest_index_holds_every_symbol_and_reference_or_nothing() {
    let store = registered();
    assert_eq!(
        store.latest_index(PROJECT).unwrap(),
        PreviousIndex::default()
    );
    store.persist(first_scan(10)).unwrap();
    let index = store.latest_index(PROJECT).unwrap();
    assert_eq!((index.symbols.len(), index.references.len()), (3, 1));
    assert_eq!(index.references[0].file, FileId::new(11));
}

/// The Ce values of the class `App.A` (id 21) in a snapshot.
#[cfg(test)]
fn ce_of_a(store: &codeskimr_storage::SqliteStore, snapshot: u64) -> Vec<f64> {
    let key = SymbolKey::Id(symbol(21, 11, SymbolKind::Class, "x", None).id);
    store
        .symbol_detail(SnapshotId::new(snapshot), &key)
        .unwrap()
        .map(|detail| {
            detail
                .metrics
                .iter()
                .filter(|m| m.kind == MetricKind::EfferentCoupling)
                .map(|m| m.value.get())
                .collect()
        })
        .unwrap_or_default()
}

/// Coupling depends on references across files, so every snapshot keeps its own values: a new
/// value does not change an older snapshot, and a snapshot that computed none has none (ADR 0037).
#[test]
fn coupling_metrics_belong_to_their_snapshot() {
    let store = registered();
    let mut first = first_scan(10);
    first
        .metrics
        .push(metric(21, MetricKind::EfferentCoupling, 1.0));
    store.persist(first).unwrap();
    let mut second = data(header(40, ScanMode::Incremental));
    second
        .metrics
        .push(metric(21, MetricKind::EfferentCoupling, 5.0));
    store.persist(second).unwrap();
    store
        .persist(data(header(50, ScanMode::Incremental)))
        .unwrap();
    assert_eq!(ce_of_a(&store, 10), vec![1.0]);
    assert_eq!(ce_of_a(&store, 40), vec![5.0]);
    assert_eq!(ce_of_a(&store, 50), Vec::<f64>::new());
}
