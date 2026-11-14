//! The stored result of the tool check, and the languages of one project.
#![cfg(test)]

#[cfg(test)]
mod common;

use codeskimr_application::ports::{ProjectCatalog, ToolReportStore};
use codeskimr_domain::{
    LanguageId, ProjectId, Timestamp, ToolFinding, ToolReport, ToolRole, ToolState,
};
use common::{PROJECT, code_of, project, scanned, store};

fn finding(tool: &str, state: ToolState, found: Option<&str>) -> ToolFinding {
    ToolFinding {
        language: LanguageId::new("csharp").unwrap(),
        tool: tool.to_owned(),
        role: ToolRole::Toolchain,
        state,
        found: found.map(str::to_owned),
        required: Some("9.0.100".to_owned()),
        purpose_key: format!("tool-{tool}-purpose"),
        install_key: format!("tool-{tool}-install"),
    }
}

fn report(at: i64, findings: Vec<ToolFinding>) -> ToolReport {
    ToolReport {
        checked_at: Timestamp(at),
        findings,
    }
}

#[test]
fn a_project_that_was_never_checked_has_no_report() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.load(PROJECT).unwrap(), None);
}

#[test]
fn a_report_is_stored_with_its_findings_in_order() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let stored = report(
        50,
        vec![
            finding("scip-dotnet", ToolState::Missing, None),
            finding("dotnet", ToolState::TooOld, Some("8.0.404")),
        ],
    );
    store.replace(PROJECT, &stored).unwrap();
    assert_eq!(store.load(PROJECT).unwrap(), Some(stored));
}

#[test]
fn a_new_check_replaces_the_old_report_and_an_empty_one_is_still_a_report() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store
        .replace(
            PROJECT,
            &report(50, vec![finding("dotnet", ToolState::Missing, None)]),
        )
        .unwrap();
    let newer = report(
        60,
        vec![finding("dotnet", ToolState::Ready, Some("9.0.100"))],
    );
    store.replace(PROJECT, &newer).unwrap();
    assert_eq!(store.load(PROJECT).unwrap(), Some(newer));
    let empty = report(70, Vec::new());
    store.replace(PROJECT, &empty).unwrap();
    assert_eq!(store.load(PROJECT).unwrap(), Some(empty));
}

#[test]
fn reports_of_different_projects_do_not_mix() {
    let store = store(3);
    store.save_project(&project(1, "/work/a", &[])).unwrap();
    store.save_project(&project(2, "/work/b", &[])).unwrap();
    let first = report(50, vec![finding("dotnet", ToolState::Missing, None)]);
    store.replace(PROJECT, &first).unwrap();
    assert_eq!(store.load(ProjectId::new(2)).unwrap(), None);
    assert_eq!(store.load(PROJECT).unwrap(), Some(first));
}

#[test]
fn a_report_for_an_unknown_project_is_refused() {
    let store = store(3);
    let error = store.replace(PROJECT, &report(1, Vec::new())).unwrap_err();
    assert!(code_of(&error).starts_with("STO-"), "{}", code_of(&error));
}

#[test]
fn the_languages_of_a_project_come_from_its_newest_snapshot() {
    let store = scanned(&[], Vec::new(), Vec::new(), Vec::new());
    let languages = store.languages(PROJECT).unwrap();
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].language.as_str(), "csharp");
}

#[test]
fn a_project_without_a_scan_has_no_languages_and_an_unknown_one_is_refused() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.languages(PROJECT).unwrap(), Vec::new());
    let error = store.languages(ProjectId::new(9)).unwrap_err();
    assert_eq!(code_of(&error), "PRJ-201");
}
