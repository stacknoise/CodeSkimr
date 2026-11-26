//! The comparison of two scans and its impact (ADR 0097), through the path the server takes.

use std::fs;

use serde_json::{Value, json};

use super::tests::Fixture;

const CALLER: &str = "namespace Demo;\n\npublic class Caller\n{\n    public int Run() { return new Calculator().Add(1, 2); }\n}\n";
const CHANGED: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b, int c) { return a + b + c; }\n}\n";

/// A project with two scans: the second one gives `Calculator.Add` another parameter list, and the
/// untouched class `Caller` still calls it.
fn two_scans(fixture: &Fixture) -> u64 {
    fs::write(fixture.root.path().join("app/Caller.cs"), CALLER).unwrap();
    let id = fixture.scanned();
    fs::write(fixture.root.path().join("app/Calculator.cs"), CHANGED).unwrap();
    fixture
        .call("projects.rescan", json!({ "project": id }))
        .unwrap();
    id
}

fn ask(fixture: &Fixture, command: &str, input: Value) -> Result<Value, &'static str> {
    fixture.call(command, input)
}

#[test]
fn the_comparison_lists_the_symbols_without_the_usages() {
    let fixture = Fixture::installed(None);
    let id = two_scans(&fixture);
    let answer = ask(&fixture, "scans.compare", json!({ "project": id })).unwrap();
    assert_eq!(answer["schema"], 1);
    assert!(answer["base"]["snapshot"].as_u64() < answer["target"]["snapshot"].as_u64());
    assert_eq!(answer["added"]["total"], 1);
    assert_eq!(answer["removed"]["total"], 1);
    assert_eq!(
        answer["removed"]["items"][0]["qualified_name"],
        "Demo.Calculator.Add"
    );
    assert_eq!(answer["impact"], Value::Null);
    assert_eq!(answer["direct_usages_only"], true);
    assert_eq!(answer["note_key"], "compare-direct-only-note");
}

#[test]
fn the_impact_names_the_usage_that_was_not_touched() {
    let fixture = Fixture::installed(None);
    let id = two_scans(&fixture);
    let answer = ask(&fixture, "scans.impact", json!({ "project": id })).unwrap();
    let symbol = &answer["impact"]["items"][0];
    assert_eq!(symbol["cause"], "removed");
    assert_eq!(symbol["symbol"]["qualified_name"], "Demo.Calculator.Add");
    let usage = &symbol["usages"]["items"][0];
    assert_eq!(usage["from"]["qualified_name"], "Demo.Caller.Run");
    assert_eq!(usage["status"], "not_touched");
    assert_eq!(symbol["not_touched"], 1);
    assert_eq!(answer["summary"]["usages_not_touched"], 1);
}

#[test]
fn two_scans_can_be_named() {
    let fixture = Fixture::installed(None);
    let id = two_scans(&fixture);
    let scans = ask(&fixture, "scans.list", json!({ "project": id })).unwrap();
    let rows = scans["scans"].as_array().unwrap();
    let (newest, oldest) = (&rows[0]["snapshot"], &rows[1]["snapshot"]);
    let named = ask(
        &fixture,
        "scans.compare",
        json!({ "project": id, "base": oldest, "target": newest }),
    )
    .unwrap();
    assert_eq!(&named["base"]["snapshot"], oldest);
    assert_eq!(&named["target"]["snapshot"], newest);
}

#[test]
fn a_base_that_is_not_older_is_refused_with_its_code() {
    let fixture = Fixture::installed(None);
    let id = two_scans(&fixture);
    let scans = ask(&fixture, "scans.list", json!({ "project": id })).unwrap();
    let newest = scans["scans"][0]["snapshot"].clone();
    let refused = ask(
        &fixture,
        "scans.compare",
        json!({ "project": id, "base": newest, "target": newest }),
    );
    assert_eq!(refused.unwrap_err(), "CMP-101");
}

#[test]
fn a_project_with_one_scan_has_nothing_to_compare() {
    let fixture = Fixture::installed(None);
    let id = fixture.scanned();
    let refused = ask(&fixture, "scans.impact", json!({ "project": id }));
    assert_eq!(refused.unwrap_err(), "CMP-201");
}

#[test]
fn an_unknown_project_is_refused() {
    let fixture = Fixture::installed(None);
    assert!(ask(&fixture, "scans.compare", json!({ "project": 999 })).is_err());
}
