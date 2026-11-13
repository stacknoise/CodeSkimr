//! Tests of registry validation and rendering with small inline registries.

use codeskimr_codegen::registry::Registry;
use codeskimr_codegen::validate::{check_messages, validate_registry};
use codeskimr_codegen::{render_md, render_rust, render_ts};

const GOOD: &str = r#"
[groups.SRC]
title = "Sources"
crate = "codeskimr_source"

[[codes]]
id = "SRC-201"
name = "source_path_not_found"
severity = "error"
retryable = false
description = "Missing folder."
since = "0.1.0"
"#;

fn problems(text: &str) -> Vec<String> {
    Registry::parse(text)
        .map(|registry| validate_registry(&registry))
        .ok()
        .and_then(Result::err)
        .unwrap_or_default()
}

fn with_code(extra: &str) -> String {
    format!("{GOOD}\n[[codes]]\n{extra}")
}

#[test]
fn good_registry_is_valid() {
    let registry = Registry::parse(GOOD).expect("parses");
    assert!(validate_registry(&registry).is_ok());
}

#[test]
fn duplicate_id_is_rejected() {
    let text = with_code(
        "id = \"SRC-201\"\nname = \"other\"\nseverity = \"error\"\nretryable = false\ndescription = \"x\"\nsince = \"0.1.0\"\n",
    );
    assert!(problems(&text).iter().any(|p| p.contains("duplicate id")));
}

#[test]
fn malformed_ids_are_rejected() {
    for bad in [
        "SRC-701", "SRC-200", "SRC-20", "src-201", "SRC201", "S-201", "SRC-2011",
    ] {
        let text = GOOD.replace("SRC-201", bad);
        assert!(!problems(&text).is_empty(), "{bad} should be rejected");
    }
}

#[test]
fn undeclared_group_and_bad_name_are_rejected() {
    let text = with_code(
        "id = \"NET-501\"\nname = \"Bad Name\"\nseverity = \"error\"\nretryable = true\ndescription = \"x\"\nsince = \"0.1.0\"\n",
    );
    let found = problems(&text);
    assert!(found.iter().any(|p| p.contains("not declared")));
    assert!(found.iter().any(|p| p.contains("snake_case")));
}

#[test]
fn missing_message_and_orphan_are_reported() {
    let registry = Registry::parse(GOOD).expect("parses");
    let missing = check_messages(&registry, "de", "");
    assert!(missing.iter().any(|p| p.contains("SRC-201")));
    let orphan = check_messages(&registry, "en", "error-SRC-201 = a\nerror-SRC-999 = b\n");
    assert_eq!(orphan.len(), 1);
    assert!(orphan[0].contains("SRC-999"));
}

#[test]
fn deprecated_code_needs_no_message() {
    let text = GOOD.replace("since = \"0.1.0\"", "since = \"0.1.0\"\ndeprecated = true");
    let registry = Registry::parse(&text).expect("parses");
    let missing = check_messages(&registry, "de", "");
    assert!(missing.is_empty(), "unexpected findings: {missing:?}");
}

#[test]
fn renderers_contain_the_code() {
    let registry = Registry::parse(GOOD).expect("parses");
    assert!(render_rust::render(&registry).contains("SourcePathNotFound"));
    assert!(render_ts::render(&registry).contains("\"SRC-201\""));
    assert!(render_md::render(&registry).contains("| SRC-201 | `source_path_not_found`"));
}
