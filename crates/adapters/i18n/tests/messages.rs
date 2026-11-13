//! Tests of the embedded Fluent bundles against the real `locales/` folder.

use std::collections::BTreeSet;

use codeskimr_codes::{Coded, ErrorCode, i18n};
use codeskimr_i18n::{FluentArgs, I18nError, Messages};
use fluent_syntax::ast::{
    CallArguments, Entry, Expression, InlineExpression, Pattern, PatternElement,
};
use fluent_syntax::parser::parse;

const GERMAN: &str = "de";
const ENGLISH: &str = "en";

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn messages() -> Result<Messages, I18nError> {
    Messages::load()
}

#[test]
fn german_and_english_are_available() -> TestResult {
    let messages = messages()?;
    let langs: Vec<_> = messages.languages().collect();
    assert_eq!(langs, [GERMAN, ENGLISH]);
    Ok(())
}

#[test]
fn every_generated_key_resolves_in_every_language() -> TestResult {
    let messages = messages()?;
    let mut args = FluentArgs::new();
    for name in ["count", "ms", "mb", "projects", "types"] {
        args.set(name, 3);
    }
    for lang in messages.languages() {
        for key in i18n::ALL {
            let text = messages.format(lang, key, Some(&args));
            assert!(text.is_ok_and(|t| !t.is_empty()), "{lang}: {key}");
        }
    }
    Ok(())
}

#[test]
fn every_active_error_code_has_a_message_in_every_language() -> TestResult {
    let messages = messages()?;
    for code in ErrorCode::ALL.into_iter().filter(|c| !c.entry().deprecated) {
        for lang in messages.languages() {
            let text = messages.format(lang, code.message_key(), None);
            assert!(text.is_ok(), "{lang}: {}", code.id());
        }
    }
    Ok(())
}

#[test]
fn german_plural_and_arguments_are_rendered() -> TestResult {
    let messages = messages()?;
    let mut args = FluentArgs::new();
    args.set("projects", 1);
    args.set("types", 12);
    let one = messages.format(GERMAN, i18n::PROJECTS_SUMMARY, Some(&args))?;
    assert_eq!(one, "1 Projekt · 12 Typen");
    args.set("projects", 4);
    let many = messages.format(ENGLISH, i18n::PROJECTS_SUMMARY, Some(&args))?;
    assert_eq!(many, "4 projects · 12 types");
    Ok(())
}

#[test]
fn unknown_language_falls_back_to_english() -> TestResult {
    let text = messages()?.format("xx", i18n::APP_WORKSPACE, None)?;
    assert_eq!(text, "Workspace");
    Ok(())
}

#[test]
fn missing_key_reports_the_registered_code() -> TestResult {
    let result = messages()?.format(GERMAN, "no-such-key", None);
    assert!(matches!(result, Err(I18nError::MessageKeyMissing { .. })));
    assert_eq!(
        result.err().map(|e| e.code()),
        Some(ErrorCode::MessageKeyMissing)
    );
    Ok(())
}

/// Collects the names of all `$variables` used in a pattern.
fn pattern_variables(pattern: &Pattern<&str>, out: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_variables(expression, out);
        }
    }
}

fn expression_variables(expression: &Expression<&str>, out: &mut BTreeSet<String>) {
    match expression {
        Expression::Inline(inline) => inline_variables(inline, out),
        Expression::Select { selector, variants } => {
            inline_variables(selector, out);
            for variant in variants {
                pattern_variables(&variant.value, out);
            }
        }
    }
}

fn inline_variables(inline: &InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match inline {
        InlineExpression::VariableReference { id } => {
            out.insert(id.name.to_string());
        }
        InlineExpression::Placeable { expression } => expression_variables(expression, out),
        InlineExpression::FunctionReference { arguments, .. } => call_variables(arguments, out),
        _ => {}
    }
}

fn call_variables(arguments: &CallArguments<&str>, out: &mut BTreeSet<String>) {
    for positional in &arguments.positional {
        inline_variables(positional, out);
    }
}

/// Variables per message id for one embedded file text.
fn variables_by_message(text: &str) -> Result<Vec<(String, BTreeSet<String>)>, String> {
    let resource = parse(text).map_err(|(_, errors)| format!("{errors:?}"))?;
    Ok(resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(message) => Some(message),
            _ => None,
        })
        .map(|message| {
            let mut vars = BTreeSet::new();
            if let Some(pattern) = &message.value {
                pattern_variables(pattern, &mut vars);
            }
            (message.id.name.to_string(), vars)
        })
        .collect())
}

fn locale_file(lang: &str, file: &str) -> std::io::Result<String> {
    let path = format!(
        "{}/../../../locales/{lang}/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path)
}

#[test]
fn translations_use_the_same_variables_as_english() -> TestResult {
    let dir = format!("{}/../../../locales/{ENGLISH}", env!("CARGO_MANIFEST_DIR"));
    for entry in std::fs::read_dir(dir)?.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        let english = variables_by_message(&locale_file(ENGLISH, &file)?)?;
        let german = variables_by_message(&locale_file(GERMAN, &file)?)?;
        assert_eq!(english, german, "variables differ in {file}");
    }
    Ok(())
}
