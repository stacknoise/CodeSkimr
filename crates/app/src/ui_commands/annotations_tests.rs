//! The commands of annotations and alerts through the same path the server takes (ADR 0068): what
//! each caller may do (S-06), the limits of the texts, pages, summaries and the alert flow with
//! its event.

use std::fs;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use codeskimr_application::events::{AlertsChanged, Event};
use codeskimr_domain::{Principal, ProjectId};
use serde_json::{Value, json};

use super::tests::Fixture;
use crate::principal;

const CLASS: &str = "Demo.Calculator";
const METHOD: &str = "Demo.Calculator.Add";
const SOURCE_FILE: &str = "app/Calculator.cs";
const PATIENCE: Duration = Duration::from_millis(50);

fn class() -> Value {
    json!({ "kind": "class", "name": CLASS })
}

fn method() -> Value {
    json!({ "kind": "method", "name": METHOD })
}

fn note(project: u64, text: &str) -> Value {
    json!({ "project": project, "target": class(), "text": text })
}

fn task(project: u64, title: &str) -> Value {
    json!({ "project": project, "target": class(), "title": title })
}

fn annotate_only() -> Principal {
    principal::annotator()
}

/// The events that arrived, without waiting for more.
fn alerts_seen(events: &Receiver<Event>) -> Vec<AlertsChanged> {
    let mut seen = Vec::new();
    while let Ok(event) = events.recv_timeout(PATIENCE) {
        if let Event::AlertsChanged(changed) = event {
            seen.push(changed);
        }
    }
    seen
}

fn change_the_method(fixture: &Fixture) {
    let path = fixture.root.path().join(SOURCE_FILE);
    let text = fs::read_to_string(&path).unwrap();
    fs::write(path, text.replace("a + b", "a - b")).unwrap();
}

#[test]
fn a_note_is_written_listed_and_counted() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let created = fixture
        .call("annotations.add_note", note(project, "Rounds half up"))
        .unwrap();
    assert_eq!(created["kind"], "note");
    assert_eq!(created["text"], "Rounds half up");
    assert_eq!(created["author"], "local");
    assert_eq!(created["without_target"], false);
    assert_eq!(created["target"]["name"], CLASS);
    let listed = fixture
        .call(
            "annotations.list",
            json!({ "project": project, "symbol": class() }),
        )
        .unwrap();
    assert_eq!(listed["total"], 1);
    assert_eq!(listed["items"][0]["id"], created["id"]);
    let other = fixture
        .call(
            "annotations.list",
            json!({ "project": project, "symbol": method() }),
        )
        .unwrap();
    assert_eq!(other["total"], 0);
}

#[test]
fn the_summary_counts_marks_notes_and_tasks_per_symbol_in_one_call() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let on_class = |input: Value| fixture.call("annotations.create", input).unwrap();
    on_class(json!({ "project": project, "target": class(), "kind": "critical" }));
    on_class(json!({ "project": project, "target": class(), "kind": "note", "text": "a" }));
    on_class(json!({ "project": project, "target": class(), "kind": "note", "text": "b" }));
    let done = on_class(
        json!({ "project": project, "target": class(), "kind": "task",
        "title": "t1", "status": "done" }),
    );
    on_class(json!({ "project": project, "target": class(), "kind": "task", "title": "t2" }));
    fixture
        .call(
            "annotations.add_note",
            json!({ "project": project, "target": method(), "text": "c" }),
        )
        .unwrap();
    assert_eq!(done["status"], "done");
    let summary = fixture
        .call("annotations.summary", json!({ "project": project }))
        .unwrap();
    assert_eq!(summary["total"], 2);
    assert_eq!(
        summary["items"][0],
        json!({
            "target": { "kind": "class", "name": CLASS, "parameters": [] },
            "critical": true, "notes": 2, "tasks_done": 1, "tasks_total": 2,
            "without_target": false
        })
    );
    assert_eq!(summary["items"][1]["target"]["name"], METHOD);
    assert_eq!(summary["items"][1]["critical"], false);
}

#[test]
fn the_types_and_members_of_the_read_models_carry_the_key_of_their_annotations() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    let node = &outline["layers"][0]["namespaces"][0]["types"][0];
    assert_eq!(node["qualified_name"], CLASS);
    let shown = fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": node["id"] }),
        )
        .unwrap();
    assert_eq!(shown["qualified_name"], CLASS);
    assert_eq!(shown["members"][0]["qualified_name"], METHOD);
    assert_eq!(shown["members"][0]["kind"], "method");
}

#[test]
fn the_commands_with_a_shortcut_carry_it_and_fix_their_kind() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let shortcuts: Vec<_> = fixture
        .api
        .list(&principal::resolve())
        .into_iter()
        .filter(|info| info.id.starts_with("annotations.") || info.id.starts_with("alerts."))
        .map(|info| (info.id, info.shortcut, info.read_only))
        .collect();
    let by_key = |key: &str| {
        shortcuts
            .iter()
            .find(|(_, shortcut, _)| *shortcut == Some(key))
            .map(|(id, _, read_only)| (*id, *read_only))
    };
    assert_eq!(by_key("K"), Some(("annotations.mark_critical", false)));
    assert_eq!(by_key("N"), Some(("annotations.add_note", false)));
    assert_eq!(by_key("T"), Some(("annotations.add_task", false)));
    let contradiction = json!({ "project": project, "target": class(), "kind": "task",
        "text": "x" });
    assert_eq!(
        fixture.call("annotations.add_note", contradiction),
        Err("API-101")
    );
    let missing = json!({ "project": project, "target": class(), "text": "x" });
    assert_eq!(fixture.call("annotations.create", missing), Err("API-101"));
    let repeated = json!({ "project": project, "target": class(), "kind": "note", "text": "x" });
    assert!(fixture.call("annotations.add_note", repeated).is_ok());
}

#[test]
fn the_assignee_the_name_and_the_parameters_are_held_to_the_configured_limits() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let task = |extra: Value| {
        let mut base = json!({ "project": project, "target": class(), "title": "t" });
        base.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        fixture.call("annotations.add_task", base)
    };
    assert_eq!(task(json!({ "assignee": "a".repeat(101) })), Err("ANN-104"));
    let long_name = json!({ "kind": "class", "name": "n".repeat(501) });
    assert_eq!(task(json!({ "target": long_name })), Err("ANN-105"));
    let long_type = json!({ "kind": "class", "name": CLASS, "parameters": ["p".repeat(201)] });
    assert_eq!(task(json!({ "target": long_type })), Err("ANN-105"));
    let many = json!({ "kind": "class", "name": CLASS, "parameters": vec!["int"; 33] });
    assert_eq!(task(json!({ "target": many })), Err("ANN-105"));
    let created = task(json!({ "assignee": "a".repeat(100) })).unwrap();
    let update = |extra: Value| {
        let mut base = json!({ "project": project, "id": created["id"] });
        base.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        fixture.call("annotations.update", base)
    };
    assert_eq!(
        update(json!({ "assignee": "a".repeat(101) })),
        Err("ANN-104")
    );
    let moved = json!({ "kind": "class", "name": "n".repeat(501) });
    assert_eq!(update(json!({ "target": moved })), Err("ANN-105"));
}

#[test]
fn the_texts_are_held_to_the_configured_limits_and_the_fields_to_the_kind() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let too_long = "x".repeat(4_001);
    let long_title = "t".repeat(501);
    let exact = "y".repeat(4_000);
    let create = |input: Value| fixture.call("annotations.create", input);
    let on = |extra: Value| {
        let mut base = json!({ "project": project, "target": class() });
        base.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        base
    };
    assert_eq!(
        create(on(json!({ "kind": "note", "text": too_long }))),
        Err("ANN-102")
    );
    assert_eq!(
        create(on(json!({ "kind": "task", "title": long_title }))),
        Err("ANN-102")
    );
    assert_eq!(
        create(on(json!({ "kind": "note", "text": "   " }))),
        Err("ANN-103")
    );
    assert_eq!(create(on(json!({ "kind": "task" }))), Err("ANN-103"));
    assert_eq!(
        create(on(json!({ "kind": "note", "text": "x", "due": 5 }))),
        Err("ANN-103")
    );
    assert!(create(on(json!({ "kind": "note", "text": exact }))).is_ok());
    assert_eq!(
        fixture
            .call("annotations.list", json!({ "project": project }))
            .unwrap()["total"],
        1,
        "the refused ones left nothing behind"
    );
}

#[test]
fn a_symbol_has_one_critical_mark_and_must_be_in_the_newest_scan() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let mark = json!({ "project": project, "target": class() });
    assert!(
        fixture
            .call("annotations.mark_critical", mark.clone())
            .is_ok()
    );
    assert_eq!(
        fixture.call("annotations.mark_critical", mark),
        Err("ANN-301")
    );
    let gone = json!({ "project": project, "target": { "kind": "class", "name": "Demo.Gone" },
        "text": "x" });
    assert_eq!(fixture.call("annotations.add_note", gone), Err("ANN-101"));
    let unknown = json!({ "project": 99, "target": class(), "text": "x" });
    assert_eq!(
        fixture.call("annotations.add_note", unknown),
        Err("PRJ-201")
    );
    let bad_kind = json!({ "project": project, "target": { "kind": "widget", "name": CLASS },
        "text": "x" });
    assert_eq!(
        fixture.call("annotations.add_note", bad_kind),
        Err("API-101")
    );
}

#[test]
fn an_update_changes_what_is_named_clears_what_is_null_and_keeps_the_rest() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let created = fixture
        .call(
            "annotations.add_task",
            json!({ "project": project, "target": class(), "title": "Split it",
                "text": "in two", "assignee": "anton", "due": 100 }),
        )
        .unwrap();
    assert_eq!(created["status"], "open");
    let id = created["id"].clone();
    let updated = fixture
        .call(
            "annotations.update",
            json!({ "project": project, "id": id, "status": "done", "assignee": null }),
        )
        .unwrap();
    assert_eq!(updated["status"], "done");
    assert_eq!(updated["assignee"], Value::Null);
    assert_eq!(updated["text"], "in two");
    assert_eq!(updated["title"], "Split it");
    assert_eq!(updated["due"], 100);
    let refused = json!({ "project": project, "id": id, "status": "maybe" });
    assert_eq!(fixture.call("annotations.update", refused), Err("API-101"));
    let missing = json!({ "project": project, "id": 999, "status": "done" });
    assert_eq!(fixture.call("annotations.update", missing), Err("ANN-201"));
    let to_gone = json!({ "project": project, "id": id,
        "target": { "kind": "class", "name": "Demo.Gone" } });
    assert_eq!(fixture.call("annotations.update", to_gone), Err("ANN-101"));
    let moved = fixture
        .call(
            "annotations.update",
            json!({ "project": project, "id": id, "target": method() }),
        )
        .unwrap();
    assert_eq!(moved["target"]["name"], METHOD);
    let wrong_field = json!({ "project": project, "id": id, "title": "t" });
    assert!(fixture.call("annotations.update", wrong_field).is_ok());
}

#[test]
fn an_annotation_of_another_project_is_not_found_and_a_delete_is_final() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let created = fixture
        .call("annotations.add_note", note(project, "x"))
        .unwrap();
    let id = created["id"].clone();
    let foreign = json!({ "project": project + 1, "id": id });
    assert_eq!(fixture.call("annotations.delete", foreign), Err("ANN-201"));
    let own = json!({ "project": project, "id": id });
    assert_eq!(
        fixture.call("annotations.delete", own.clone()),
        Ok(json!({ "id": id }))
    );
    assert_eq!(fixture.call("annotations.delete", own), Err("ANN-201"));
}

fn three_notes(fixture: &Fixture) -> u64 {
    let project = fixture.scanned();
    for index in 0..3 {
        let text = format!("note {index}");
        fixture
            .call("annotations.add_note", note(project, &text))
            .unwrap();
    }
    project
}

fn listed(fixture: &Fixture, input: Value) -> Value {
    fixture.call("annotations.list", input).unwrap()
}

#[test]
fn lists_come_in_pages() {
    let fixture = Fixture::installed(None);
    let project = three_notes(&fixture);
    let first = listed(&fixture, json!({ "project": project, "limit": 2 }));
    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    assert_eq!(first["total"], 3);
    assert_eq!(first["next_offset"], 2);
    let last = listed(
        &fixture,
        json!({ "project": project, "limit": 2, "offset": 2 }),
    );
    assert_eq!(last["items"].as_array().unwrap().len(), 1);
    assert_eq!(last["next_offset"], Value::Null);
}

#[test]
fn the_page_size_is_held_to_the_configuration() {
    let fixture = Fixture::installed(None);
    let project = three_notes(&fixture);
    let huge = listed(&fixture, json!({ "project": project, "limit": 1_000_000 }));
    assert_eq!(huge["limit"], 500, "annotations.max_page_size");
    let none = listed(&fixture, json!({ "project": project, "limit": 0 }));
    assert_eq!(none["limit"], 1);
    let default = listed(&fixture, json!({ "project": project }));
    assert_eq!(default["limit"], 100, "annotations.default_page_size");
}

/// A project with the critical method changed and rescanned, so that one alert is open.
fn raised(fixture: &Fixture, targets: &[Value]) -> (u64, Receiver<Event>) {
    let project = fixture.scanned();
    let events = fixture.events.subscribe();
    for target in targets {
        fixture
            .call(
                "annotations.mark_critical",
                json!({ "project": project, "target": target, "text": "money" }),
            )
            .unwrap();
    }
    change_the_method(fixture);
    fixture
        .call("projects.rescan", json!({ "project": project }))
        .unwrap();
    (project, events)
}

fn open_alerts(fixture: &Fixture, project: u64) -> Value {
    fixture
        .call("alerts.list", json!({ "project": project }))
        .unwrap()
}

#[test]
fn a_change_of_a_critical_symbol_raises_an_alert_with_an_event() {
    let fixture = Fixture::installed(None);
    let (project, events) = raised(&fixture, &[method()]);
    let seen = alerts_seen(&events);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].project, ProjectId::new(project));
    assert_eq!(seen[0].open, 1);
    let alerts = open_alerts(&fixture, project);
    assert_eq!(alerts["open"], 1);
    let alert = &alerts["items"][0];
    assert_eq!(alert["kind"], "changed");
    assert_eq!(alert["state"], "open");
    assert_eq!(alert["changes"], json!(["content"]));
    assert_eq!(alert["target"]["name"], METHOD);
    assert_eq!(alert["reason"], "money");
}

#[test]
fn a_dismissal_takes_the_code_as_it_is_as_the_baseline() {
    let fixture = Fixture::installed(None);
    let (project, events) = raised(&fixture, &[method()]);
    alerts_seen(&events);
    let alert = open_alerts(&fixture, project)["items"][0]["id"].clone();
    let one = json!({ "project": project, "alert": alert });
    let dismissed = fixture.call("alerts.dismiss", one.clone()).unwrap();
    assert_eq!(dismissed, json!({ "dismissed": 1, "open": 0 }));
    let seen = alerts_seen(&events);
    assert_eq!((seen.len(), seen[0].open), (1, 0));
    let done = fixture
        .call(
            "alerts.list",
            json!({ "project": project, "state": "dismissed" }),
        )
        .unwrap();
    assert_eq!(
        (done["total"].clone(), done["open"].clone()),
        (json!(1), json!(0))
    );
    assert_eq!(fixture.call("alerts.dismiss", one), Err("ANN-202"));
    fixture
        .call("projects.rescan", json!({ "project": project }))
        .unwrap();
    assert_eq!(alerts_seen(&events).len(), 0, "a rescan raises nothing");
}

#[test]
fn dismissing_all_needs_the_confirmation() {
    let fixture = Fixture::installed(None);
    let (project, events) = raised(&fixture, &[class(), method()]);
    assert_eq!(open_alerts(&fixture, project)["open"], 2);
    alerts_seen(&events);
    let unconfirmed = json!({ "project": project });
    assert_eq!(
        fixture.call("alerts.dismiss_all", unconfirmed),
        Err("API-101")
    );
    let declined = json!({ "project": project, "confirm": false });
    assert_eq!(fixture.call("alerts.dismiss_all", declined), Err("API-101"));
    assert_eq!(alerts_seen(&events).len(), 0, "a refusal publishes nothing");
    assert_eq!(open_alerts(&fixture, project)["open"], 2);
}

#[test]
fn dismissing_all_takes_the_filter_and_publishes_one_event() {
    let fixture = Fixture::installed(None);
    let (project, events) = raised(&fixture, &[class(), method()]);
    alerts_seen(&events);
    let wrong_kind = json!({ "project": project, "confirm": true, "kinds": ["removed"] });
    assert_eq!(
        fixture.call("alerts.dismiss_all", wrong_kind),
        Ok(json!({ "dismissed": 0, "open": 2 }))
    );
    let all = json!({ "project": project, "confirm": true, "kinds": ["changed", "orphaned"] });
    assert_eq!(
        fixture.call("alerts.dismiss_all", all),
        Ok(json!({ "dismissed": 2, "open": 0 }))
    );
    let seen = alerts_seen(&events);
    assert_eq!(
        (seen.len(), seen[0].open),
        (1, 0),
        "one event for the filter"
    );
}

fn reads(project: u64) -> Vec<(&'static str, Value)> {
    vec![
        ("annotations.list", json!({ "project": project })),
        ("annotations.summary", json!({ "project": project })),
        ("alerts.list", json!({ "project": project })),
    ]
}

fn writes(project: u64, text: &str) -> Vec<(&'static str, Value)> {
    vec![
        (
            "annotations.create",
            json!({ "project": project, "target": class(), "kind": "note", "text": text }),
        ),
        ("annotations.add_note", note(project, text)),
        ("annotations.add_task", task(project, text)),
        (
            "annotations.mark_critical",
            json!({ "project": project, "target": method() }),
        ),
        (
            "annotations.update",
            json!({ "project": project, "id": 999, "text": text }),
        ),
        (
            "annotations.delete",
            json!({ "project": project, "id": 999 }),
        ),
        (
            "alerts.dismiss",
            json!({ "project": project, "alert": 999 }),
        ),
        (
            "alerts.dismiss_all",
            json!({ "project": project, "confirm": true }),
        ),
    ]
}

/// What `caller` gets for every command: the denial exactly when it lacks the right.
fn check_caller(
    fixture: &Fixture,
    project: u64,
    (who, caller): (&str, Principal),
    rights: (bool, bool),
) {
    let (may_read, may_write) = rights;
    for (id, input) in reads(project) {
        let answer = fixture.call_as(caller, id, input);
        assert_eq!(answer.is_ok(), may_read, "{who} calls {id}: {answer:?}");
        assert!(may_read || answer == Err("AUTH-401"), "{who} calls {id}");
    }
    for (id, input) in writes(project, who) {
        let answer = fixture.call_as(caller, id, input);
        // Allowed, it runs and ends in its own answer; never in the denial.
        assert_eq!(answer == Err("AUTH-401"), !may_write, "{who} calls {id}");
    }
}

#[test]
fn every_command_runs_for_exactly_the_callers_that_hold_its_permission() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    // Who holds what: the owner both, a reader only `read`, an annotator only `annotate`, the
    // clients of MCP and of `serve` both, nobody nothing.
    let callers = [
        ("owner", principal::resolve(), (true, true)),
        ("mcp client", principal::remote_client(), (true, true)),
        ("browser client", principal::browser_client(), (true, true)),
        ("reader", principal::reader(), (true, false)),
        ("annotator", annotate_only(), (false, true)),
        ("nobody", principal::nobody(), (false, false)),
    ];
    for (who, caller, rights) in callers {
        check_caller(&fixture, project, (who, caller), rights);
    }
}

fn both_lists(fixture: &Fixture, project: u64) -> (Value, Value) {
    (
        listed(fixture, json!({ "project": project })),
        open_alerts(fixture, project),
    )
}

#[test]
fn a_denied_call_changes_nothing_and_publishes_no_event() {
    let fixture = Fixture::installed(None);
    let (project, _) = raised(&fixture, &[method()]);
    let before = both_lists(&fixture, project);
    let events = fixture.events.subscribe();
    let id = before.1["items"][0]["id"].clone();
    let note_id = before.0["items"][0]["id"].clone();
    for caller in [principal::reader(), principal::nobody()] {
        let attempts = [
            ("annotations.add_note", note(project, "sneaky")),
            (
                "annotations.update",
                json!({ "project": project, "id": note_id, "text": "x" }),
            ),
            (
                "annotations.delete",
                json!({ "project": project, "id": note_id }),
            ),
            ("alerts.dismiss", json!({ "project": project, "alert": id })),
            (
                "alerts.dismiss_all",
                json!({ "project": project, "confirm": true }),
            ),
        ];
        for (command, input) in attempts {
            let answer = fixture.call_as(caller, command, input);
            assert_eq!(answer, Err("AUTH-401"), "{command}");
        }
    }
    assert_eq!(both_lists(&fixture, project), before);
    assert_eq!(alerts_seen(&events).len(), 0);
}
