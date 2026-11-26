//! `get_context` against fake ports: one answer with the symbol, its callers, dependencies,
//! affected files and annotations, cut by the budget.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    Annotation, AnnotationContent, AnnotationId, AnnotationKind, AnnotationLimits,
    AnnotationTarget, Author, Permission, PermissionSet, Principal, QualifiedName, SymbolKind,
    Timestamp,
};
use codeskimr_mcp::ContextArgs;
use common::{FakeAnnotations, FakeIndex, FakeUsages, PROJECT, SCANNED_AT, tools, tools_annotated};
use std::sync::Arc;

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn context(symbol: &str, budget: Option<&str>) -> ContextArgs {
    ContextArgs {
        symbol: symbol.to_owned(),
        budget: budget.map(str::to_owned),
        project: None,
    }
}

#[cfg(test)]
fn note(id: u64, name: &str, text: &str) -> Annotation {
    let limits = AnnotationLimits {
        max_text_chars: 100,
        max_title_chars: 20,
    };
    Annotation {
        id: AnnotationId::new(id),
        project: PROJECT,
        target: AnnotationTarget {
            kind: SymbolKind::Class,
            name: QualifiedName::new(name).unwrap(),
            parameters: Vec::new(),
        },
        author: Author::new("local").unwrap(),
        content: AnnotationContent {
            kind: AnnotationKind::Note,
            title: None,
            text: Some(limits.text(text).unwrap()),
            status: None,
            due: None,
            assignee: None,
        },
        created: Timestamp(SCANNED_AT),
        updated: Timestamp(SCANNED_AT),
        without_target: false,
    }
}

#[test]
fn one_answer_holds_the_symbol_its_callers_dependencies_files_and_notes() {
    let notes = FakeAnnotations {
        annotations: vec![note(1, "App.Domain.Order", "Keep the rounding")],
        alerts: Vec::new(),
    };
    let answer = tools_annotated(notes)
        .get_context(&owner(), &context("App.Domain.Order", None))
        .unwrap();
    assert_eq!(answer["snapshot"]["id"], 10);
    assert_eq!(answer["budget"], "medium");
    assert_eq!(answer["symbol"]["qualified_name"], "App.Domain.Order");
    assert_eq!(answer["symbol"]["signature"], "Order()");
    assert_eq!(answer["symbol"]["doc"], "Does the work.");
    assert_eq!(answer["symbol"]["layer"], "Domain");
    assert_eq!(answer["symbol"]["first_line"], 5);
    assert_callers_and_notes(&answer);
}

#[cfg(test)]
fn assert_callers_and_notes(answer: &serde_json::Value) {
    let callers = &answer["callers"];
    assert_eq!(callers["total"], 3);
    assert_eq!(callers["items"][0]["verified"], true);
    assert_eq!(
        callers["items"][0]["from"]["qualified_name"],
        "App.Application.Service.Run"
    );
    assert_eq!(callers["items"][0]["line"], 12);
    assert_eq!(answer["dependencies"]["items"][0]["path"], "src/far.cs");
    assert_eq!(answer["affected_files"]["items"][0], "src/app/service.cs");
    assert_eq!(answer["annotations"]["total"], 1);
    assert_eq!(
        answer["annotations"]["items"][0]["text"],
        "Keep the rounding"
    );
    assert_eq!(answer["preliminary"], true);
}

#[test]
fn the_budget_cuts_every_list_and_a_notice_says_so() {
    let short = tools()
        .get_context(&owner(), &context("1", Some("short")))
        .unwrap();
    assert_eq!(short["budget"], "short");
    assert_eq!(short["callers"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(short["callers"]["total"], 3);
    assert_eq!(
        short["notice"]["code"],
        ErrorCode::ResponseLimitExceeded.id()
    );
    assert!(short["symbol"].get("doc").is_none());
    let long = tools()
        .get_context(&owner(), &context("1", Some("long")))
        .unwrap();
    assert_eq!(long["callers"]["items"].as_array().unwrap().len(), 3);
    assert_eq!(long["notice"], serde_json::Value::Null);
}

#[test]
fn only_direct_callers_without_tests_are_asked_for() {
    let usages = Arc::new(FakeUsages::default());
    let index = FakeIndex {
        scanned: true,
        fail: false,
    };
    common::tools_using(index, Arc::clone(&usages))
        .get_context(&owner(), &context("1", None))
        .unwrap();
    let asked = usages.asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0.depth, 1);
    assert!(!asked[0].0.include_tests);
}

#[test]
fn a_wrong_budget_a_missing_symbol_and_a_missing_right_fail_with_their_codes() {
    let bad = tools()
        .get_context(&owner(), &context("1", Some("huge")))
        .unwrap_err();
    assert_eq!(bad.code(), ErrorCode::ToolArgumentsInvalid);
    let missing = tools()
        .get_context(&owner(), &context("999", None))
        .unwrap_err();
    assert_eq!(missing.code(), ErrorCode::SymbolNotFound);
    let nobody = Principal::limited(PermissionSet::none());
    let denied = tools()
        .get_context(&nobody, &context("1", None))
        .unwrap_err();
    assert_eq!(denied.code(), ErrorCode::PermissionDenied);
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    assert!(tools().get_context(&reader, &context("1", None)).is_ok());
}
