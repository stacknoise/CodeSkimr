//! Tests of the check for error codes that nothing uses.

use codeskimr_codegen::registry::Registry;
use codeskimr_codegen::usage::{Source, unused_codes};

const REGISTRY: &str = r#"
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

[[codes]]
id = "SRC-202"
name = "source_path_not_found_twice"
severity = "error"
retryable = false
description = "Missing folder, again."
since = "0.1.0"
"#;

/// The test registry; an unparsable one is empty, which makes every test below fail.
fn registry() -> Registry {
    Registry::parse(REGISTRY).unwrap_or(Registry {
        groups: std::collections::BTreeMap::default(),
        codes: Vec::new(),
    })
}

fn source(text: &str) -> Vec<Source> {
    vec![("crates/x/src/lib.rs".to_string(), text.to_string())]
}

#[test]
fn a_code_named_in_rust_is_used() {
    let found = unused_codes(&registry(), &source("ErrorCode::SourcePathNotFoundTwice"));
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("SRC-201"));
}

#[test]
fn a_variant_that_only_starts_like_another_does_not_count() {
    let found = unused_codes(&registry(), &source("ErrorCode::SourcePathNotFoundTwice"));
    assert!(found.iter().all(|p| !p.contains("SRC-202")));
    assert!(found.iter().any(|p| p.contains("SRC-201")));
}

#[test]
fn a_quoted_id_in_typescript_is_used() {
    let found = unused_codes(&registry(), &source("const a = \"SRC-201\"; \"SRC-202\""));
    assert_eq!(found.len(), 0);
}

#[test]
fn nothing_used_reports_every_code() {
    assert_eq!(unused_codes(&registry(), &[]).len(), 2);
}

#[test]
fn a_retired_code_is_not_reported() {
    let text = REGISTRY.replace("since = \"0.1.0\"", "since = \"0.1.0\"\ndeprecated = true");
    let registry = Registry::parse(&text).expect("parses");
    assert_eq!(unused_codes(&registry, &[]).len(), 0);
}

#[test]
fn a_mention_in_an_inline_test_module_does_not_count() {
    let text = "fn f() {}\n#[cfg(test)]\nmod tests { fn t() { ErrorCode::SourcePathNotFound; } }";
    assert_eq!(unused_codes(&registry(), &source(text)).len(), 2);
}

#[test]
fn a_longer_type_name_in_front_does_not_count() {
    let found = unused_codes(&registry(), &source("FooErrorCode::SourcePathNotFound"));
    assert!(found.iter().any(|p| p.contains("SRC-201")));
}

#[test]
fn a_full_path_counts() {
    let found = unused_codes(
        &registry(),
        &source("codeskimr_codes::ErrorCode::SourcePathNotFound"),
    );
    assert!(found.iter().all(|p| !p.contains("SRC-201")));
}

#[test]
fn a_test_attribute_on_an_import_does_not_hide_the_code_below() {
    let text = "#[cfg(test)]\nuse x::y;\nfn f() { ErrorCode::SourcePathNotFound; }";
    let found = unused_codes(&registry(), &source(text));
    assert!(found.iter().all(|p| !p.contains("SRC-201")));
}
