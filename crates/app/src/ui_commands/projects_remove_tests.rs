//! `projects.remove` through the same path the server takes, and its refusal while a scan runs.

use std::fs;
use std::num::NonZeroU32;
use std::sync::Arc;

use codeskimr_application::ports::{ProjectRepository, ProjectToolCheck};
use codeskimr_application::registry::CommandExt;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    BranchName, FolderPath, GitUrl, Project, ProjectId, ProjectName, ProjectSource,
};
use codeskimr_source::GitCache;
use codeskimr_storage::SqliteStore;
use serde_json::{Value, json};

use super::projects_remove::{RemoveInput, RemoveProject};
use super::projects_tests::{BrokenFirst, context};
use super::tests::Fixture;
use crate::principal;
use crate::running::RunningScans;

#[test]
fn a_removed_project_is_gone_from_the_list_and_its_files_stay() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let removed = fixture
        .call("projects.remove", json!({ "project": project }))
        .unwrap();
    assert_eq!(removed["project"], project);
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 0);
    assert!(fs::metadata(fixture.root.path().join("app/Calculator.cs")).is_ok());
    for id in ["projects.rescan", "projects.remove"] {
        let again = fixture.call(id, json!({ "project": project }));
        assert_eq!(again, Err("PRJ-201"), "{id}");
    }
}

#[test]
fn a_project_can_be_added_again_after_its_removal() {
    let fixture = Fixture::installed(None);
    let first = fixture.scanned();
    fixture
        .call("projects.remove", json!({ "project": first }))
        .unwrap();
    let second = fixture.scanned();
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], second);
}

#[test]
fn removing_a_git_project_deletes_its_clone_in_the_data_folder() {
    let fixture = Fixture::installed(None);
    let url = GitUrl::new("https://example.org/demo.git").unwrap();
    let branch = BranchName::new("main").unwrap();
    let store = SqliteStore::open(&fixture.locations.database(), NonZeroU32::MIN).unwrap();
    let project = Project {
        id: ProjectId::new(1),
        name: ProjectName::new("demo").unwrap(),
        source: ProjectSource::Git {
            url: url.clone(),
            branch: branch.clone(),
            credential: None,
        },
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    let cache = GitCache::new(fixture.locations.git_cache());
    let clone = cache.checkout_dir(&url, &branch);
    fs::create_dir_all(&clone).unwrap();
    fs::write(clone.join("lib.rs"), "fn one() {}").unwrap();
    fixture
        .call("projects.remove", json!({ "project": 1 }))
        .unwrap();
    assert!(!clone.exists());
}

#[test]
fn a_project_with_a_running_scan_is_not_removed() {
    let running = RunningScans::default();
    let store = Arc::new(SqliteStore::open_in_memory(NonZeroU32::MIN).unwrap());
    let project = store
        .register_project(
            ProjectName::new("demo").unwrap(),
            ProjectSource::Folder(FolderPath::new("/work/demo").unwrap()),
        )
        .unwrap();
    let tools: Arc<dyn ProjectToolCheck> = Arc::new(BrokenFirst);
    let command = RemoveProject {
        projects: store.clone(),
        removal: store.clone(),
        context: context(tools, running.clone()),
    };
    let input = RemoveInput {
        project: project.id.get(),
    };
    let scanning = running.start(project.id).unwrap();
    let refused = command.run(&principal::resolve(), input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::SnapshotInProgress);
    assert_eq!(store.project(project.id).unwrap(), project);
    drop(scanning);
    command.run(&principal::resolve(), input).unwrap();
    let gone = store.project(project.id).unwrap_err();
    assert_eq!(gone.code(), ErrorCode::ProjectNotFound);
    // The scan lock is released again.
    assert!(!running.is_running(project.id));
}

#[test]
fn a_caller_without_the_right_to_register_cannot_remove() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let reader = principal::reader();
    let refused = fixture.call_as(reader, "projects.remove", json!({ "project": project }));
    assert_eq!(refused, Err("AUTH-401"));
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
}
