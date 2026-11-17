//! The layer editor commands through the same path the server takes (ADR 0065).

use std::fs;
use std::num::NonZeroU32;

use codeskimr_codes::Coded;
use codeskimr_domain::{Project, ProjectId};
use codeskimr_storage::SqliteStore;
use serde_json::{Value, json};

use super::tests::Fixture;
use crate::source_arg::classify;

const PROJECT_FILE: &str = ".codeskimr.toml";

fn demo_layer() -> Value {
    json!([{ "name": "calculation", "namespaces": ["Demo"] }])
}

#[test]
fn the_namespaces_of_the_scan_are_offered_without_a_layer() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let shown = fixture
        .call("layers.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(shown["editable"], true);
    assert_eq!(shown["scanned"], true);
    assert_eq!(shown["layers"], json!([]));
    assert_eq!(
        shown["namespaces"],
        json!([{ "name": "Demo", "types": 1, "layer": null }])
    );
}

#[test]
fn saved_layers_reach_the_file_and_apply_with_the_next_scan() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let input = json!({ "project": project, "layers": demo_layer() });
    assert_eq!(
        fixture.call("layers.set", input),
        Ok(json!({ "layers": 1 }))
    );
    let file = fs::read_to_string(fixture.root.path().join("app").join(PROJECT_FILE)).unwrap();
    assert!(file.contains("[[layers]]") && file.contains("calculation"));
    fixture
        .call("projects.rescan", json!({ "project": project }))
        .unwrap();
    let shown = fixture
        .call("layers.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(shown["layers"], demo_layer());
    assert_eq!(shown["namespaces"][0]["layer"], "calculation");
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["layers"][0]["name"], "calculation");
}

#[test]
fn invalid_layers_leave_the_file_alone() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let twice = json!([
        { "name": "a", "namespaces": ["Demo"] },
        { "name": "a", "namespaces": ["Other"] },
    ]);
    for layers in [twice, json!([{ "name": "a", "namespaces": [] }])] {
        let input = json!({ "project": project, "layers": layers });
        assert_eq!(fixture.call("layers.set", input), Err("CFG-101"));
    }
    assert!(!fixture.root.path().join("app").join(PROJECT_FILE).exists());
}

#[test]
fn a_file_that_is_not_toml_is_not_touched() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    fs::write(&path, "[broken").unwrap();
    let input = json!({ "project": project, "layers": demo_layer() });
    assert_eq!(fixture.call("layers.set", input), Err("CFG-102"));
    assert_eq!(fs::read_to_string(path).unwrap(), "[broken");
}

#[test]
fn a_git_project_has_no_file_to_write() {
    let fixture = Fixture::installed(None);
    let store = SqliteStore::open(&fixture.locations.database(), NonZeroU32::MIN).unwrap();
    let spec = classify("https://example.org/team/repo.git", None).unwrap();
    let project = Project {
        id: ProjectId::new(1),
        name: spec.name,
        source: spec.source,
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    let shown = fixture.call("layers.get", json!({ "project": 1 })).unwrap();
    assert_eq!(shown["editable"], false);
    assert_eq!(shown["scanned"], false);
    let input = json!({ "project": 1, "layers": demo_layer() });
    assert_eq!(fixture.call("layers.set", input), Err("PRJ-101"));
}

#[test]
fn an_unknown_project_is_refused() {
    let fixture = Fixture::installed(None);
    assert_eq!(
        fixture.call("layers.get", json!({ "project": 99 })),
        Err("PRJ-201")
    );
}

#[test]
fn a_result_over_the_size_limit_is_refused_and_the_file_stays() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    let before = format!("# {}\n", "x".repeat(1_048_000));
    fs::write(&path, &before).unwrap();
    let layers = json!([{ "name": "big", "namespaces": ["n".repeat(2_000)] }]);
    let input = json!({ "project": project, "layers": layers });
    assert_eq!(fixture.call("layers.set", input), Err("CFG-101"));
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn a_link_as_project_file_is_not_read_and_not_replaced() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let target = fixture.root.path().join("elsewhere.toml");
    fs::write(&target, "").unwrap();
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    std::os::unix::fs::symlink(&target, &path).unwrap();
    let input = json!({ "project": project, "layers": demo_layer() });
    assert_eq!(fixture.call("layers.set", input), Err("CFG-102"));
    assert!(
        fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[cfg(unix)]
#[test]
fn the_mode_of_the_old_file_is_kept() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    fs::write(&path, "").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let input = json!({ "project": project, "layers": demo_layer() });
    fixture.call("layers.set", input).unwrap();
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_file_that_cannot_be_written_is_cfg_104_and_leaves_nothing_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing-folder").join(PROJECT_FILE);
    let failure = super::layers::write_project_file(&path, "text").unwrap_err();
    assert_eq!(failure.code().entry().id, "CFG-104");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
