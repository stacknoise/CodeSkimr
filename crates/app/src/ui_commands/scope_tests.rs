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

const CONFIG: &str =
    "<DATA><STARTING_FOLDER>/code/</STARTING_FOLDER><FOLDER_LOGIC>FULL</FOLDER_LOGIC></DATA>";
const CLASS: &str = "CLASS zcl_a DEFINITION PUBLIC.\nENDCLASS.\nCLASS zcl_a IMPLEMENTATION.\n  METHOD run.\n  ENDMETHOD.\nENDCLASS.\n";

#[test]
fn an_abapgit_folder_is_recognised_and_counted() {
    let fixture = Fixture::installed(None);
    let app = fixture.root.path().join("app");
    fs::create_dir_all(app.join("code/sub")).unwrap();
    fs::write(app.join(".abapgit.xml"), CONFIG).unwrap();
    fs::write(app.join("code/zcl_a.clas.abap"), CLASS).unwrap();
    fs::write(app.join("code/zcl_a.clas.xml"), "<x/>").unwrap();
    fs::write(app.join("code/sub/zcl_b.clas.abap"), CLASS).unwrap();
    fs::write(
        app.join("code/sub/zif_c.intf.abap"),
        "INTERFACE zif_c PUBLIC.\nENDINTERFACE.\n",
    )
    .unwrap();
    let project = fixture.scanned();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    let abapgit = &answer["abapgit"];
    assert_eq!(abapgit["starting_folder"], "code");
    assert_eq!(abapgit["folder_logic"], "full");
    assert_eq!(abapgit["configured"], true);
    assert_eq!(abapgit["kinds"][0], json!({ "kind": "CLAS", "objects": 2 }));
    assert_eq!(abapgit["kinds"][1], json!({ "kind": "INTF", "objects": 1 }));
    assert_eq!(abapgit["packages"], 2);
    assert_eq!(abapgit["source_files"], 3);
    let plain = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    assert!(
        plain["languages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["language"] == "abap")
    );
}

#[test]
fn a_folder_without_abapgit_has_none() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(answer["abapgit"], Value::Null);
}

/// The path of the demo folder as the window "New project" would send it.
fn typed(fixture: &Fixture) -> String {
    fixture
        .root
        .path()
        .join("app")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn a_folder_is_inspected_before_it_is_a_project() {
    let fixture = Fixture::installed(None);
    let app = fixture.root.path().join("app");
    fs::create_dir_all(app.join("code")).unwrap();
    fs::write(app.join(".abapgit.xml"), CONFIG).unwrap();
    fs::write(app.join("code/zcl_a.clas.abap"), CLASS).unwrap();
    let answer = fixture
        .call("scope.inspect", json!({ "source": typed(&fixture) }))
        .unwrap();
    assert_eq!(answer["abapgit"]["starting_folder"], "code");
    assert_eq!(answer["abapgit"]["folder_logic"], "full");
    assert_eq!(
        answer["abapgit"]["kinds"][0],
        json!({ "kind": "CLAS", "objects": 1 })
    );
    assert_eq!(answer["abapgit"]["packages"], 1);
}

#[test]
fn text_that_is_no_abapgit_folder_gets_an_empty_answer() {
    let fixture = Fixture::installed(None);
    for source in [
        typed(&fixture),
        "https://example.com/team/repo.git".to_owned(),
        "no such folder".to_owned(),
        String::new(),
    ] {
        let answer = fixture
            .call("scope.inspect", json!({ "source": source }))
            .unwrap();
        assert_eq!(answer, json!({ "abapgit": null }));
    }
}

#[test]
fn a_reader_may_not_inspect_a_folder() {
    let fixture = Fixture::installed(None);
    let refused = fixture.call_as(
        principal::reader(),
        "scope.inspect",
        json!({ "source": typed(&fixture) }),
    );
    assert_eq!(refused, Err("AUTH-401"));
}

#[test]
fn the_explorer_shows_a_package_relative_to_the_starting_folder() {
    let fixture = Fixture::installed(None);
    let app = fixture.root.path().join("app");
    fs::create_dir_all(app.join("code/sub")).unwrap();
    fs::write(app.join(".abapgit.xml"), CONFIG).unwrap();
    fs::write(app.join("code/zcl_a.clas.abap"), CLASS).unwrap();
    fs::write(app.join("code/sub/zcl_b.clas.abap"), CLASS).unwrap();
    let project = fixture.scanned();
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    let names: Vec<&str> = outline["layers"][0]["namespaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|space| space["name"].as_str().unwrap())
        .collect();
    // The starting folder `code` is the root package, `code/sub` is `sub`; "Demo" is the C# file
    // of the fixture.
    assert_eq!(names, ["", "Demo", "sub"]);
}

#[test]
fn abap_files_outside_an_abapgit_layout_are_reported_as_not_recognised() {
    let fixture = Fixture::installed(None);
    let plain = fixture.scanned();
    let answer = fixture
        .call("scope.get", json!({ "project": plain }))
        .unwrap();
    assert_eq!(answer["abapgit_unrecognised"], false);

    let app = fixture.root.path().join("app");
    fs::create_dir_all(app.join("lib")).unwrap();
    fs::write(app.join("lib/zcl_a.clas.abap"), CLASS).unwrap();
    let project = fixture.scanned();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(answer["abapgit"], serde_json::Value::Null);
    assert_eq!(answer["abapgit_unrecognised"], true);

    fs::write(app.join(".abapgit.xml"), CONFIG).unwrap();
    let answer = fixture
        .call("scope.get", json!({ "project": project }))
        .unwrap();
    assert_eq!(answer["abapgit_unrecognised"], false);
}
