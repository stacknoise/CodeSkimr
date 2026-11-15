//! The names as written (issue #49): stored with the files, carried with them, read together with
//! the files and the index in one call.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    PendingReference, PreviousState, ProjectRepository, Receiver, SnapshotData, SnapshotStore,
    StoredNames,
};
use codeskimr_domain::{
    FileId, QualifiedName, ReferenceId, ReferenceKind, ScanMode, SnapshotId, SymbolId, SymbolKind,
};
use common::{PROJECT, data, file, header, reference, registered_store, span, symbol};

/// A call of `target` written in file `file` by symbol `from`, on line `line`.
#[cfg(test)]
fn name(
    file: u64,
    from: u64,
    line: u32,
    target: &str,
    receiver: Option<Receiver>,
) -> PendingReference {
    PendingReference {
        from: SymbolId::new(from),
        file: FileId::new(file),
        span: span(line),
        kind: ReferenceKind::Call,
        target: QualifiedName::new(target).unwrap(),
        receiver,
    }
}

/// All names of the newest snapshot.
#[cfg(test)]
fn all_names(store: &codeskimr_storage::SqliteStore) -> Vec<PendingReference> {
    let state = store.latest_state(PROJECT).unwrap();
    store
        .stored_names(state.snapshot.unwrap(), StoredNames::All)
        .unwrap()
}

/// One file `a.cs` (id 11) with a class and a method; the method writes `names`.
#[cfg(test)]
fn first_scan(names: Vec<PendingReference>) -> SnapshotData {
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "a.cs", "ha")];
    scan.symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.A", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Run", Some(21)),
    ];
    scan.references = vec![reference(31, 11, 22, 21)];
    scan.pending = names;
    scan
}

#[test]
fn without_a_snapshot_there_is_no_state_and_no_names() {
    let store = registered_store();
    assert_eq!(
        store.latest_state(PROJECT).unwrap(),
        PreviousState::default()
    );
}

#[test]
fn every_kind_of_receiver_survives_storing_and_loading() {
    let typed = QualifiedName::new("App.Typed").unwrap();
    let named = QualifiedName::new("Other.Named").unwrap();
    let receivers = [
        None,
        Some(Receiver::Typed(typed)),
        Some(Receiver::Named(named)),
        Some(Receiver::Own),
        Some(Receiver::Base),
        Some(Receiver::Unknown),
        Some(Receiver::Free),
    ];
    let written: Vec<PendingReference> = receivers
        .into_iter()
        .zip(1..)
        .map(|(receiver, line)| name(11, 22, line, "Target", receiver))
        .collect();
    let store = registered_store();
    store.persist(first_scan(written.clone())).unwrap();
    assert_eq!(all_names(&store), written);
    assert!(store.latest_state(PROJECT).unwrap().names_stored);
}

#[test]
fn the_state_holds_the_files_the_index_and_the_names_of_one_snapshot() {
    let store = registered_store();
    store
        .persist(first_scan(vec![name(11, 22, 1, "App.A", None)]))
        .unwrap();
    let state = store.latest_state(PROJECT).unwrap();
    assert_eq!(state.files, store.latest_files(PROJECT).unwrap());
    assert_eq!(state.index, store.latest_index(PROJECT).unwrap());
}

#[test]
fn an_incremental_scan_carries_the_names_of_unchanged_files() {
    let store = registered_store();
    let kept = name(11, 22, 1, "Gone", None);
    store.persist(first_scan(vec![kept.clone()])).unwrap();
    let mut second = data(header(40, ScanMode::Incremental));
    second.files = vec![file(40, 41, "b.cs", "hb")];
    second.symbols = vec![symbol(42, 41, SymbolKind::Class, "App.B", None)];
    let fresh = name(41, 42, 3, "App.A", None);
    second.pending = vec![fresh.clone()];
    store.persist(second).unwrap();
    assert_eq!(all_names(&store), vec![kept, fresh]);
    assert!(store.latest_state(PROJECT).unwrap().names_stored);
}

#[test]
fn the_names_of_a_removed_file_leave_with_it() {
    let store = registered_store();
    store
        .persist(first_scan(vec![name(11, 22, 1, "App.A", None)]))
        .unwrap();
    let mut second = data(header(40, ScanMode::Incremental));
    second.removed = vec![codeskimr_domain::RelativePath::new("a.cs").unwrap()];
    store.persist(second).unwrap();
    assert_eq!(all_names(&store), Vec::new());
}

#[test]
fn a_dropped_reference_is_not_in_the_new_snapshot_but_the_totals_follow() {
    let store = registered_store();
    store.persist(first_scan(Vec::new())).unwrap();
    let mut second = data(header(40, ScanMode::Incremental));
    second.dropped_references = vec![ReferenceId::new(31)];
    let snapshot = store.persist(second).unwrap();
    assert_eq!(snapshot.stats.references, 0);
    assert_eq!(store.latest_index(PROJECT).unwrap().references, Vec::new());
}

#[test]
fn names_are_read_by_kind_and_by_the_symbol_they_are_written_in() {
    let store = registered_store();
    let mut inherit = name(11, 21, 2, "Base", None);
    inherit.kind = ReferenceKind::Inherit;
    let mut import = name(11, 21, 3, "System", None);
    import.kind = ReferenceKind::Import;
    let call = name(11, 22, 4, "Target", Some(Receiver::Own));
    store
        .persist(first_scan(vec![
            inherit.clone(),
            import.clone(),
            call.clone(),
        ]))
        .unwrap();
    let snapshot = store.latest_state(PROJECT).unwrap().snapshot.unwrap();
    let read = |which: StoredNames<'_>| store.stored_names(snapshot, which).unwrap();
    let inside_run = [SymbolId::new(22)];
    assert_eq!(read(StoredNames::Bases), vec![inherit]);
    assert_eq!(
        read(StoredNames::ImportsOf(&[FileId::new(11)])),
        vec![import]
    );
    assert_eq!(read(StoredNames::ImportsOf(&[FileId::new(12)])), Vec::new());
    assert_eq!(
        read(StoredNames::Callers(&[FileId::new(11)], &inside_run)),
        vec![call]
    );
    assert_eq!(
        read(StoredNames::Callers(&[FileId::new(11)], &[])),
        Vec::new()
    );
    assert_eq!(
        read(StoredNames::Callers(&[FileId::new(12)], &inside_run)),
        Vec::new(),
        "the symbol is not in that file"
    );
}

#[test]
fn names_of_a_snapshot_that_does_not_exist_are_an_error_not_an_empty_answer() {
    let store = registered_store();
    let error = store
        .stored_names(SnapshotId::new(99), StoredNames::All)
        .unwrap_err();
    assert_eq!(common::code_of(&error), "STO-901");
}
