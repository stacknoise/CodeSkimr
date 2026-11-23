//! What a read does while a write holds the shared connection, as a scan does for as long as it
//! stores its snapshot: the reads of the screens answer at once with the state before the write,
//! the reads a write is built on wait for it and see what it wrote.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use codeskimr_application::ports::{ProjectCatalog, SnapshotData};
use codeskimr_domain::{
    FolderPath, Project, ProjectId, ProjectName, ProjectSource, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, Timestamp,
};

use crate::snapshots::write_snapshot;
use crate::store::SqliteStore;

/// Long enough for an answer that does not wait, far too short to sit out a test that hangs.
const ANSWER_IN: Duration = Duration::from_secs(10);
/// How long the waiting read is watched to see that it does not answer early.
const STILL_WAITING: Duration = Duration::from_millis(300);
const KEPT_SNAPSHOTS: u32 = 3;
const PROJECT: ProjectId = ProjectId::new(1);
const SNAPSHOT: SnapshotId = SnapshotId::new(5);

fn project() -> Project {
    Project {
        id: PROJECT,
        name: ProjectName::new("app").unwrap(),
        source: ProjectSource::Folder(FolderPath::new("/work/app").unwrap()),
        layers: Vec::new(),
    }
}

/// What a first scan of the project stores, without any file.
fn scan() -> SnapshotData {
    let snapshot = Snapshot {
        id: SNAPSHOT,
        project: PROJECT,
        commit: None,
        created: Timestamp(1_000),
        mode: ScanMode::Full,
        stats: SnapshotStats::default(),
    };
    SnapshotData {
        naming_rules: String::new(),
        ambiguous_places: None,
        snapshot,
        left_out: Vec::new(),
        warnings: Vec::new(),
        parse_micros: Vec::new(),
        dropped_references: Vec::new(),
        pending: Vec::new(),
        removed: Vec::new(),
        hashes: Vec::new(),
        fingerprints: Vec::new(),
        metrics: Vec::new(),
        references: Vec::new(),
        symbols: Vec::new(),
        files: Vec::new(),
    }
}

fn file_store(name: &str) -> (SqliteStore, PathBuf) {
    let folder =
        std::env::temp_dir().join(format!("codeskimr-during-{}-{name}", std::process::id()));
    let store = SqliteStore::open(
        &folder.join("workspace.db"),
        NonZeroU32::new(KEPT_SNAPSHOTS).unwrap(),
    )
    .unwrap();
    store.save_project(&project()).unwrap();
    (store, folder)
}

/// What `ask` answers while a write that stores the scan is held open, if it answers within
/// `patience`, and what it answers once the write is through, if it had not answered before.
fn asked_during_the_write<T: Send>(
    store: &SqliteStore,
    ask: impl FnOnce() -> T + Send,
    patience: Duration,
) -> (Option<T>, Option<T>) {
    let ((started, begun), (release, held)) = (channel(), channel::<()>());
    let (answer, answered) = channel();
    thread::scope(|scope| {
        scope.spawn(move || {
            let stored = store.write(|tx| {
                write_snapshot(tx, &scan(), KEPT_SNAPSHOTS)?;
                started.send(()).unwrap();
                held.recv().unwrap();
                Ok(())
            });
            stored.unwrap();
        });
        begun.recv_timeout(ANSWER_IN).unwrap();
        scope.spawn(move || answer.send(ask()));
        let during = answered.recv_timeout(patience).ok();
        release.send(()).unwrap();
        (during, answered.recv_timeout(ANSWER_IN).ok())
    })
}

#[test]
fn the_project_list_answers_during_a_write_with_the_state_before_it() {
    let (store, folder) = file_store("list");
    let (during, _) = asked_during_the_write(&store, || store.overviews().unwrap(), ANSWER_IN);
    let rows = during.expect("the list waited for the write");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].project.id, PROJECT);
    assert!(rows[0].latest.is_none());
    let newest = store.overviews().unwrap()[0]
        .latest
        .as_ref()
        .map(|scan| scan.id);
    assert_eq!(newest, Some(SNAPSHOT));
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn the_next_free_id_waits_for_the_write_and_counts_what_it_wrote() {
    let (store, folder) = file_store("id");
    assert_eq!(store.next_free_id().unwrap(), crate::consts::FIRST_ID);
    let (early, late) =
        asked_during_the_write(&store, || store.next_free_id().unwrap(), STILL_WAITING);
    assert_eq!(early, None, "the next free id did not wait for the write");
    assert!(late.unwrap() > SNAPSHOT.get());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn the_namespace_graph_answers_during_a_write_and_is_not_kept() {
    let (store, folder) = file_store("graph");
    let graph = || store.namespace_graph(SNAPSHOT).is_ok();
    let (during, _) = asked_during_the_write(&store, graph, ANSWER_IN);
    assert_eq!(during, Some(true), "the graph waited for the write");
    let version = store.data_version().unwrap();
    assert!(store.graphs.get(SNAPSHOT, version).is_none());
    std::fs::remove_dir_all(folder).unwrap();
}
