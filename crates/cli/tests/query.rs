//! The query commands: their arguments, and the JSON and text they print.

use codeskimr_cli::{
    Action, CompareQuery, ContextQuery, ImpactQuery, ModulesQuery, Parsed, QueryAction, QueryKind,
    QueryTarget, RenameQuery, ScansQuery, SymbolsQuery, UnusedQuery, UsagesQuery, change_markdown,
    parse, query_code_report, query_error_json, query_json, query_text,
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
            {"kind": "declaration", "path": "Order.cs", "line": 5, "column": 3,
             "code": "public void Place() {"},
            {"kind": "usage", "path": "Api.cs", "line": 41, "column": 9,
             "confidence": "certain", "from": {"qualified_name": "Shop.Api.Post"},
             "code": "order.Place();"}
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
        "    public void Place() {",
        "    order.Place();",
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

fn scans_target(project: Option<&str>, json: bool) -> QueryTarget {
    QueryTarget {
        project: project.map(str::to_owned),
        json,
    }
}

#[test]
fn scans_takes_the_project_as_its_first_argument() {
    assert_eq!(
        action(&["codeskimr", "scans", "shop", "--json"]),
        QueryAction::Scans(ScansQuery {
            target: scans_target(Some("shop"), true),
        })
    );
    assert_eq!(
        action(&["codeskimr", "scans"]),
        QueryAction::Scans(ScansQuery {
            target: scans_target(None, false),
        })
    );
}

#[test]
fn compare_and_change_read_the_two_scans() {
    let expected = CompareQuery {
        base: Some(4),
        newer: Some(9),
        markdown: false,
        target: scans_target(Some("shop"), false),
    };
    let arguments = ["shop", "--base", "4", "--target", "9"];
    assert_eq!(
        action(&[
            "codeskimr",
            "compare",
            arguments[0],
            arguments[1],
            arguments[2],
            arguments[3],
            arguments[4]
        ]),
        QueryAction::Compare(expected.clone())
    );
    assert_eq!(
        action(&[
            "codeskimr",
            "change",
            arguments[0],
            arguments[1],
            arguments[2],
            arguments[3],
            arguments[4]
        ]),
        QueryAction::Change(expected)
    );
}

#[test]
fn change_without_options_takes_the_newest_scan_and_the_one_before() {
    assert_eq!(
        action(&["codeskimr", "change"]),
        QueryAction::Change(CompareQuery {
            base: None,
            newer: None,
            markdown: false,
            target: scans_target(None, false),
        })
    );
}

#[test]
fn markdown_is_a_flag_of_change_that_excludes_json() {
    let QueryAction::Change(query) = action(&["codeskimr", "change", "--markdown"]) else {
        panic!("not a change");
    };
    assert!(query.markdown);
    let messages = Messages::load().unwrap();
    let both = parse(
        ["codeskimr", "change", "--markdown", "--json"],
        &messages,
        "en",
    );
    assert!(both.is_err(), "--json and --markdown exclude each other");
    let compare = parse(["codeskimr", "compare", "--markdown"], &messages, "en");
    assert!(compare.is_err(), "only change prints Markdown");
}

#[test]
fn a_name_cannot_break_out_of_the_code_span_of_the_markdown() {
    let answer = json!({
        "base": {"snapshot": 1}, "target": {"snapshot": 2},
        "summary": {"changed_symbols": 1, "public_symbols": 0, "usages_certain": 1,
                    "usages_probable": 0, "usages_not_touched": 1},
        "is_cut": true,
        "impact": {"items": [{
            "symbol": {"qualified_name": "A.`evil`\n# heading"},
            "cause": "removed",
            "usages": {"items": [{
                "from": {"qualified_name": "B.run"},
                "path": "b.cs", "line": 3, "kind": "call", "status": "not_touched"
            }]}
        }]},
    });
    let messages = Messages::load().unwrap();
    let markdown = change_markdown(&messages, "en", &answer).unwrap();
    let item = markdown
        .lines()
        .find(|line| line.starts_with("- [ ] "))
        .unwrap();
    assert!(item.contains("`A.'evil' # heading`"), "{item}");
    assert!(
        markdown.contains("cut to the configured limit"),
        "{markdown}"
    );
    assert_eq!(markdown.lines().next(), Some("<!-- codeskimr-change -->"));
}

#[test]
fn modules_reads_the_depth_the_limit_and_the_output() {
    assert_eq!(
        action(&[
            "codeskimr",
            "modules",
            "--depth",
            "2",
            "--limit",
            "20",
            "--json"
        ]),
        QueryAction::Modules(ModulesQuery {
            depth: Some(2),
            limit: Some(20),
            from: None,
            to: None,
            target: QueryTarget {
                project: None,
                json: true
            },
        })
    );
}

#[test]
fn modules_reads_the_two_modules_of_a_dependency_and_wants_both() {
    let QueryAction::Modules(query) = action(&[
        "codeskimr",
        "modules",
        "--from",
        "App.Web",
        "--to",
        "App.Domain",
    ]) else {
        unreachable!("a modules command")
    };
    assert_eq!(query.from.as_deref(), Some("App.Web"));
    assert_eq!(query.to.as_deref(), Some("App.Domain"));
    let messages = Messages::load().unwrap();
    let alone = parse(
        ["codeskimr", "modules", "--from", "App.Web"],
        &messages,
        "en",
    );
    assert!(alone.is_err());
}

#[test]
fn the_text_of_the_places_of_a_dependency_lists_file_line_and_symbols_in_both_languages() {
    let messages = Messages::load().unwrap();
    let answer = json!({
        "level": "module", "from": "App.Web", "to": "App.Domain", "total": 3,
        "places": [
            {"path": "web/Page.cs", "line": 8, "from": "App.Web.Page.Show", "to": "App.Domain.Invoice.Total"},
            {"path": "web/Page.cs", "line": 20, "from": "App.Web.Page.Show", "to": "App.Domain.Invoice.Pay"}
        ]
    });
    let english = query_text(&messages, "en", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(
        english.contains("Places of App.Web -> App.Domain: 2 of 3 shown."),
        "{english}"
    );
    assert!(
        english.contains("web/Page.cs:8  App.Web.Page.Show -> App.Domain.Invoice.Total"),
        "{english}"
    );
    let german = query_text(&messages, "de", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(
        german.contains("Stellen von App.Web -> App.Domain: 2 von 3 gezeigt."),
        "{german}"
    );
    let none = json!({"from": "A", "to": "B", "total": 0, "places": []});
    let empty = query_text(&messages, "en", QueryKind::Modules, (&none, None)).unwrap();
    assert_eq!(empty, "No places of A -> B found.");
}

fn module_answer() -> Value {
    json!({
        "level": "module",
        "total": 3,
        "modules": [
            {"name": "App.Domain", "layer": "Domain", "incoming": 11, "outgoing": 2,
             "stability": 154, "types": 2, "in_cycle": true, "violations": 0},
            {"name": "App.Web", "layer": null, "incoming": 0, "outgoing": 7,
             "stability": 1000, "types": 1, "in_cycle": false, "violations": 1},
            {"name": "App.Lonely", "layer": null, "incoming": 0, "outgoing": 0,
             "stability": null, "types": 1, "in_cycle": false, "violations": 0}
        ],
        "cycles": [{
            "modules": ["App.Domain", "App.Application"],
            "edges": [],
            "resolution": [{"from": "App.Domain", "to": "App.Application", "count": 1}]
        }],
        "violations": [{
            "edge": {"from": "App.Web", "to": "App.Domain", "count": 4},
            "from_layer": "Web", "to_layer": "Domain"
        }],
        "risks": [{
            "module": "App.Domain", "incoming": 11, "stability": 154,
            "depends_on": "App.Web", "depends_on_stability": 1000
        }]
    })
}

#[test]
fn the_text_of_modules_names_rings_violations_and_risks_in_both_languages() {
    let messages = Messages::load().unwrap();
    let answer = module_answer();
    let english = query_text(&messages, "en", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(english.contains("App.Domain [Domain]"), "{english}");
    assert!(english.contains("stability 0.15"), "{english}");
    assert!(english.contains("stability none"), "{english}");
    assert!(english.contains("in a cycle"), "{english}");
    assert!(
        english.contains("2 modules in a ring: App.Domain -> App.Application"),
        "{english}"
    );
    assert!(
        english.contains("Ending it takes away App.Domain -> App.Application (1 place)"),
        "{english}"
    );
    assert!(
        english.contains("App.Web (Web) -> App.Domain (Domain): 4 places"),
        "{english}"
    );
    assert!(english.contains("is used in 11 places"), "{english}");
    let german = query_text(&messages, "de", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(german.contains("Stabilität 0,15"), "{german}");
    assert!(german.contains("im Zyklus"), "{german}");
    assert!(german.contains("1 Stelle"), "{german}");
}

#[test]
fn the_text_of_modules_rounds_the_stability_like_the_ui() {
    let messages = Messages::load().unwrap();
    let mut answer = module_answer();
    answer["modules"][0]["stability"] = json!(995);
    answer["modules"][1]["stability"] = json!(999);
    let english = query_text(&messages, "en", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(english.contains("stability 1.00"), "{english}");
    assert!(!english.contains("stability 0.99"), "{english}");
    let german = query_text(&messages, "de", QueryKind::Modules, (&answer, None)).unwrap();
    assert!(german.contains("Stabilität 1,00"), "{german}");
}

#[test]
fn the_text_of_modules_says_when_there_are_none() {
    let messages = Messages::load().unwrap();
    let none = json!({"modules": [], "total": 0});
    let text = query_text(&messages, "en", QueryKind::Modules, (&none, None)).unwrap();
    assert_eq!(text, "No modules found.");
}
