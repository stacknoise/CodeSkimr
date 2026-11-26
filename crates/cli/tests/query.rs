//! The query commands: their arguments, and the JSON and text they print.

use codeskimr_cli::{
    Action, ContextQuery, ImpactQuery, Parsed, QueryAction, QueryKind, QueryTarget, RenameQuery,
    SymbolsQuery, UnusedQuery, UsagesQuery, parse, query_code_report, query_error_json, query_json,
    query_text,
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
        serde_json::from_str(&query_error_json(&json!({"code": "MCP-202"}))).unwrap();
    assert_eq!(error["schema"], 1);
    assert_eq!(error["error"]["code"], "MCP-202");
    assert_eq!(query_code_report("MCP-202"), json!({"code": "MCP-202"}));
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

#[test]
fn unused_reads_its_options_and_refuses_values_outside_its_lists() {
    assert_eq!(
        action(&[
            "codeskimr",
            "unused",
            "--kind",
            "functions",
            "--language",
            "csharp",
            "--assessment",
            "likely",
            "--include-tests",
            "--limit",
            "3",
        ]),
        QueryAction::Unused(UnusedQuery {
            kind: Some("functions".to_owned()),
            language: Some("csharp".to_owned()),
            assessment: Some("likely".to_owned()),
            include_tests: true,
            limit: Some(3),
            target: QueryTarget::default(),
        })
    );
    let messages = Messages::load().unwrap();
    for arguments in [
        ["codeskimr", "unused", "--assessment", "dead"],
        ["codeskimr", "unused", "--kind", "everything"],
    ] {
        assert!(parse(arguments.iter().copied(), &messages, "en").is_err());
    }
}

#[test]
fn the_text_of_unused_names_the_reason_and_lists_what_the_exception_list_left_out() {
    let messages = Messages::load().unwrap();
    let answer = json!({
        "counts": {"left_out": 1},
        "unused": [{
            "kind": "function", "qualified_name": "Demo.Shop.Hidden", "path": "Shop.cs",
            "assessment": "likely", "reason": "private"
        }],
        "left_out": [{
            "kind": "function", "qualified_name": "Demo.Shop.Migrate", "path": "Shop.cs",
            "rule": {"reason": "called by the migration tool"}
        }]
    });
    let text = query_text(&messages, "en", QueryKind::Unused, (&answer, None)).unwrap();
    assert!(
        text.contains("Demo.Shop.Hidden  Shop.cs  (likely, private)"),
        "{text}"
    );
    assert!(
        text.contains("Left out by the exception list (1 of 1):"),
        "{text}"
    );
    assert!(
        text.contains("because: called by the migration tool"),
        "{text}"
    );
    let none = query_text(&messages, "de", QueryKind::Unused, (&json!({}), None));
    assert_eq!(none.unwrap(), "Nichts gefunden.");
}

#[test]
fn preview_rename_takes_the_symbol_and_the_new_name_and_both_are_required() {
    assert_eq!(
        action(&[
            "codeskimr",
            "preview-rename",
            "Shop.Order.Place",
            "Submit",
            "--limit",
            "5"
        ]),
        QueryAction::Rename(RenameQuery {
            symbol: "Shop.Order.Place".to_owned(),
            new_name: "Submit".to_owned(),
            limit: Some(5),
            target: QueryTarget::default(),
        })
    );
    let messages = Messages::load().unwrap();
    for arguments in [
        vec!["codeskimr", "preview-rename"],
        vec!["codeskimr", "preview-rename", "Shop.Order.Place"],
    ] {
        assert!(parse(arguments.iter().copied(), &messages, "en").is_err());
    }
}

#[test]
fn the_text_of_a_rename_preview_says_what_it_cannot_see() {
    let messages = Messages::load().unwrap();
    let answer = json!({
        "symbol": {"qualified_name": "Shop.Order.Place"},
        "new_name": "Submit",
        "public_interface": true,
        "sites": {"total": 2, "files": 2, "verified": 1, "items": [
            {"kind": "declaration", "path": "Order.cs", "line": 5, "column": 3},
            {"kind": "usage", "path": "Api.cs", "line": 41, "column": 9,
             "confidence": "certain", "from": {"qualified_name": "Shop.Api.Post"}}
        ]},
        "conflicts": [{"kind": "method", "qualified_name": "Shop.Order.Submit",
                       "path": "Order.cs", "same_signature": true}],
        "text_hits": {"searched": true, "total": 1, "items": [
            {"path": "appsettings.json", "line": 18, "kind": "configuration", "text": "\"Place\""}
        ]}
    });
    let text = query_text(&messages, "en", QueryKind::Rename, (&answer, None)).unwrap();
    for expected in [
        "Rename preview: Shop.Order.Place -> Submit. CodeSkimr changes nothing.",
        "Places that would have to change: 2 in 2 files (1 checked by the indexer)",
        "Order.cs:5:3  declaration",
        "Api.cs:41:9",
        "Name conflicts (1 of 1):",
        "same signature: the two clash",
        "appsettings.json:18  configuration",
        "The symbol is public.",
        "does not say whether the code compiles",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    let unsearched = json!({"sites": {}, "text_hits": {"searched": false}});
    let text = query_text(&messages, "de", QueryKind::Rename, (&unsearched, None)).unwrap();
    assert!(text.contains("nicht nach Text durchsucht"), "{text}");
}
