//! `get_project`, and the tools `rescan` reports.
#![cfg(test)]

mod common;

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, PermissionSet, Principal, ToolState};
use codeskimr_mcp::{McpError, ProjectArgs, RescanArgs};
use common::{FakeToolCheck, tools, tools_checking};

fn owner() -> Principal {
    Principal::local_owner()
}

fn tool_named<'a>(answer: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    answer["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["tool"] == name)
        .unwrap()
}

fn args(project: Option<u64>) -> ProjectArgs {
    ProjectArgs { project }
}

#[test]
fn a_missing_indexer_is_an_answer_with_name_purpose_and_install_hint() {
    let answer = tools().get_project(&owner(), &args(None)).unwrap();
    assert_eq!(answer["name"], "Shop");
    let missing = tool_named(&answer, "scip-dotnet");
    assert_eq!(missing["tool"], "scip-dotnet");
    assert_eq!(missing["state"], "missing");
    assert_eq!(missing["role"], "indexer");
    assert!(
        missing["purpose"]
            .as_str()
            .unwrap()
            .contains("exact usages")
    );
    assert!(
        missing["install"]
            .as_str()
            .unwrap()
            .contains("dotnet tool install --global scip-dotnet")
    );
    assert_eq!(missing["required"], "9.0.100");
}

#[test]
fn every_language_says_whether_level_2_can_run() {
    let answer = tools().get_project(&owner(), &args(None)).unwrap();
    let languages = answer["languages"].as_array().unwrap();
    assert_eq!(languages[0]["language"], "csharp");
    assert_eq!(languages[0]["level2_ready"], false);
    assert_eq!(languages[1]["language"], "go");
    assert_eq!(languages[1]["level2_ready"], false, "Go has no level 2");

    let ready = Arc::new(FakeToolCheck::with(&[
        ("scip-dotnet", ToolState::Ready),
        ("dotnet", ToolState::Ready),
    ]));
    let answer = tools_checking(ready)
        .get_project(&owner(), &args(Some(1)))
        .unwrap();
    assert_eq!(answer["languages"][0]["level2_ready"], true);
    assert_eq!(answer["tools_checked_at"], 1_090);
}

#[test]
fn the_tools_are_checked_on_every_call() {
    let check = Arc::new(FakeToolCheck::default());
    let tools = tools_checking(Arc::clone(&check));
    tools.get_project(&owner(), &args(None)).unwrap();
    tools.get_project(&owner(), &args(None)).unwrap();
    assert_eq!(*check.runs.lock().unwrap(), 2);
}

#[test]
fn a_caller_who_may_not_read_gets_nothing_and_nothing_is_checked() {
    let check = Arc::new(FakeToolCheck::default());
    let nobody = Principal::limited(PermissionSet::none().with(Permission::ReadSource));
    let error = tools_checking(Arc::clone(&check))
        .get_project(&nobody, &args(None))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
    assert_eq!(*check.runs.lock().unwrap(), 0);
}

#[test]
fn an_unknown_project_is_refused() {
    let error = tools().get_project(&owner(), &args(Some(99))).unwrap_err();
    assert!(matches!(error, McpError::ProjectNotRegistered(99)));
}

#[test]
fn a_failing_check_is_reported_with_its_code() {
    let check = FakeToolCheck {
        fail: true,
        ..FakeToolCheck::default()
    };
    let error = tools_checking(Arc::new(check))
        .get_project(&owner(), &args(None))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnexpectedInternalError);
}

#[test]
fn a_rescan_reports_the_tools_it_found_missing() {
    let answer = tools()
        .rescan(&owner(), &RescanArgs { project: None })
        .unwrap();
    assert_eq!(tool_named(&answer, "scip-dotnet")["state"], "missing");
}

#[test]
fn a_rescan_does_not_fail_when_the_check_does() {
    let check = FakeToolCheck {
        fail: true,
        ..FakeToolCheck::default()
    };
    let answer = tools_checking(Arc::new(check))
        .rescan(&owner(), &RescanArgs { project: None })
        .unwrap();
    assert_eq!(answer["tools"].as_array().unwrap().len(), 0);
}
