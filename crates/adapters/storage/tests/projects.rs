//! Registered projects and the file list of the last snapshot.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{ProjectRepository, SnapshotStore};
use codeskimr_domain::{ProjectId, ScanMode};
use common::{PROJECT, code_of, data, file, header, project, store};

#[test]
fn project_with_layers_round_trips() {
    let store = store(3);
    let saved = project(
        1,
        "/work/app",
        &[("Api", &["App.Api", "App.Web"]), ("Core", &["App.Core"])],
    );
    store.save_project(&saved).unwrap();
    assert_eq!(store.project(PROJECT).unwrap(), saved);
}

#[test]
fn saving_again_updates_name_and_layers() {
    let store = store(3);
    store
        .save_project(&project(1, "/work/app", &[("Api", &["App.Api"])]))
        .unwrap();
    let renamed = project(1, "/work/app", &[("Core", &["App.Core"])]);
    store.save_project(&renamed).unwrap();
    assert_eq!(store.project(PROJECT).unwrap(), renamed);
}

#[test]
fn unknown_project_is_not_found() {
    let error = store(3).project(ProjectId::new(9)).unwrap_err();
    assert_eq!(code_of(&error), "PRJ-201");
}

#[test]
fn second_project_with_the_same_source_is_rejected() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let error = store
        .save_project(&project(2, "/work/app", &[]))
        .unwrap_err();
    assert_eq!(code_of(&error), "PRJ-301");
}

#[test]
fn latest_files_are_empty_before_the_first_snapshot() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.latest_files(PROJECT).unwrap(), Vec::new());
}

#[test]
fn latest_files_come_from_the_newest_snapshot() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    for (snapshot, file_id, hash) in [(10, 11, "old"), (20, 21, "new")] {
        let mut scan = data(header(snapshot, ScanMode::Full));
        scan.files.push(file(snapshot, file_id, "src/a.cs", hash));
        store.persist(scan).unwrap();
    }
    let files = store.latest_files(PROJECT).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].content_hash.as_str(), "new");
}

#[test]
fn persisting_for_an_unknown_project_fails_and_writes_nothing() {
    let store = store(3);
    let error = store.persist(data(header(10, ScanMode::Full))).unwrap_err();
    assert_eq!(code_of(&error), "PRJ-201");
    assert!(store.latest_snapshot(PROJECT).unwrap().is_none());
}
