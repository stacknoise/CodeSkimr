//! The screen "Rules" through the path the server takes (ADR 0104).

use std::fs;

use serde_json::{Value, json};

use super::stats_tests::project;
use super::tests::Fixture;

const FORBID: &str = "[[rules.forbid]]\nfrom = \"Demo.Shop\"\nto = \"Demo.Calculator\"\nreason = \"shop must not count\"\n\n[[rules.forbid]]\nfrom = \"Demo.Calculator\"\nto = \"Demo.Shop\"\n";

fn show(fixture: &Fixture, id: u64) -> Value {
    fixture
        .call("rules.show", json!({ "project": id }))
        .unwrap()
}

#[test]
fn a_forbidden_use_is_shown_with_every_place_and_a_kept_rule_has_none() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let app = fixture.root.path().join("app");
    fs::write(app.join(".codeskimr.toml"), FORBID).unwrap();
    let answer = show(&fixture, id);
    assert_eq!(answer["scanned"], true);
    assert_eq!(
        answer["new"], 2,
        "the call and the parameter type: {answer}"
    );
    let [broken, kept] = answer["rules"].as_array().unwrap().as_slice() else {
        panic!("two rules: {answer}");
    };
    assert_eq!(broken["kind"], "forbid");
    assert_eq!(broken["reason"], "shop must not count");
    let violation = broken["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|violation| violation["to"] == "Demo.Calculator.Add")
        .unwrap();
    assert_eq!(violation["from"], "Demo.Shop.Total");
    let lines: Vec<_> = violation["places"]
        .as_array()
        .unwrap()
        .iter()
        .map(|place| place["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [7, 9]);
    assert_eq!(violation["places"][0]["path"], "Shop.cs");
    assert_eq!(kept["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn a_held_state_makes_the_violation_an_old_burden() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let app = fixture.root.path().join("app");
    fs::write(app.join(".codeskimr.toml"), FORBID).unwrap();
    let held = "held_on = \"2026-11-26\"\n\n[[entries]]\nrule = \"forbid:Demo.Shop>Demo.Calculator\"\nfrom = \"Demo.Shop.Total\"\nto = \"Demo.Calculator.Add\"\n\n[[entries]]\nrule = \"forbid:Demo.Shop>Demo.Calculator\"\nfrom = \"Demo.Shop.Gone\"\nto = \"Demo.Calculator.Add\"\n";
    fs::write(app.join(".codeskimr-baseline.toml"), held).unwrap();
    let answer = show(&fixture, id);
    assert_eq!(answer["held_on"], "2026-11-26");
    assert_eq!(answer["old"], 1);
    assert_eq!(answer["new"], 1, "the parameter type is not held: {answer}");
    assert_eq!(answer["resolved"], 1);
    let held = answer["rules"][0]["violations"].as_array().unwrap();
    let old: Vec<_> = held.iter().filter(|v| v["old"] == true).collect();
    assert_eq!(old.len(), 1);
    assert_eq!(old[0]["to"], "Demo.Calculator.Add");
}

#[test]
fn a_project_without_rules_shows_none_and_a_broken_file_fails() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    assert_eq!(show(&fixture, id)["rules"], json!([]));
    let app = fixture.root.path().join("app");
    fs::write(
        app.join(".codeskimr.toml"),
        "[[rules.forbid]]\nfrom = \"A\"\nto = \"A.B\"\n",
    )
    .unwrap();
    let failed = fixture.call("rules.show", json!({ "project": id }));
    assert!(failed.is_err(), "{failed:?}");
}
