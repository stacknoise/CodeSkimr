//! The screen "Rules" through the path the server takes (ADR 0104).

use std::fs;
use std::num::NonZeroU32;

use serde_json::{Value, json};

use super::stats_tests::project;
use super::tests::Fixture;

const LAYERS: &str = "[[layers]]\nname = \"shop\"\nnamespaces = [\"Demo.Shop\"]\nmay_use = []\n\n[[layers]]\nname = \"calc\"\nnamespaces = [\"Demo.Calculator\"]\n";
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
    fixture.registered(7);
    let answer = show(&fixture, 7);
    assert_eq!(answer["scanned"], false);
    assert_eq!(answer["cycles"], true);
    assert_eq!(answer["rules"], json!([]));
}

fn hold(fixture: &Fixture, id: u64) -> Result<Value, &'static str> {
    fixture.call("rules.hold", json!({ "project": id }))
}

#[test]
fn holding_the_state_writes_every_violation_as_an_old_burden() {
    let (fixture, id) = with_rules(FORBID);
    assert_eq!(show(&fixture, id)["holdable"], true);
    assert_eq!(show(&fixture, id)["new"], 2);
    let held = hold(&fixture, id).unwrap();
    assert_eq!(held["entries"], 2);
    let day = held["held_on"].as_str().unwrap();
    assert_eq!(day.len(), 10, "{day}");
    let file =
        fs::read_to_string(fixture.root.path().join("app/.codeskimr-baseline.toml")).unwrap();
    assert!(file.contains(day), "{file}");
    assert!(
        file.contains("rule = \"forbid:Demo.Shop>Demo.Calculator\""),
        "{file}"
    );
    let after = show(&fixture, id);
    assert_eq!(after["new"], 0);
    assert_eq!(after["old"], 2);
    assert_eq!(after["held_on"], day);
    assert_eq!(after["resolved"], 0);
}

#[test]
fn holding_again_replaces_the_file_and_drops_entries_that_are_gone() {
    let (fixture, id) = with_rules(FORBID);
    let app = fixture.root.path().join("app");
    let stale = "held_on = \"2026-01-01\"\n\n[[entries]]\nrule = \"forbid:Demo.Shop>Demo.Calculator\"\nfrom = \"Demo.Shop.Gone\"\nto = \"Demo.Calculator.Add\"\n";
    fs::write(app.join(".codeskimr-baseline.toml"), stale).unwrap();
    assert_eq!(show(&fixture, id)["resolved"], 1);
    hold(&fixture, id).unwrap();
    let file = fs::read_to_string(app.join(".codeskimr-baseline.toml")).unwrap();
    assert!(!file.contains("Gone"), "{file}");
    assert!(!file.contains("2026-01-01"), "{file}");
    assert_eq!(show(&fixture, id)["resolved"], 0);
}

#[test]
fn the_layers_are_the_first_rule_and_can_be_held() {
    let (fixture, id) = with_rules(LAYERS);
    let answer = show(&fixture, id);
    let first = &answer["rules"][0];
    assert_eq!(first["kind"], "layers");
    assert_eq!(first["id"], "layers");
    assert_eq!(
        first["allowed"],
        json!([{ "layer": "shop", "may_use": [] }])
    );
    assert!(answer["new"].as_u64().unwrap() > 0, "{answer}");
    hold(&fixture, id).unwrap();
    assert_eq!(show(&fixture, id)["new"], 0);
}

#[test]
fn only_a_scanned_folder_project_can_hold_and_only_with_the_right_to_manage() {
    let fixture = Fixture::installed(None);
    fixture.registered(7);
    assert_eq!(hold(&fixture, 7), Err("MCP-203"));
    let (fixture, id) = with_rules(FORBID);
    let refused = fixture.call_as(
        crate::principal::reader(),
        "rules.hold",
        json!({ "project": id }),
    );
    assert!(refused.is_err());
    assert!(
        !fixture
            .root
            .path()
            .join("app/.codeskimr-baseline.toml")
            .exists()
    );
}

#[test]
fn a_git_project_cannot_hold_and_is_not_holdable() {
    let fixture = Fixture::installed(None);
    let store =
        codeskimr_storage::SqliteStore::open(&fixture.locations.database(), NonZeroU32::MIN)
            .unwrap();
    let project = codeskimr_domain::Project {
        id: codeskimr_domain::ProjectId::new(7),
        name: codeskimr_domain::ProjectName::new("remote").unwrap(),
        source: codeskimr_domain::ProjectSource::Git {
            url: codeskimr_domain::GitUrl::new("https://example.org/demo.git").unwrap(),
            branch: codeskimr_domain::BranchName::new("main").unwrap(),
            credential: None,
        },
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    assert_eq!(show(&fixture, 7)["holdable"], false);
    assert_eq!(hold(&fixture, 7), Err("PRJ-101"));
}

#[test]
fn a_broken_old_state_is_replaced_not_read() {
    let (fixture, id) = with_rules(FORBID);
    let file = fixture.root.path().join("app/.codeskimr-baseline.toml");
    fs::write(&file, "held_on = [").unwrap();
    let broken = fixture.call("rules.show", json!({ "project": id }));
    assert_eq!(broken, Err("CFG-102"));
    hold(&fixture, id).unwrap();
    assert_eq!(show(&fixture, id)["new"], 0);
}
