//! Writing snapshots: totals, atomicity, incremental carry-over and retention.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    Confirmation, PreviousIndex, ProjectRepository, ScanWarning, SkippedFile, SnapshotData,
    SnapshotStore, SymbolKey,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    CommitHash, ContentHash, FileId, LanguageId, MetricKind, RelativePath, ScanMode, SnapshotId,
    SymbolKind, Timestamp,
};
use common::{
    PROJECT, code_of, data, file, header, metric, project, reference, registered_store, symbol,
};

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

#[test]
fn confirming_the_newest_snapshot_changes_its_time_commit_and_mode_and_nothing_else() {
    let store = registered_store();
    let written = store.persist(first_scan(10)).unwrap();
    let commit = CommitHash::new("abc123").unwrap();
    let confirmed = store
        .confirm(&confirmation(10, Some(commit.clone())))
        .unwrap()
        .expect("the snapshot is the newest")
        .snapshot;
    assert_eq!(confirmed.id, written.id, "no snapshot is added");
    assert_eq!(confirmed.created, Timestamp(2_000));
    assert_eq!(confirmed.commit, Some(commit));
    assert_eq!(confirmed.mode, ScanMode::Incremental);
    assert_eq!(confirmed.stats, written.stats);
    assert_eq!(store.latest_snapshot(PROJECT).unwrap(), Some(confirmed));
    assert_eq!(store.latest_state(PROJECT).unwrap().symbols.len(), 3);
}

/// A scan of the project that found every file as `snapshot` has it.
#[cfg(test)]
fn confirmation(snapshot: u64, commit: Option<CommitHash>) -> Confirmation {
    Confirmation {
        project: PROJECT,
        snapshot: SnapshotId::new(snapshot),
        at: Timestamp(2_000),
        commit,
    }
}

#[test]
fn only_the_newest_snapshot_is_confirmed() {
    let store = registered_store();
    assert_eq!(store.confirm(&confirmation(10, None)).unwrap(), None);
    store.persist(first_scan(10)).unwrap();
    let newer = store.persist(b_rewritten("App.B", vec![])).unwrap();
    assert_eq!(
        store.confirm(&confirmation(10, None)).unwrap(),
        None,
        "a snapshot written after the scan read the newest one is not stamped by it"
    );
    assert_eq!(store.latest_snapshot(PROJECT).unwrap(), Some(newer));
}

#[test]
fn a_confirmed_snapshot_tells_the_warnings_of_its_scan() {
    let store = registered_store();
    let mut scan = first_scan(10);
    scan.warnings = vec![ScanWarning {
        code: ErrorCode::AmbiguousReference,
        count: 4,
    }];
    store.persist(scan.clone()).unwrap();
    let confirmed = store.confirm(&confirmation(10, None)).unwrap().unwrap();
    assert_eq!(confirmed.warnings, scan.warnings);
}

#[test]
fn layers_that_change_after_the_scan_build_the_layer_graph_of_the_newest_snapshot_again() {
    let store = registered_store();
    store.persist(first_scan(10)).unwrap();
    assert_eq!(store.layer_edges(SnapshotId::new(10)).unwrap(), Vec::new());
    let layers: &[(&str, &[&str])] = &[("Upper", &["App.A"]), ("Lower", &["App.B"])];
    store
        .save_project(&project(1, "/work/app", layers))
        .unwrap();
    let edges = store.layer_edges(SnapshotId::new(10)).unwrap();
    let shown: Vec<_> = edges
        .iter()
        .map(|edge| (edge.from.as_str(), edge.to.as_str(), edge.count))
        .collect();
    assert_eq!(shown, [("Upper", "Lower", 1)]);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.layer_edges(SnapshotId::new(10)).unwrap(), Vec::new());
}

#[test]
fn the_head_of_the_newest_snapshot_leaves_the_symbols_to_a_second_read() {
    let store = registered_store();
    store.persist(first_scan(10)).unwrap();
    let state = store.latest_state(PROJECT).unwrap();
    let head = store.latest_head(PROJECT).unwrap();
    assert_eq!(head.symbols, Vec::new());
    assert_eq!(head.files, state.files);
    assert_eq!(head.snapshot, state.snapshot);
    assert_eq!(head.names_stored, state.names_stored);
    assert_eq!(head.ambiguous_places, state.ambiguous_places);
    let symbols = store.snapshot_symbols(SnapshotId::new(10)).unwrap();
    assert_eq!(symbols, state.symbols);
}

#[test]
fn the_files_a_scan_left_out_for_good_are_read_with_the_newest_snapshot_only() {
    let store = registered_store();
    let skipped = vec![SkippedFile {
        path: RelativePath::new("old.cs").unwrap(),
        language: LanguageId::new("csharp").unwrap(),
        content_hash: ContentHash::new("h").unwrap(),
        code: ErrorCode::FileEncodingUnsupported,
    }];
    let mut scan = first_scan(10);
    scan.skipped.clone_from(&skipped);
    store.persist(scan).unwrap();
    assert_eq!(store.latest_head(PROJECT).unwrap().skipped, skipped);
    assert_eq!(store.latest_state(PROJECT).unwrap().skipped, skipped);

    // The next scan read the file: its snapshot has no such row and carries none over.
    store.persist(b_rewritten("B", Vec::new())).unwrap();
    assert_eq!(store.latest_head(PROJECT).unwrap().skipped, Vec::new());
}

#[test]
fn persist_returns_the_totals() {
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
    assert_eq!(store.next_free_id().unwrap(), 1);
    store.persist(first_scan(10)).unwrap();
    assert_eq!(store.next_free_id().unwrap(), 32);
}

#[test]
fn the_scanner_reads_the_next_free_ids_through_the_snapshot_port() {
    let store = registered_store();
    store.persist(first_scan(10)).unwrap();
    let port: &dyn SnapshotStore = &store;
    // Files, symbols and references go on at 32; the next snapshot comes after snapshot 10.
    assert_eq!(port.next_free_ids(codeskimr_domain::ProjectId::new(7)).unwrap(), (32, 11));
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
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
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
    let store = registered_store();
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

/// A full scan as snapshot `snapshot` whose files, symbols and references carry the ids from
/// `base` on, as the pipeline issues new ids for a full scan.
#[cfg(test)]
fn full_scan_from(snapshot: u64, base: u64) -> SnapshotData {
    let mut scan = data(header(snapshot, ScanMode::Full));
    scan.files = vec![
        file(snapshot, base, "a.cs", "ha"),
        file(snapshot, base + 1, "b.cs", "hb"),
    ];
    scan.symbols = vec![
        symbol(base + 2, base, SymbolKind::Class, "App.A", None),
        symbol(base + 3, base + 1, SymbolKind::Class, "App.B", None),
    ];
    scan.references = vec![
        reference(base + 4, base, base + 2, base + 3),
        reference(base + 5, base + 1, base + 3, base + 2),
    ];
    scan
}

#[test]
fn the_references_of_the_newest_snapshot_do_not_include_those_of_older_full_scans() {
    let store = common::store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store.persist(full_scan_from(10, 100)).unwrap();
    let newest = store.persist(full_scan_from(20, 200)).unwrap();
    assert_eq!(newest.stats.references, 2);
    let mut ids: Vec<u64> = store
        .latest_index(PROJECT)
        .unwrap()
        .references
        .iter()
        .map(|reference| reference.id.get())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, [204, 205]);
}

#[test]
fn new_metrics_replace_carried_ones_of_the_same_symbol_and_kind() {
    let store = registered_store();
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
    let store = registered_store();
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
fn ce_of_a(store: &codeskimr_storage::ProjectDb, snapshot: u64) -> Vec<f64> {
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
    let store = registered_store();
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
