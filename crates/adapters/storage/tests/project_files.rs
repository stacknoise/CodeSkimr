//! One database file per project and a catalog (ADR 0095): routing, removal, notes and the lock
//! on the clean-up of files that the catalog does not list.
#![cfg(test)]

mod common;

use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

use codeskimr_application::ids::{first_snapshot_of, project_of_raw};
use codeskimr_application::ports::{
    AnnotationCreation, AnnotationFilter, AnnotationStore, CodeIndex, CodeOutline, DatabaseState,
    ProjectCatalog, ProjectRemoval, SnapshotStore,
};
use codeskimr_domain::{
    AnnotationContent, AnnotationKind, AnnotationLimits, AnnotationTarget, Author, FolderPath,
    NewAnnotation, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, ScanMode,
    SnapshotId, SymbolKind, Timestamp,
};
use codeskimr_storage::SqliteStore;
use common::{TempDb, code_of, data, file, header, symbol};

const LIMITS: AnnotationLimits = AnnotationLimits {
    max_text_chars: 100,
    max_title_chars: 20,
};

fn open(catalog: &Path) -> SqliteStore {
    SqliteStore::open(catalog, NonZeroU32::new(3).unwrap()).unwrap()
}

fn register(store: &SqliteStore, folder: &str) -> Project {
    let name = ProjectName::new(format!("project {folder}")).unwrap();
    let source = ProjectSource::Folder(FolderPath::new(folder).unwrap());
    store.register_project(name, source).unwrap()
}

/// A full scan of `project` with the class `App.Order`; the snapshot id carries the project number.
fn scan(store: &SqliteStore, project: &Project, round: u64) -> SnapshotId {
    let snapshot = first_snapshot_of(project.id) + round;
    let mut header = header(snapshot, ScanMode::Full);
    header.project = project.id;
    let mut scan = data(header);
    let (file_id, symbol_id) = (2 + round * 10, 3 + round * 10);
    scan.files = vec![file(
        snapshot,
        file_id,
        "src/order.cs",
        &format!("h{round}"),
    )];
    scan.symbols = vec![symbol(
        symbol_id,
        file_id,
        SymbolKind::Class,
        "App.Order",
        None,
    )];
    store.persist(scan).unwrap();
    SnapshotId::new(snapshot)
}

/// Two registered projects with one scan each.
fn two_scanned(store: &SqliteStore) -> (Project, Project) {
    let (first, second) = (register(store, "/work/a"), register(store, "/work/b"));
    scan(store, &first, 0);
    scan(store, &second, 0);
    (first, second)
}

fn note(project: ProjectId, text: &str) -> AnnotationCreation {
    AnnotationCreation {
        annotation: NewAnnotation {
            project,
            target: AnnotationTarget {
                kind: SymbolKind::Class,
                name: QualifiedName::new("App.Order").unwrap(),
                parameters: Vec::new(),
            },
            author: Author::new("anton").unwrap(),
            content: AnnotationContent {
                kind: AnnotationKind::Note,
                title: None,
                text: Some(LIMITS.text(text).unwrap()),
                status: None,
                due: None,
                assignee: None,
            },
        },
        now: Timestamp(1_000),
    }
}

fn listed(store: &SqliteStore, project: ProjectId) -> usize {
    let filter = AnnotationFilter {
        project,
        target: None,
        kind: None,
    };
    store.list_annotations(&filter).unwrap().len()
}

fn project_file(catalog: &Path, project: &Project) -> std::path::PathBuf {
    let folder = catalog.parent().unwrap().join("projects");
    folder.join(format!("{}.db", project.id.get()))
}

#[test]
fn every_project_has_a_file_of_its_own_and_the_list_says_where_and_how_large() {
    let db = TempDb::new();
    let store = open(&db.0);
    let (first, second) = two_scanned(&store);
    assert_ne!(project_file(&db.0, &first), project_file(&db.0, &second));
    let overviews = store.overviews().unwrap();
    assert_eq!(overviews.len(), 2);
    for (overview, project) in overviews.iter().zip([&first, &second]) {
        let info = overview
            .database
            .as_ref()
            .expect("a store with files says where");
        assert_eq!(Path::new(&info.path), project_file(&db.0, project));
        assert_eq!(info.state, DatabaseState::Ok);
        assert!(info.size_bytes > 0);
    }
}

#[test]
fn removing_a_project_deletes_its_file_and_nothing_of_the_others() {
    let db = TempDb::new();
    let store = open(&db.0);
    let (gone, kept) = (register(&store, "/work/a"), register(&store, "/work/b"));
    let gone_snapshot = scan(&store, &gone, 0);
    let kept_snapshot = scan(&store, &kept, 0);
    store.create_annotation(&note(gone.id, "x")).unwrap();
    store.create_annotation(&note(kept.id, "y")).unwrap();
    assert!(project_file(&db.0, &gone).exists());

    store.remove_project(gone.id).unwrap();

    assert!(!project_file(&db.0, &gone).exists());
    assert_eq!(store.overviews().unwrap().len(), 1);
    assert_eq!(
        code_of(&store.latest_snapshot(gone.id).unwrap_err()),
        "PRJ-201"
    );
    assert_eq!(
        code_of(&store.outline(gone_snapshot).unwrap_err()),
        "PRJ-201"
    );
    assert_eq!(store.outline(kept_snapshot).unwrap().len(), 1);
    assert_eq!(listed(&store, kept.id), 1);
    assert_eq!(listed(&store, gone.id), 0, "its notes went with its file");
}

#[test]
fn a_snapshot_id_leads_to_the_file_of_its_project() {
    let db = TempDb::new();
    let store = open(&db.0);
    let (first, second) = (register(&store, "/work/a"), register(&store, "/work/b"));
    let in_first = scan(&store, &first, 0);
    let in_second = scan(&store, &second, 0);
    assert_eq!(
        codeskimr_application::ids::project_of_snapshot(in_first),
        first.id
    );
    assert_eq!(
        codeskimr_application::ids::project_of_snapshot(in_second),
        second.id
    );
    assert_eq!(
        store.latest_snapshot(first.id).unwrap().unwrap().id,
        in_first
    );
    assert_eq!(
        store.latest_snapshot(second.id).unwrap().unwrap().id,
        in_second
    );
}

#[test]
fn notes_stay_with_their_project_and_a_new_scan_does_not_touch_them() {
    let db = TempDb::new();
    let store = open(&db.0);
    let (first, second) = two_scanned(&store);
    let in_first = store.create_annotation(&note(first.id, "one")).unwrap();
    let in_second = store.create_annotation(&note(second.id, "two")).unwrap();
    assert_eq!(
        project_of_raw(in_first.id.get()),
        first.id,
        "the id names the project"
    );
    assert_eq!(project_of_raw(in_second.id.get()), second.id);
    assert_eq!(
        (listed(&store, first.id), listed(&store, second.id)),
        (1, 1)
    );

    scan(&store, &first, 1);

    assert_eq!(listed(&store, first.id), 1);
    store.delete_annotation(in_first.id).unwrap();
    assert_eq!(
        (listed(&store, first.id), listed(&store, second.id)),
        (0, 1)
    );
}

#[test]
fn a_lost_catalog_does_not_cost_the_project_files() {
    let db = TempDb::new();
    let first = {
        let store = open(&db.0);
        let project = register(&store, "/work/a");
        scan(&store, &project, 0);
        project
    };
    for suffix in ["", "-wal", "-shm"] {
        let mut name = db.0.as_os_str().to_owned();
        name.push(suffix);
        let _ = fs::remove_file(name);
    }

    let store = open(&db.0);

    assert_eq!(store.overviews().unwrap().len(), 0);
    assert!(
        project_file(&db.0, &first).exists(),
        "the file is left alone, not deleted"
    );
}

#[test]
fn a_file_of_another_installation_is_left_alone() {
    let db = TempDb::new();
    drop(open(&db.0));
    let foreign = db.0.parent().unwrap().join("projects").join("1.db");
    fs::create_dir_all(foreign.parent().unwrap()).unwrap();
    fs::write(&foreign, b"not a database of this installation").unwrap();

    let store = open(&db.0);

    assert!(foreign.exists());
    assert_eq!(store.overviews().unwrap().len(), 0);
}

#[test]
fn the_database_of_an_earlier_version_is_not_opened_and_not_deleted() {
    let db = TempDb::new();
    let old = db.0.with_file_name("codeskimr.db");
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    fs::write(&old, b"the shared database of 1.0.0-rc.11").unwrap();

    let store = open(&db.0);

    assert_eq!(store.overviews().unwrap().len(), 0);
    assert!(old.exists());
}

#[test]
fn a_project_registered_twice_is_the_same_project() {
    let db = TempDb::new();
    let store = open(&db.0);
    let first = register(&store, "/work/a");
    let again = register(&store, "/work/a");
    assert_eq!(first.id, again.id);
    assert_eq!(store.overviews().unwrap().len(), 1);
    let numbers: Vec<u64> = store
        .overviews()
        .unwrap()
        .iter()
        .map(|overview| overview.project.id.get())
        .collect();
    assert_eq!(numbers, vec![first.id.get()]);
}
