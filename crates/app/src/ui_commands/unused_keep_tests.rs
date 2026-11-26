//! "Keep on purpose" and "Lift" through the path the server takes (ADR 0101).

use std::fs;

use serde_json::{Value, json};

use super::stats_tests::{names, project, ranking};
use super::tests::Fixture;

const PROJECT_FILE: &str = ".codeskimr.toml";

fn keep(project: u64, symbol: &str) -> Value {
    json!({ "project": project, "symbol": symbol, "reason": "called by reflection (2026-11-26)" })
}

fn unused(fixture: &Fixture, project: u64) -> Value {
    ranking(
        fixture,
        project,
        &json!({ "ranking": "unused", "kinds": "functions" }),
    )
}

#[test]
fn a_kept_symbol_leaves_the_ranking_with_its_reason_and_comes_back_when_lifted() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let kept = fixture.call("unused.keep", keep(id, "Demo.Shop.Hidden"));
    assert_eq!(kept, Ok(json!({ "entries": 1 })));
    let file = fs::read_to_string(fixture.root.path().join("app").join(PROJECT_FILE)).unwrap();
    assert!(file.contains("[[unused.ignore]]") && file.contains("Demo.Shop.Hidden"));
    let after = unused(&fixture, id);
    assert!(!names(&after).contains(&"Hidden"), "{after}");
    let rule = &after["left_out"][0]["rule"];
    assert_eq!(rule["reason"], "called by reflection (2026-11-26)");
    let lift = json!({ "project": id, "symbols": rule["symbols"], "reason": rule["reason"] });
    assert_eq!(
        fixture.call("unused.lift", lift),
        Ok(json!({ "entries": 0 }))
    );
    assert!(names(&unused(&fixture, id)).contains(&"Hidden"));
}

#[test]
fn keeping_twice_writes_one_entry_and_the_other_tables_of_the_file_stay() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    fs::write(
        &path,
        "[[layers]]\nname = \"all\"\nnamespaces = [\"Demo\"]\n",
    )
    .unwrap();
    fixture
        .call("unused.keep", keep(id, "Demo.Shop.Hidden"))
        .unwrap();
    let again = fixture.call("unused.keep", keep(id, "Demo.Shop.Hidden"));
    assert_eq!(again, Ok(json!({ "entries": 1 })));
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("name = \"all\"")
    );
}

#[test]
fn an_entry_without_a_reason_or_with_a_wildcard_is_refused_and_the_file_stays_away() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let blank = json!({ "project": id, "symbol": "Demo.Shop.Hidden", "reason": "  " });
    assert_eq!(fixture.call("unused.keep", blank), Err("CFG-101"));
    assert_eq!(
        fixture.call("unused.keep", keep(id, "*.Hidden")),
        Err("CFG-101")
    );
    assert!(!fixture.root.path().join("app").join(PROJECT_FILE).exists());
}

#[test]
fn lifting_an_entry_that_is_not_there_and_unknown_projects_are_refused() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let lift = json!({ "project": id, "symbols": ["Demo.Shop.Hidden"], "reason": "x" });
    assert_eq!(fixture.call("unused.lift", lift), Err("CFG-101"));
    assert_eq!(fixture.call("unused.keep", keep(99, "A.B")), Err("PRJ-201"));
}

#[test]
fn a_file_that_is_not_toml_is_not_touched() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let path = fixture.root.path().join("app").join(PROJECT_FILE);
    fs::write(&path, "[broken").unwrap();
    assert_eq!(fixture.call("unused.keep", keep(id, "A.B")), Err("CFG-102"));
    assert_eq!(fs::read_to_string(path).unwrap(), "[broken");
}
