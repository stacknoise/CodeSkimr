//! Changing the rules of a project through the path the server takes (ADR 0112).

use std::fs;

use serde_json::{Value, json};

use super::stats_tests::project;
use super::tests::Fixture;

const OLD: &str = "# my own note\n[[layers]]\nname = \"api\"\nnamespaces = [\"Demo.Shop\"]\n\n[[layers]]\nname = \"core\"\nnamespaces = [\"Demo.Calculator\"]\n";

fn edit(id: u64, forbid: &Value, limits: &Value) -> Value {
    json!({ "project": id, "cycles": true, "forbid": forbid, "public_api": [], "limits": limits })
}

fn file(fixture: &Fixture) -> std::path::PathBuf {
    fixture.root.path().join("app").join(".codeskimr.toml")
}

fn fixture_with(text: &str) -> (Fixture, u64) {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    fs::write(file(&fixture), text).unwrap();
    (fixture, id)
}

#[test]
fn setting_rules_keeps_comments_and_layers_and_shows_in_the_screen() {
    let (fixture, id) = fixture_with(OLD);
    let forbid = json!([{ "from": "Demo.Shop", "to": "Demo.Calculator", "reason": "no" }]);
    let limits = json!([{ "layer": "api", "may_use": ["core"] }]);
    let saved = fixture
        .call("rules.set", edit(id, &forbid, &limits))
        .unwrap();
    assert_eq!(saved["rules"], 2);
    let text = fs::read_to_string(file(&fixture)).unwrap();
    assert!(text.starts_with("# my own note"), "{text}");
    assert!(text.contains("may_use = [\"core\"]") && text.contains("cycles = \"deny\""));
    let shown = fixture
        .call("rules.show", json!({ "project": id }))
        .unwrap();
    assert_eq!(shown["cycles"], true);
    assert!(
        shown["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["kind"] == "forbid")
    );
}

#[test]
fn a_preview_judges_without_writing() {
    let (fixture, id) = fixture_with(OLD);
    let forbid = json!([{ "from": "Demo.Shop", "to": "Demo.Calculator" }]);
    let shown = fixture
        .call("rules.preview", edit(id, &forbid, &json!([])))
        .unwrap();
    let rule = shown["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rule| rule["kind"] == "forbid")
        .unwrap();
    assert_ne!(rule["violations"], json!([]));
    assert_eq!(fs::read_to_string(file(&fixture)).unwrap(), OLD);
}

#[test]
fn invalid_rules_fail_with_their_code_and_leave_the_file_alone() {
    let (fixture, id) = fixture_with(OLD);
    let inside = json!([{ "from": "A", "to": "A.B" }]);
    assert_eq!(
        fixture.call("rules.set", edit(id, &inside, &json!([]))),
        Err("CFG-101")
    );
    let unknown = json!([{ "layer": "gone", "may_use": [] }]);
    let result = fixture.call("rules.set", edit(id, &json!([]), &unknown));
    assert!(
        result.is_ok(),
        "a limit for a layer that is not there is ignored: {result:?}"
    );
    let missing = json!([{ "layer": "api", "may_use": ["nowhere"] }]);
    assert_eq!(
        fixture.call("rules.set", edit(id, &json!([]), &missing)),
        Err("CFG-101")
    );
}

#[test]
fn repeated_edits_keep_comments_and_layers() {
    let (fixture, id) = fixture_with(OLD);
    let handles: Vec<_> = (0..6)
        .map(|n| {
            let forbid = json!([{ "from": format!("A{n}"), "to": format!("B{n}") }]);
            fixture.call("rules.set", edit(id, &forbid, &json!([])))
        })
        .collect();
    assert!(handles.iter().all(Result::is_ok));
    let text = fs::read_to_string(file(&fixture)).unwrap();
    assert!(text.contains("[[layers]]") && text.contains("# my own note"));
}

#[test]
fn too_many_rules_in_one_call_are_refused() {
    let (fixture, id) = fixture_with(OLD);
    let many: Vec<Value> = (0..501)
        .map(|n| json!({ "from": format!("A{n}"), "to": format!("B{n}") }))
        .collect();
    assert_eq!(
        fixture.call("rules.preview", edit(id, &Value::Array(many), &json!([]))),
        Err("CFG-101")
    );
}
