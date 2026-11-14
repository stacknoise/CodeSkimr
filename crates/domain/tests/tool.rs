//! Tests of versions and of the tool report.
#![cfg(test)]

use codeskimr_domain::{
    LanguageId, Timestamp, ToolFinding, ToolReport, ToolRole, ToolState, ToolVersion,
};

fn version(text: &str) -> ToolVersion {
    ToolVersion::find_in(text).unwrap()
}

#[test]
fn it_finds_the_version_in_the_output_of_common_tools() {
    for (output, expected) in [
        ("8.0.404\n", "8.0.404"),
        ("v20.11.1", "20.11.1"),
        ("Python 3.12.1", "3.12.1"),
        ("cargo 1.82.0 (8f40fc59f 2024-08-21)", "1.82.0"),
        ("openjdk version \"17.0.9\" 2023-10-17", "17.0.9"),
        (
            "rust-analyzer 0.3.2135-standalone (a5b2 2024-08-12)",
            "0.3.2135",
        ),
    ] {
        assert_eq!(version(output).to_string(), expected, "{output}");
    }
}

#[test]
fn it_finds_no_version_without_a_dotted_number() {
    for output in ["", "no version here", "build 42", "...", "1..2"] {
        assert!(ToolVersion::find_in(output).is_none(), "{output}");
    }
}

#[test]
fn versions_compare_number_by_number_and_a_missing_number_is_zero() {
    assert!(version("8.0.404") < version("9.0.100"));
    assert!(version("8.0.404") < version("8.0.1000"));
    assert!(version("3.9.18") < version("3.12.0"));
    assert_eq!(
        version("20.0"),
        ToolVersion::parse_requirement("20").unwrap()
    );
    assert!(ToolVersion::parse_requirement("20").unwrap() <= version("20.11.1"));
}

#[test]
fn a_requirement_that_is_not_a_number_asks_for_nothing() {
    for text in [
        "stable",
        "nightly-2024-01-01",
        "lts/*",
        "pypy3.10",
        "",
        ">=8",
    ] {
        assert!(ToolVersion::parse_requirement(text).is_none(), "{text}");
    }
    assert_eq!(
        ToolVersion::parse_requirement(" v18.19.0\n")
            .unwrap()
            .to_string(),
        "18.19.0"
    );
    assert_eq!(
        ToolVersion::parse_requirement("3.12.1-rc1")
            .unwrap()
            .to_string(),
        "3.12.1"
    );
}

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
