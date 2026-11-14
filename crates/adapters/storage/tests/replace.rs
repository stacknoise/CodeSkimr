//! Level 2: replacing the references of some files of the newest snapshot (ADR 0033).

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    ProjectRepository, ReferenceReplacement, SnapshotData, SnapshotStore, SymbolKey,
};
use codeskimr_domain::{FileId, MetricKind, Reference, ScanMode, SnapshotId, SymbolId, SymbolKind};
use common::{PROJECT, code_of, data, file, header, metric, project, reference, store, symbol};

const SNAPSHOT: u64 = 10;

/// `a.cs` holds `A` and `Run`; `b.cs` holds `B` and `Go`. `Run` calls `B` (level 1).
#[cfg(test)]
fn scan() -> SnapshotData {
    let mut scan = data(header(SNAPSHOT, ScanMode::Full));
    scan.files = vec![
        file(SNAPSHOT, 71, "alpha.cs", "h-alpha"),
        file(SNAPSHOT, 72, "beta.cs", "h-beta"),
    ];
    scan.symbols = vec![
        symbol(81, 71, SymbolKind::Class, "Lib.Alpha", None),
        symbol(82, 71, SymbolKind::Method, "Lib.Alpha.Run", Some(81)),
        symbol(83, 72, SymbolKind::Class, "Lib.Beta", None),
        symbol(84, 72, SymbolKind::Method, "Lib.Beta.Go", Some(83)),
    ];
    scan.references = vec![reference(91, 71, 82, 83), reference(92, 72, 84, 81)];
    scan
}

#[cfg(test)]
fn registered() -> codeskimr_storage::SqliteStore {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store.persist(scan()).unwrap();
    store
}

#[cfg(test)]
fn verified(id: u64, file: u64, from: u64, to: u64) -> Reference {
    Reference {
        verified: true,
        ..reference(id, file, from, to)
    }
}

#[cfg(test)]
fn replacement(files: &[u64], references: Vec<Reference>) -> ReferenceReplacement {
    ReferenceReplacement {
        snapshot: SnapshotId::new(SNAPSHOT),
        files: files.iter().copied().map(FileId::new).collect(),
        references,
    }
}

#[cfg(test)]
fn stored(store: &codeskimr_storage::SqliteStore) -> Vec<(u64, bool)> {
    let mut all: Vec<_> = store
        .latest_index(PROJECT)
        .unwrap()
        .references
        .iter()
        .map(|r| (r.id.get(), r.verified))
        .collect();
    all.sort_unstable();
    all
}

#[test]
fn the_references_of_the_files_are_replaced_and_the_others_stay() {
    let store = registered();
    let snapshot = store
        .replace_references(replacement(&[71], vec![verified(900, 71, 82, 84)]))
        .unwrap();
    assert_eq!(stored(&store), [(92, false), (900, true)]);
    assert_eq!(snapshot.stats.references, 2);
}

#[test]
fn the_read_models_follow_the_new_references() {
    let store = registered();
    let usages = |symbol: u64| {
        store
            .usage_count(SnapshotId::new(SNAPSHOT), SymbolId::new(symbol))
            .unwrap()
    };
    assert_eq!((usages(83), usages(84)), (1, 0));
    store
        .replace_references(replacement(&[71], vec![verified(900, 71, 82, 84)]))
        .unwrap();
    assert_eq!((usages(83), usages(84)), (0, 1));
}

#[test]
fn a_replacement_without_references_removes_those_of_the_file() {
    let store = registered();
    store
        .replace_references(replacement(&[71], Vec::new()))
        .unwrap();
    assert_eq!(stored(&store), [(92, false)]);
}

#[test]
fn a_bad_replacement_changes_nothing() {
    let store = registered();
    let wrong_file = replacement(&[71], vec![verified(900, 72, 82, 84)]);
    let unknown_symbol = replacement(&[71], vec![verified(900, 71, 82, 99)]);
    let unknown_file = replacement(&[77], Vec::new());
    for bad in [wrong_file, unknown_symbol, unknown_file] {
        let error = store.replace_references(bad).unwrap_err();
        assert_eq!(code_of(&error), "STO-901");
    }
    assert_eq!(stored(&store), [(91, false), (92, false)]);
}

#[test]
fn an_older_snapshot_that_shares_the_file_follows_in_its_read_models_too() {
    let store = registered();
    let mut later = data(header(20, ScanMode::Incremental));
    later.files = vec![file(20, 73, "gamma.cs", "h-gamma")];
    later.symbols = vec![symbol(85, 73, SymbolKind::Class, "Lib.Gamma", None)];
    store.persist(later).unwrap();
    let usages = |snapshot: u64, symbol: u64| {
        store
            .usage_count(SnapshotId::new(snapshot), SymbolId::new(symbol))
            .unwrap()
    };
    assert_eq!((usages(10, 83), usages(20, 83)), (1, 1));
    let replacement = ReferenceReplacement {
        snapshot: SnapshotId::new(20),
        ..replacement(&[71], vec![verified(900, 71, 82, 84)])
    };
    store.replace_references(replacement).unwrap();
    for snapshot in [10, 20] {
        assert_eq!((usages(snapshot, 83), usages(snapshot, 84)), (0, 1));
    }
}

#[test]
fn only_the_newest_snapshot_may_be_changed() {
    let store = registered();
    let mut later = data(header(20, ScanMode::Full));
    later.files = vec![file(20, 51, "alpha.cs", "h-alpha")];
    store.persist(later).unwrap();
    let error = store
        .replace_references(replacement(&[71], Vec::new()))
        .unwrap_err();
    assert_eq!(code_of(&error), "STO-901");
}

/// The value of a coupling metric of a symbol in the newest snapshot, `None` if there is none.
#[cfg(test)]
fn coupling_of(
    store: &codeskimr_storage::SqliteStore,
    symbol: u64,
    kind: MetricKind,
) -> Option<f64> {
    let key = SymbolKey::Id(SymbolId::new(symbol));
    let detail = store
        .symbol_detail(SnapshotId::new(SNAPSHOT), &key)
        .unwrap()?;
    detail
        .metrics
        .iter()
        .find(|m| m.kind == kind)
        .map(|m| m.value.get())
}

/// Two namespaces, `Alpha` (symbols 81, 82) and `Beta` (83, 84); `Run` calls `Go`.
#[cfg(test)]
fn namespaces() -> codeskimr_storage::SqliteStore {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let mut scan = data(header(SNAPSHOT, ScanMode::Full));
    scan.files = vec![
        file(SNAPSHOT, 71, "alpha.cs", "h-alpha"),
        file(SNAPSHOT, 72, "beta.cs", "h-beta"),
    ];
    scan.symbols = vec![
        symbol(81, 71, SymbolKind::Namespace, "Alpha", None),
        symbol(82, 71, SymbolKind::Method, "Alpha.Run", None),
        symbol(83, 72, SymbolKind::Namespace, "Beta", None),
        symbol(84, 72, SymbolKind::Method, "Beta.Go", None),
    ];
    scan.references = vec![reference(91, 71, 82, 84)];
    scan.metrics = vec![
        metric(81, MetricKind::EfferentCoupling, 1.0),
        metric(83, MetricKind::AfferentCoupling, 1.0),
    ];
    store.persist(scan).unwrap();
    store
}

#[test]
fn the_coupling_follows_the_new_references() {
    let store = namespaces();
    let ce = |symbol| coupling_of(&store, symbol, MetricKind::EfferentCoupling);
    let ca = |symbol| coupling_of(&store, symbol, MetricKind::AfferentCoupling);
    assert_eq!((ce(81), ca(83)), (Some(1.0), Some(1.0)));
    store
        .replace_references(replacement(&[71], Vec::new()))
        .unwrap();
    assert_eq!((ce(81), ca(83)), (Some(0.0), Some(0.0)));
    assert_eq!(
        coupling_of(&store, 81, MetricKind::Instability),
        None,
        "a unit without a dependency has no instability"
    );
}

#[test]
fn a_verified_reference_adds_the_dependency() {
    let store = namespaces();
    store
        .replace_references(replacement(&[72], vec![verified(900, 72, 84, 82)]))
        .unwrap();
    let value = |symbol, kind| coupling_of(&store, symbol, kind);
    assert_eq!(value(83, MetricKind::EfferentCoupling), Some(1.0));
    assert_eq!(value(81, MetricKind::Instability), Some(0.5));
}
