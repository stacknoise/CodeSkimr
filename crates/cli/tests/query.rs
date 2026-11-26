//! The query commands: their arguments, and the JSON and text they print.

use codeskimr_cli::{
    Action, ContextQuery, ImpactQuery, Parsed, QueryAction, QueryKind, QueryTarget, SymbolsQuery,
    UsagesQuery, parse, query_error_json, query_json, query_text,
};
use codeskimr_i18n::Messages;
use serde_json::{Value, json};

#[cfg(test)]
fn action(arguments: &[&str]) -> QueryAction {
    let messages = Messages::load().unwrap();
    match parse(arguments.iter().copied(), &messages, "en").unwrap() {
        Parsed::Run(invocation) => match invocation.action {
            Action::Query(query) => *query,
            other => panic!("not a query: {other:?}"),
        },
        Parsed::Display(text) => panic!("unexpected output: {text}"),
    }
}

#[test]
fn symbols_reads_its_options() {
    assert_eq!(
        action(&[
            "codeskimr",
            "symbols",
            "ord",
            "--kind",
            "class",
            "--limit",
            "5",
            "--json"
        ]),
        QueryAction::Symbols(SymbolsQuery {
            query: "ord".to_owned(),
            kind: Some("class".to_owned()),
            limit: Some(5),
            target: QueryTarget {
                project: None,
                json: true
            },
        })
    );
}

#[test]
fn usages_reads_its_options() {
    assert_eq!(
        action(&[
            "codeskimr",
            "usages",
            "A.B",
            "--direction",
            "outgoing",
            "--depth",
            "2",
            "--include-tests",
            "--project",
            "7",
        ]),
        QueryAction::Usages(UsagesQuery {
            symbol: "A.B".to_owned(),
            direction: Some("outgoing".to_owned()),
            depth: Some(2),
            include_tests: true,
            limit: None,
            target: QueryTarget {
                project: Some("7".to_owned()),
                json: false
            },
        })
    );
}

#[test]
fn impact_and_context_read_their_options() {
    assert!(matches!(
        action(&["codeskimr", "impact", "A.B", "--limit", "9"]),
        QueryAction::Impact(ImpactQuery { limit: Some(9), .. })
    ));
    assert!(matches!(
        action(&["codeskimr", "context", "A.B", "--budget", "long"]),
        QueryAction::Context(ContextQuery { budget: Some(budget), .. }) if budget == "long"
    ));
}

#[test]
fn a_direction_or_budget_outside_the_list_is_a_usage_error() {
    let messages = Messages::load().unwrap();
    for arguments in [
        ["codeskimr", "usages", "A", "--direction", "sideways"],
        ["codeskimr", "context", "A", "--budget", "huge"],
    ] {
        assert!(parse(arguments.iter().copied(), &messages, "en").is_err());
    }
}

#[test]
fn the_json_adds_the_schema_and_the_staleness_to_the_answer() {
    let text = query_json(json!({"hits": []}), Some(true)).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["schema"], 1);
    assert_eq!(value["stale"], true);
    assert_eq!(value["hits"], json!([]));
    let unknown: Value = serde_json::from_str(&query_json(json!({}), None).unwrap()).unwrap();
    assert_eq!(unknown["stale"], Value::Null);
    let error: Value =
        serde_json::from_str(&query_error_json(&json!({"code": "MCP-202"})).unwrap()).unwrap();
    assert_eq!(error["schema"], 1);
    assert_eq!(error["error"]["code"], "MCP-202");
}

#[test]
fn the_text_drops_control_characters_of_names_and_says_when_nothing_was_found() {
    let messages = Messages::load().unwrap();
    let hits = json!({"hits": [{
        "id": 4, "kind": "class", "path": "a.cs", "qualified_name": "App.\u{1b}[31mOrder"
    }]});
    let text = query_text(&messages, "en", QueryKind::Symbols, (&hits, None)).unwrap();
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("App. [31mOrder"));
    let none = query_text(
        &messages,
        "de",
        QueryKind::Symbols,
        (&json!({"hits": []}), None),
    );
    assert_eq!(none.unwrap(), "Nichts gefunden.");
}
