//! Registered projects and the file list of the last snapshot.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{ProjectRepository, SnapshotStore};
use codeskimr_domain::{
    BranchName, CredentialId, GitUrl, Principal, ProjectId, ProjectSource, ScanMode,
};
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
fn a_git_source_keeps_the_name_of_its_credential_and_nothing_else() {
    let store = store(3);
    let mut saved = project(1, "/work/app", &[]);
    for credential in [Some(CredentialId::new("work").unwrap()), None] {
        saved.source = ProjectSource::Git {
            url: GitUrl::new("https://example.org/app.git").unwrap(),
            branch: BranchName::new("main").unwrap(),
            credential,
        };
        store.save_project(&saved).unwrap();
        assert_eq!(store.project(PROJECT).unwrap(), saved);
    }
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

#[test]
fn project_is_found_by_its_source() {
    let store = store(3);
    let saved = project(1, "/work/app", &[("Api", &["App.Api"])]);
    store.save_project(&saved).unwrap();
    assert_eq!(store.project_by_source(&saved.source).unwrap(), Some(saved));
    let other = project(2, "/work/other", &[]);
    assert_eq!(store.project_by_source(&other.source).unwrap(), None);
}

#[test]
fn next_project_id_follows_the_highest_registered_id() {
    let store = store(3);
    assert_eq!(store.next_project_id().unwrap(), ProjectId::new(1));
    store.save_project(&project(4, "/work/app", &[])).unwrap();
    assert_eq!(store.next_project_id().unwrap(), ProjectId::new(5));
}

#[test]
fn consent_for_level_2_starts_empty_and_round_trips() {
    use codeskimr_application::level2::{ConsentRecord, ConsentStore, confirm, revoke};

    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.consent(PROJECT).unwrap(), ConsentRecord::default());
    let confirmed = ConsentRecord {
        enabled: true,
        confirmed: true,
    };
    confirm(&Principal::local_owner(), &store, PROJECT).unwrap();
    assert_eq!(store.consent(PROJECT).unwrap(), confirmed);
    // A second project does not share it.
    assert_eq!(
        store.consent(ProjectId::new(2)).unwrap(),
        ConsentRecord::default()
    );
    revoke(&Principal::local_owner(), &store, PROJECT).unwrap();
    assert_eq!(store.consent(PROJECT).unwrap(), ConsentRecord::default());
}

#[test]
fn a_confirmation_does_not_survive_a_change_of_the_projects_source() {
    use codeskimr_application::level2::{ConsentStore, confirm};

    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let owner = Principal::local_owner();
    confirm(&owner, &store, PROJECT).unwrap();
    // The same id, pointed at another folder: not confirmed any more.
    store.save_project(&project(1, "/work/other", &[])).unwrap();
    assert!(!store.consent(PROJECT).unwrap().confirmed);
    // Pointed back, the old confirmation fits again only if nobody confirmed anew in between; a
    // new confirmation binds to the new place.
    confirm(&owner, &store, PROJECT).unwrap();
    assert!(store.consent(PROJECT).unwrap().confirmed);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert!(!store.consent(PROJECT).unwrap().confirmed);
    // A Git branch counts too.
    let mut git = project(1, "/work/app", &[]);
    git.source = ProjectSource::Git {
        url: GitUrl::new("https://example.org/app.git").unwrap(),
        branch: BranchName::new("main").unwrap(),
        credential: None,
    };
    store.save_project(&git).unwrap();
    confirm(&owner, &store, PROJECT).unwrap();
    assert!(store.consent(PROJECT).unwrap().confirmed);
    git.source = ProjectSource::Git {
        url: GitUrl::new("https://example.org/app.git").unwrap(),
        branch: BranchName::new("evil").unwrap(),
        credential: None,
    };
    store.save_project(&git).unwrap();
    assert!(!store.consent(PROJECT).unwrap().confirmed);
}

#[test]
fn consent_for_a_project_that_does_not_exist_is_refused() {
    use codeskimr_application::level2::{ConsentRecord, ConsentStore, confirm};

    let store = store(3);
    let error = confirm(&Principal::local_owner(), &store, ProjectId::new(7)).unwrap_err();
    assert_eq!(code_of(&error), "PRJ-201");
    assert_eq!(
        store.consent(ProjectId::new(7)).unwrap(),
        ConsentRecord::default()
    );
}
