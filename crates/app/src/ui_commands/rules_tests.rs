//! The screen "Rules" through the path the server takes (ADR 0104).

use std::fs;

use std::num::NonZeroU32;

use codeskimr_domain::{FolderPath, Project, ProjectId, ProjectName, ProjectSource};
use codeskimr_storage::SqliteStore;
use serde_json::{Value, json};

use super::stats_tests::project;
use super::tests::Fixture;

const FORBID: &str = "[[rules.forbid]]\nfrom = \"Demo.Shop\"\nto = \"Demo.Calculator\"\nreason = \"shop must not count\"\n\n[[rules.forbid]]\nfrom = \"Demo.Calculator\"\nto = \"Demo.Shop\"\n";

/// A scanned project whose project file holds `rules`.
fn with_rules(rules: &str) -> (Fixture, u64) {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    fs::write(
        fixture.root.path().join("app").join(".codeskimr.toml"),
        rules,
    )
    .unwrap();
    (fixture, id)
}

fn show(fixture: &Fixture, id: u64) -> Value {
    fixture
        .call("rules.show", json!({ "project": id }))
        .unwrap()
}

#[test]
fn a_forbidden_use_is_shown_with_every_place_and_a_kept_rule_has_none() {
    let (fixture, id) = with_rules(FORBID);
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
    let (fixture, id) = with_rules(FORBID);
    let app = fixture.root.path().join("app");
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
fn a_project_without_rules_shows_none() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    assert_eq!(show(&fixture, id)["rules"], json!([]));
}

#[test]
fn a_broken_project_file_or_held_state_fails_with_its_code() {
    let (fixture, id) = with_rules("[[rules.forbid]]\nfrom = \"A\"\nto = \"A.B\"\n");
    let call = || fixture.call("rules.show", json!({ "project": id }));
    assert_eq!(call(), Err("CFG-101"));
    let app = fixture.root.path().join("app");
    fs::write(app.join(".codeskimr.toml"), FORBID).unwrap();
    fs::write(app.join(".codeskimr-baseline.toml"), "held_on = [").unwrap();
    assert_eq!(call(), Err("CFG-102"));
}

#[test]
fn a_project_without_a_scan_is_not_judged_but_says_whether_cycles_are_denied() {
    let fixture = Fixture::installed(None);
    let app = fixture.root.path().join("app");
    fs::write(app.join(".codeskimr.toml"), "[rules]\ncycles = \"deny\"\n").unwrap();
    let store = SqliteStore::open(&fixture.locations.database(), NonZeroU32::MIN).unwrap();
    let project = Project {
        id: ProjectId::new(7),
        name: ProjectName::new("fresh").unwrap(),
        source: ProjectSource::Folder(FolderPath::new(fixture.folder()).unwrap()),
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    let answer = show(&fixture, 7);
    assert_eq!(answer["scanned"], false);
    assert_eq!(answer["cycles"], true);
    assert_eq!(answer["rules"], json!([]));
}
