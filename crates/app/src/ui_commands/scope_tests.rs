//! `scope.get` and `scope.set`: what a project leaves out, and that a scan keeps to it.

use std::fs;

use serde_json::{Value, json};

use super::tests::Fixture;
use crate::principal;

const PYTHON: &str = "def hello():\n    return 1\n";
const OLD: &str = "namespace Old;\n\npublic class Legacy\n{\n}\n";

/// The demo folder with a Python tool, and an old C# class in `legacy/`.
fn with_more_code(fixture: &Fixture) {
    let app = fixture.root.path().join("app");
    fs::create_dir_all(app.join("tools")).unwrap();
    fs::write(app.join("tools/job.py"), PYTHON).unwrap();
    fs::create_dir_all(app.join("legacy/deep")).unwrap();
    fs::write(app.join("legacy/Old.cs"), OLD).unwrap();
    fs::write(app.join("legacy/deep/Older.cs"), OLD).unwrap();
}

fn files_of(fixture: &Fixture, project: u64) -> u64 {
    let list = fixture.call("projects.list", Value::Null).unwrap();
    let row = list
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == project)
        .unwrap();
    row["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|language| language["files"].as_u64().unwrap())
        .sum()
}

fn folder<'a>(answer: &'a Value, path: &str) -> &'a Value {
    answer["folders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|folder| folder["path"] == path)
        .unwrap()
}

#[test]
fn a_new_project_leaves_nothing_out() {
    let fixture = Fixture::installed(None);
    with_more_code(&fixture);
    let project = fixture.scanned();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(answer["live"], true);
    assert_eq!(answer["scope"], json!({ "languages": [], "folders": [] }));
    assert_eq!(answer["languages"][0]["language"], "csharp");
    assert_eq!(answer["languages"][0]["files"], 3);
    assert_eq!(answer["languages"][0]["covered"], true);
    assert_eq!(folder(&answer, "legacy")["files"], 2);
    assert_eq!(folder(&answer, "legacy")["covered"], true);
    assert_eq!(
        folder(&answer, "tools")["languages"][0]["language"],
        "python"
    );
}

#[test]
fn the_next_scan_keeps_to_the_scope() {
    let fixture = Fixture::installed(None);
    with_more_code(&fixture);
    let project = fixture.scanned();
    assert_eq!(files_of(&fixture, project), 4);
    let saved = fixture
        .call(
            "scope.set",
            json!({ "project": project, "languages": ["python"], "folders": ["legacy/", "legacy/deep"] }),
        )
        .unwrap();
    assert_eq!(saved["changed"], true);
    assert_eq!(
        saved["scope"],
        json!({ "languages": ["python"], "folders": ["legacy"] })
    );
    fixture
        .call(
            "projects.rescan",
            json!({ "project": project, "level2": false }),
        )
        .unwrap();
    assert_eq!(files_of(&fixture, project), 1);
}

#[test]
fn what_is_left_out_stays_on_the_page_so_it_can_be_taken_back() {
    let fixture = Fixture::installed(None);
    with_more_code(&fixture);
    let project = fixture.scanned();
    fixture
        .call(
            "scope.set",
            json!({ "project": project, "languages": ["python"], "folders": ["legacy"] }),
        )
        .unwrap();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    let python = answer["languages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|language| language["language"] == "python")
        .unwrap();
    assert_eq!(python["covered"], false);
    assert_eq!(folder(&answer, "legacy")["left_out"], true);
    assert_eq!(folder(&answer, "legacy")["covered"], false);
    assert_eq!(folder(&answer, "tools")["covered"], true);
}

#[test]
fn the_same_scope_again_is_no_change() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let input = json!({ "project": project, "languages": [], "folders": ["docs"] });
    assert_eq!(
        fixture.call("scope.set", input.clone()).unwrap()["changed"],
        true
    );
    assert_eq!(fixture.call("scope.set", input).unwrap()["changed"], false);
}

#[test]
fn a_folder_that_leaves_the_project_is_refused() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    for folder in [
        "",
        "..",
        "../other",
        "/etc",
        "C:/Windows",
        "a/../b",
        "a//b",
        ".",
    ] {
        let refused = fixture.call(
            "scope.set",
            json!({ "project": project, "languages": [], "folders": [folder] }),
        );
        assert_eq!(refused, Err("PRJ-103"), "{folder:?}");
    }
    let refused = fixture.call(
        "scope.set",
        json!({ "project": project, "languages": ["no good"], "folders": [] }),
    );
    assert_eq!(refused, Err("PRJ-103"));
}

#[test]
fn an_unknown_project_and_a_reader_are_refused() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let input = json!({ "project": 99, "languages": [], "folders": [] });
    assert_eq!(fixture.call("scope.set", input), Err("PRJ-201"));
    assert_eq!(
        fixture.call("scope.get", json!({ "project": 99 })),
        Err("PRJ-201")
    );
    let refused = fixture.call_as(
        principal::reader(),
        "scope.set",
        json!({ "project": project, "languages": [], "folders": [] }),
    );
    assert_eq!(refused, Err("AUTH-401"));
}
