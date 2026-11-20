//! The commands of the level 2 switch through the same path the server takes (ADR 0071).

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_cli::{GlobalOptions, Level2Action};
use codeskimr_codes::ErrorCode;
use serde_json::{Value, json};

use crate::level2_consent;
use crate::principal;
use crate::ui_commands::tests::{Fixture, USER_CONFIG_FILE};

const SWITCH_ON: &str = "[level2]\nenabled = true\n";
const NAME: &str = "app";

fn status(fixture: &Fixture, project: u64) -> Value {
    fixture
        .call_as(
            principal::browser_client(),
            "level2.status",
            json!({ "project": project }),
        )
        .unwrap()
}

/// The owner confirms in the terminal: the name is typed, as the command line asks it.
fn confirm_at_the_terminal(fixture: &Fixture, project: u64) {
    let action = Level2Action::Confirm {
        project: project.to_string(),
        from_ui: false,
    };
    let typed = |_: &level2_consent::Target| -> Result<String, CodedError> { Ok(NAME.to_owned()) };
    let mut global = GlobalOptions::default();
    global.overrides.clear();
    level2_consent::run(&action, &global, &fixture.locations, &typed).unwrap();
}

#[test]
fn the_answer_holds_the_project_the_state_the_confirmation_and_the_way_to_it_and_nothing_else() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let answer = status(&fixture, project);
    assert_eq!(
        answer,
        json!({ "project": project, "state": "not_enabled", "confirmed": false, "launch": "copy" })
    );
    // No path and no text of the project.
    let text = answer.to_string();
    assert!(
        !text.contains(&fixture.folder()) && !text.contains(NAME),
        "{text}"
    );
}

#[test]
fn the_switch_in_the_user_configuration_decides_between_not_enabled_and_off() {
    let fixture = Fixture::installed_with(None, Some(SWITCH_ON));
    let project = fixture.scanned();
    assert_eq!(status(&fixture, project)["state"], "off");
    // The file is read again with every call, so an edit counts at once.
    let file = fixture.root.path().join(USER_CONFIG_FILE);
    std::fs::write(&file, "[level2]\nenabled = false\n").unwrap();
    assert_eq!(status(&fixture, project)["state"], "not_enabled");
    std::fs::write(&file, SWITCH_ON).unwrap();
    assert_eq!(status(&fixture, project)["state"], "off");
}

#[test]
fn a_confirmation_at_the_terminal_makes_it_on_and_one_click_switches_it_off() {
    let fixture = Fixture::installed_with(None, Some(SWITCH_ON));
    let project = fixture.scanned();
    confirm_at_the_terminal(&fixture, project);
    assert_eq!(status(&fixture, project)["state"], "on");
    assert_eq!(status(&fixture, project)["confirmed"], true);
    let revoked = fixture
        .call_as(
            principal::browser_client(),
            "level2.revoke",
            json!({ "project": project }),
        )
        .unwrap();
    assert_eq!(revoked["state"], "off");
    assert_eq!(revoked["confirmed"], false);
    assert_eq!(status(&fixture, project), revoked);
}

#[test]
fn a_confirmation_that_exists_is_reported_although_the_switch_is_off() {
    let fixture = Fixture::installed_with(None, Some(SWITCH_ON));
    let project = fixture.scanned();
    confirm_at_the_terminal(&fixture, project);
    std::fs::write(
        fixture.root.path().join(USER_CONFIG_FILE),
        "[level2]\nenabled = false\n",
    )
    .unwrap();
    let answer = status(&fixture, project);
    assert_eq!(answer["state"], "not_enabled");
    assert_eq!(answer["confirmed"], true);
}

#[test]
fn revoking_needs_the_right_and_an_unknown_project_is_prj_201() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    for caller in [
        principal::reader(),
        principal::nobody(),
        principal::annotator(),
    ] {
        assert_eq!(
            fixture.call_as(caller, "level2.revoke", json!({ "project": project })),
            Err("AUTH-401")
        );
    }
    for id in ["level2.revoke", "level2.status"] {
        assert_eq!(
            fixture.call_as(principal::browser_client(), id, json!({ "project": 99 })),
            Err("PRJ-201"),
            "{id}"
        );
    }
}

#[test]
fn the_input_takes_only_the_project_and_the_status_needs_the_read_right_only() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    for input in [
        json!({}),
        json!({ "project": "1" }),
        json!({ "project": project, "confirm": true }),
        Value::Null,
    ] {
        assert_eq!(
            fixture.call_as(principal::browser_client(), "level2.status", input),
            Err("API-101")
        );
    }
    assert_eq!(
        fixture.call_as(
            principal::nobody(),
            "level2.status",
            json!({ "project": project })
        ),
        Err("AUTH-401")
    );
}

#[test]
fn a_browser_client_sees_both_commands_and_none_of_them_confirms() {
    let fixture = Fixture::installed(None);
    let ids: Vec<_> = fixture
        .api
        .list(&principal::browser_client())
        .into_iter()
        .map(|info| info.id)
        .filter(|id| id.starts_with("level2."))
        .collect();
    assert_eq!(ids, ["level2.revoke", "level2.status"]);
    let project = fixture.scanned();
    // Whatever a client sends, revoking never confirms.
    let answer = fixture
        .call_as(
            principal::browser_client(),
            "level2.revoke",
            json!({ "project": project }),
        )
        .unwrap();
    assert_eq!(answer["confirmed"], false);
    assert_eq!(status(&fixture, project)["confirmed"], false);
    let sees_status_only = fixture.api.list(&principal::reader());
    assert!(
        sees_status_only
            .iter()
            .any(|info| info.id == "level2.status")
    );
    assert!(
        sees_status_only
            .iter()
            .all(|info| info.id != "level2.revoke")
    );
}

#[test]
fn the_registry_flags_the_two_commands_as_the_token_rule_needs_them() {
    let fixture = Fixture::installed(None);
    let flags: Vec<_> = fixture
        .api
        .list(&principal::browser_client())
        .into_iter()
        .filter(|info| info.id.starts_with("level2."))
        .map(|info| (info.id, info.read_only))
        .collect();
    // Revoking changes something, so the route asks for the installation token; reading does not.
    assert_eq!(flags, [("level2.revoke", false), ("level2.status", true)]);
    let _ = (Arc::clone(&fixture.api), ErrorCode::PermissionDenied);
}

#[test]
fn an_mcp_client_may_read_the_state_but_not_revoke() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let ids: Vec<_> = fixture
        .api
        .list(&principal::remote_client())
        .into_iter()
        .map(|info| info.id)
        .filter(|id| id.starts_with("level2."))
        .collect();
    assert_eq!(ids, ["level2.status"]);
    assert_eq!(
        fixture.call_as(
            principal::remote_client(),
            "level2.revoke",
            json!({ "project": project })
        ),
        Err("AUTH-401")
    );
}
