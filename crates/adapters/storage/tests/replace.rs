//! Level 2: replacing the references of some files of the newest snapshot (ADR 0033).

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    Level2Replacement, ProjectRepository, SnapshotData, SnapshotStore, SymbolKey, UsageIndex,
};
use codeskimr_domain::{
    Direction, FileId, MetricKind, Reference, ScanMode, SnapshotId, Symbol, SymbolId, SymbolKind,
    SymbolOrigin, UsageQuery,
};
use common::{PROJECT, code_of, data, file, header, metric, project, reference, store, symbol};

const SNAPSHOT: u64 = 10;

/// A full scan with the files `alpha.cs` (71) and `beta.cs` (72).
#[cfg(test)]
fn two_files() -> SnapshotData {
    let mut scan = data(header(SNAPSHOT, ScanMode::Full));
    scan.files = vec![
        file(SNAPSHOT, 71, "alpha.cs", "h-alpha"),
        file(SNAPSHOT, 72, "beta.cs", "h-beta"),
    ];
    scan
}

/// `a.cs` holds `A` and `Run`; `b.cs` holds `B` and `Go`. `Run` calls `B` (level 1).
#[cfg(test)]
fn scan() -> SnapshotData {
    let mut scan = two_files();
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
fn replacement(files: &[u64], references: Vec<Reference>) -> Level2Replacement {
    Level2Replacement {
        snapshot: SnapshotId::new(SNAPSHOT),
        files: files.iter().copied().map(FileId::new).collect(),
        symbols: Vec::new(),
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
        .replace_level2(replacement(&[71], vec![verified(900, 71, 82, 84)]))
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
        .replace_level2(replacement(&[71], vec![verified(900, 71, 82, 84)]))
        .unwrap();
    assert_eq!((usages(83), usages(84)), (0, 1));
}

#[test]
fn a_replacement_without_references_removes_those_of_the_file() {
    let store = registered();
    store
        .replace_level2(replacement(&[71], Vec::new()))
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
        let error = store.replace_level2(bad).unwrap_err();
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
    let replacement = Level2Replacement {
        snapshot: SnapshotId::new(20),
        ..replacement(&[71], vec![verified(900, 71, 82, 84)])
    };
    store.replace_level2(replacement).unwrap();
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
        .replace_level2(replacement(&[71], Vec::new()))
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
    let mut scan = two_files();
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
        .replace_level2(replacement(&[71], Vec::new()))
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
        .replace_level2(replacement(&[72], vec![verified(900, 72, 84, 82)]))
        .unwrap();
    let value = |symbol, kind| coupling_of(&store, symbol, kind);
    assert_eq!(value(83, MetricKind::EfferentCoupling), Some(1.0));
    assert_eq!(value(81, MetricKind::Instability), Some(0.5));
}

/// The ids of the symbols that use `symbol` directly in the newest snapshot.
#[cfg(test)]
fn users_of(store: &codeskimr_storage::SqliteStore, symbol: u64) -> Vec<u64> {
    let query = UsageQuery {
        symbol: SymbolId::new(symbol),
        direction: Direction::Incoming,
        depth: 1,
        include_tests: true,
    };
    let page = store.usages(SnapshotId::new(SNAPSHOT), &query, 10).unwrap();
    page.entries.iter().map(|e| e.from.id.get()).collect()
}

#[test]
fn a_kept_usage_index_follows_the_replacement() {
    let store = namespaces();
    assert_eq!(users_of(&store, 84), [82]);
    store
        .replace_level2(replacement(&[71], vec![verified(900, 71, 82, 83)]))
        .unwrap();
    assert_eq!(users_of(&store, 84), Vec::<u64>::new());
    assert_eq!(users_of(&store, 83), [82]);
}

/// A symbol that level 2 created (ADR 0055).
#[cfg(test)]
fn created(id: u64, file: u64, kind: SymbolKind, qualified: &str, parent: Option<u64>) -> Symbol {
    Symbol {
        origin: SymbolOrigin::Index,
        visibility: None,
        signature: Some("pub struct Gen".to_owned()),
        ..symbol(id, file, kind, qualified, parent)
    }
}

/// `Lib.Gen` and its variant, created in `alpha.cs`, and a use of the variant from `Go`.
#[cfg(test)]
fn with_created() -> Level2Replacement {
    Level2Replacement {
        symbols: vec![
            created(301, 71, SymbolKind::Enum, "Lib.Gen", Some(81)),
            created(302, 71, SymbolKind::EnumMember, "Lib.Gen.One", Some(301)),
        ],
        ..replacement(&[71], vec![verified(900, 72, 84, 302)])
    }
}

#[cfg(test)]
fn symbols_of_index(store: &codeskimr_storage::SqliteStore) -> Vec<(u64, Option<u64>)> {
    let mut found: Vec<_> = store
        .latest_index(PROJECT)
        .unwrap()
        .symbols
        .iter()
        .filter(|s| s.origin == SymbolOrigin::Index)
        .map(|s| (s.id.get(), s.parent.map(SymbolId::get)))
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn the_symbols_of_level_two_are_stored_with_their_origin_and_counted_as_usages() {
    let store = registered();
    // The reference stands in `beta.cs`, which is not replaced, so it is not allowed: only the
    // files of the replacement may hold references.
    let error = store.replace_level2(with_created()).unwrap_err();
    assert_eq!(code_of(&error), "STO-901");
    let allowed = Level2Replacement {
        files: vec![FileId::new(71), FileId::new(72)],
        ..with_created()
    };
    let snapshot = store.replace_level2(allowed).unwrap();
    assert_eq!(
        symbols_of_index(&store),
        [(301, Some(81)), (302, Some(301))]
    );
    assert_eq!(snapshot.stats.symbols, 6);
    assert_eq!(
        store
            .usage_count(SnapshotId::new(SNAPSHOT), SymbolId::new(302))
            .unwrap(),
        1
    );
    let index = store.latest_index(PROJECT).unwrap();
    let stored = index.symbols.iter().find(|s| s.id.get() == 302).unwrap();
    assert_eq!(stored.origin, SymbolOrigin::Index);
    assert_eq!(stored.qualified_name.as_str(), "Lib.Gen.One");
}

#[test]
fn a_second_replacement_replaces_the_created_symbols_and_the_references_to_them() {
    let store = registered();
    let both = |symbols: Vec<Symbol>, references: Vec<Reference>| Level2Replacement {
        symbols,
        ..replacement(&[71, 72], references)
    };
    store
        .replace_level2(both(
            vec![created(301, 71, SymbolKind::Struct, "Lib.Old", Some(81))],
            vec![verified(900, 72, 84, 301)],
        ))
        .unwrap();
    // Only `alpha.cs` is replaced again: its created symbol is gone, and so is the reference
    // from `beta.cs` that pointed at it.
    store
        .replace_level2(Level2Replacement {
            symbols: vec![created(310, 71, SymbolKind::Struct, "Lib.New", Some(81))],
            ..replacement(&[71], Vec::new())
        })
        .unwrap();
    assert_eq!(symbols_of_index(&store), [(310, Some(81))]);
    // Level 1 references 91 and 92 went with the first replacement of their files; 900 went with
    // its target.
    assert_eq!(stored(&store), Vec::<(u64, bool)>::new());
}

#[test]
fn a_replacement_with_bad_symbols_changes_nothing() {
    let store = registered();
    let bad = |symbols: Vec<Symbol>| Level2Replacement {
        symbols,
        ..replacement(&[71], Vec::new())
    };
    let syntax = symbol(301, 71, SymbolKind::Class, "Lib.G", Some(81));
    let other_file = created(301, 72, SymbolKind::Class, "Lib.G", Some(83));
    let no_parent = created(301, 71, SymbolKind::Class, "Lib.G", Some(999));
    for symbols in [vec![syntax], vec![other_file], vec![no_parent]] {
        let error = store.replace_level2(bad(symbols)).unwrap_err();
        assert_eq!(code_of(&error), "STO-901");
    }
    assert_eq!(symbols_of_index(&store), []);
    assert_eq!(stored(&store), [(91, false), (92, false)]);
}

/// What the newest snapshot says about the direct users of `symbol`: `(from, verified)`.
#[cfg(test)]
fn verified_users_of(
    store: &codeskimr_storage::SqliteStore,
    snapshot: u64,
    symbol: u64,
) -> Vec<(u64, bool)> {
    let query = UsageQuery {
        symbol: SymbolId::new(symbol),
        direction: Direction::Incoming,
        depth: 1,
        include_tests: true,
    };
    let page = store.usages(SnapshotId::new(snapshot), &query, 10).unwrap();
    page.entries
        .iter()
        .map(|e| (e.from.id.get(), e.reference.verified))
        .collect()
}

/// Level 2 ran, then the UI rescanned (an incremental scan, level 1 only): the verified usage of an
/// unchanged file stays verified in the newest snapshot, and a usage that level 2 never checked
/// stays unchecked.
#[test]
fn a_later_incremental_scan_keeps_the_verified_usages_of_unchanged_files() {
    let store = registered();
    store
        .replace_level2(replacement(&[71], vec![verified(900, 71, 82, 83)]))
        .unwrap();
    assert_eq!(verified_users_of(&store, SNAPSHOT, 83), [(82, true)]);
    let mut next = data(header(20, ScanMode::Incremental));
    next.files = vec![file(20, 73, "gamma.cs", "h-gamma")];
    store.persist(next).unwrap();
    assert_eq!(verified_users_of(&store, 20, 83), [(82, true)]);
    // The reference of `beta.cs` was never checked by level 2.
    assert_eq!(verified_users_of(&store, 20, 81), [(84, false)]);
}

#[test]
fn a_store_that_kept_the_index_sees_what_another_connection_replaced() {
    // The server answers queries from one store and keeps the index of the snapshot; a scan of
    // the browser, or `codeskimr scan` in another process, writes through its own store.
    let db = common::TempDb::new();
    let serving = codeskimr_storage::SqliteStore::open(&db.0, std::num::NonZeroU32::MIN).unwrap();
    serving.save_project(&project(1, "/work/app", &[])).unwrap();
    serving.persist(scan()).unwrap();
    assert_eq!(users_of(&serving, 83), [82], "the index is built and kept");
    let writer = codeskimr_storage::SqliteStore::open(&db.0, std::num::NonZeroU32::MIN).unwrap();
    writer
        .replace_level2(replacement(&[71], vec![verified(900, 71, 82, 84)]))
        .unwrap();
    assert_eq!(users_of(&serving, 83), Vec::<u64>::new());
    assert_eq!(users_of(&serving, 84), [82]);
}
