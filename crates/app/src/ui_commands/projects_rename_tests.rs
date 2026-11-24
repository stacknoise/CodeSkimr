//! `projects.rename`, `projects.suggest_name` and the name of `projects.add`.

use serde_json::{Value, json};

use super::tests::Fixture;
use crate::principal;

fn name_of(fixture: &Fixture, id: u64) -> String {
    let list = fixture.call("projects.list", Value::Null).unwrap();
    let row = list
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap();
    row["name"].as_str().unwrap().to_owned()
}

#[test]
fn a_renamed_project_keeps_its_id_and_shows_the_new_name() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let renamed = fixture
        .call(
            "projects.rename",
            json!({ "project": project, "name": "  Kundenportal " }),
        )
        .unwrap();
    assert_eq!(renamed["name"], "Kundenportal");
    assert_eq!(name_of(&fixture, project), "Kundenportal");
}

#[test]
fn an_empty_name_goes_back_to_the_name_of_the_source() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let original = name_of(&fixture, project);
    fixture
        .call(
            "projects.rename",
            json!({ "project": project, "name": "Other" }),
        )
        .unwrap();
    fixture
        .call(
            "projects.rename",
            json!({ "project": project, "name": " " }),
        )
        .unwrap();
    assert_eq!(name_of(&fixture, project), original);
}

#[test]
fn a_name_with_control_characters_or_too_long_is_refused() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    // A line break, and a direction override that could turn the name around on the screen.
    for name in [
        "two\nlines".to_owned(),
        "a\u{202E}b".to_owned(),
        "x".repeat(81),
    ] {
        let refused = fixture.call(
            "projects.rename",
            json!({ "project": project, "name": name }),
        );
        assert_eq!(refused, Err("PRJ-102"));
    }
}

#[test]
fn a_reader_cannot_ask_for_a_suggestion() {
    let fixture = Fixture::installed(None);
    let refused = fixture.call_as(
        principal::reader(),
        "projects.suggest_name",
        json!({ "source": fixture.folder() }),
    );
    assert_eq!(refused, Err("AUTH-401"));
}

#[test]
fn renaming_an_unknown_project_fails_with_prj_201() {
    let fixture = Fixture::installed(None);
    let refused = fixture.call("projects.rename", json!({ "project": 99, "name": "x" }));
    assert_eq!(refused, Err("PRJ-201"));
}

#[test]
fn a_reader_cannot_rename() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let refused = fixture.call_as(
        principal::reader(),
        "projects.rename",
        json!({ "project": project, "name": "x" }),
    );
    assert_eq!(refused, Err("AUTH-401"));
}

#[test]
fn the_name_suggested_for_a_source_is_the_name_it_gets() {
    let fixture = Fixture::installed(None);
    let folder = fixture.root.path().join("shop").join("src");
    std::fs::create_dir_all(&folder).unwrap();
    let suggested = fixture
        .call(
            "projects.suggest_name",
            json!({ "source": folder.to_string_lossy() }),
        )
        .unwrap();
    assert_eq!(suggested["name"], "shop/src");
    let git = fixture
        .call(
            "projects.suggest_name",
            json!({ "source": "https://example.org/a/tool.git" }),
        )
        .unwrap();
    assert_eq!(git["name"], "tool");
    let nothing = fixture
        .call(
            "projects.suggest_name",
            json!({ "source": "no such thing" }),
        )
        .unwrap();
    assert_eq!(nothing["name"], Value::Null);
}

#[test]
fn a_chosen_name_at_adding_is_used_and_an_invalid_one_registers_nothing() {
    let fixture = Fixture::installed(None);
    let source = fixture.root.path().join("app");
    let refused = fixture.call(
        "projects.add",
        json!({ "source": source.to_string_lossy(), "name": "bad\u{7}name" }),
    );
    assert_eq!(refused, Err("PRJ-102"));
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert!(list.as_array().unwrap().is_empty());
    let added = fixture
        .call(
            "projects.add",
            json!({ "source": source.to_string_lossy(), "name": "Mein Projekt" }),
        )
        .unwrap();
    assert_eq!(
        name_of(&fixture, added["project"].as_u64().unwrap()),
        "Mein Projekt"
    );
}
