//! Tests of the tool report.
#![cfg(test)]

use codeskimr_domain::{LanguageId, Timestamp, ToolFinding, ToolReport, ToolRole, ToolState};

fn finding(language: &str, tool: &str, state: ToolState) -> ToolFinding {
    ToolFinding {
        language: LanguageId::new(language).unwrap(),
        tool: tool.to_owned(),
        role: ToolRole::Indexer,
        state,
        found: None,
        required: None,
        purpose_key: format!("tool-{tool}-purpose"),
        install_key: format!("tool-{tool}-install"),
    }
}

fn report(findings: Vec<ToolFinding>) -> ToolReport {
    ToolReport {
        checked_at: Timestamp(1),
        findings,
    }
}

#[test]
fn a_language_is_ready_only_when_all_its_tools_are() {
    let report = report(vec![
        finding("csharp", "scip-dotnet", ToolState::Ready),
        finding("csharp", "dotnet", ToolState::TooOld),
        finding("rust", "cargo", ToolState::Ready),
    ]);
    let ready = |name: &str| report.level2_ready(&LanguageId::new(name).unwrap());
    assert!(!ready("csharp"));
    assert!(ready("rust"));
    assert!(!ready("go"), "a language without tools has no level 2");
}

#[test]
fn tools_shared_by_languages_are_listed_once_with_all_their_languages() {
    let report = report(vec![
        finding("typescript", "node", ToolState::Missing),
        finding("tsx", "node", ToolState::Missing),
        finding("typescript", "scip-typescript", ToolState::Ready),
    ]);
    let problems = report.problems();
    assert_eq!(problems.len(), 1);
    let languages: Vec<_> = problems[0]
        .languages
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(languages, ["tsx", "typescript"]);
    assert_eq!(report.tools().len(), 2);
    assert_eq!(report.languages().len(), 2);
}
