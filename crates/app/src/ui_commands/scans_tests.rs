//! The list of the last scans (ADR 0097), through the path the server takes.

use std::fs;

use serde_json::{Value, json};

use super::tests::Fixture;

fn scans(fixture: &Fixture, project: u64) -> Value {
    fixture
        .call("scans.list", json!({ "project": project }))
        .unwrap()
}

#[test]
fn a_scanned_project_lists_its_scan_with_the_duration() {
    let fixture = Fixture::installed(None);
    let id = fixture.scanned();
    let answer = scans(&fixture, id);
    let rows = answer["scans"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["mode"], "full");
    assert!(rows[0]["duration_ms"].is_u64(), "the scan is timed");
    assert_eq!(
        rows[0]["files_changed"],
        Value::Null,
        "the first scan has none"
    );
    assert!(answer["keep"].as_u64().unwrap() > 0);
    assert!(answer["average_ms"].is_u64());
}

#[test]
fn a_changed_file_adds_a_scan_that_counts_it() {
    let fixture = Fixture::installed(None);
    let id = fixture.scanned();
    fs::write(
        fixture.root.path().join("app").join("Extra.cs"),
        "namespace Demo;\n\npublic class Extra { }\n",
    )
    .unwrap();
    fixture
        .call("projects.rescan", json!({ "project": id }))
        .unwrap();
    let rows = scans(&fixture, id)["scans"].as_array().unwrap().clone();
    assert_eq!(rows.len(), 2, "newest first");
    assert_eq!(rows[0]["mode"], "incremental");
    assert_eq!(rows[0]["files_changed"], 1);
}

#[test]
fn an_unknown_project_is_refused() {
    let fixture = Fixture::installed(None);
    assert!(
        fixture
            .call("scans.list", json!({ "project": 999 }))
            .is_err()
    );
}
