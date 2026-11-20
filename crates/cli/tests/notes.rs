//! `codeskimr annotations` and `codeskimr alerts` (ADR 0068): what the parser makes of them.

use codeskimr_cli::{
    Action, AlertsAction, AnnotationsAction, CliError, Invocation, Parsed, SymbolArg, parse,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::Messages;

#[cfg(test)]
fn run_in(language: &str, arguments: &[&str]) -> Result<Parsed, CliError> {
    let messages = Messages::load().unwrap();
    parse(arguments.iter().copied(), &messages, language)
}

#[cfg(test)]
fn invocation(arguments: &[&str]) -> Invocation {
    match run_in("en", arguments).unwrap() {
        Parsed::Run(invocation) => invocation,
        Parsed::Display(text) => panic!("unexpected output: {text}"),
    }
}

#[cfg(test)]
fn annotations(arguments: &[&str]) -> AnnotationsAction {
    match invocation(arguments).action {
        Action::Annotations(action) => *action,
        other => panic!("annotations expected, got {other:?}"),
    }
}

#[cfg(test)]
fn alerts(arguments: &[&str]) -> AlertsAction {
    match invocation(arguments).action {
        Action::Alerts(action) => *action,
        other => panic!("alerts expected, got {other:?}"),
    }
}

#[test]
fn add_takes_a_symbol_a_kind_and_the_fields() {
    let AnnotationsAction::Add(add) = annotations(&[
        "codeskimr",
        "annotations",
        "add",
        "7",
        "--symbol",
        "App.Order.Total",
        "--symbol-kind",
        "method",
        "--param",
        "int",
        "--kind",
        "task",
        "--title",
        "Split it",
        "--status",
        "open",
        "--due",
        "1700000000",
        "--json",
    ]) else {
        panic!("add expected");
    };
    assert_eq!(add.project, "7");
    assert_eq!(
        add.symbol,
        SymbolArg {
            kind: "method".to_owned(),
            name: "App.Order.Total".to_owned(),
            parameters: vec!["int".to_owned()],
        }
    );
    assert_eq!(add.kind, "task");
    assert_eq!(add.fields.title.as_deref(), Some("Split it"));
    assert_eq!(add.fields.due, Some(1_700_000_000));
    assert!(add.json);
}

#[test]
fn add_without_a_symbol_or_a_kind_is_refused() {
    let error = run_in("en", &["codeskimr", "annotations", "add", "7"]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CliArgumentsInvalid);
    let unknown = run_in(
        "en",
        &[
            "codeskimr",
            "annotations",
            "add",
            "7",
            "--symbol",
            "A",
            "--symbol-kind",
            "class",
            "--kind",
            "memo",
        ],
    )
    .unwrap_err();
    assert_eq!(unknown.code(), ErrorCode::CliArgumentsInvalid);
}

#[test]
fn update_tells_setting_from_clearing() {
    let AnnotationsAction::Update(update) = annotations(&[
        "codeskimr",
        "annotations",
        "update",
        "7",
        "3",
        "--title",
        "New",
        "--clear-text",
        "--clear-assignee",
    ]) else {
        panic!("update expected");
    };
    assert_eq!(update.id, 3);
    assert_eq!(update.fields.title.as_deref(), Some("New"));
    assert!(update.clears.text);
    assert!(update.clears.assignee);
    assert!(!update.clears.due);
    assert!(update.symbol.is_none());
}

#[test]
fn list_and_remove_take_filters_and_a_page() {
    let AnnotationsAction::List(list) = annotations(&[
        "codeskimr",
        "annotations",
        "list",
        "https://example.com/repo.git",
        "--kind",
        "critical",
        "--limit",
        "5",
        "--offset",
        "10",
    ]) else {
        panic!("list expected");
    };
    assert_eq!(list.project, "https://example.com/repo.git");
    assert_eq!(list.kind.as_deref(), Some("critical"));
    assert_eq!(list.page.limit, Some(5));
    assert_eq!(list.page.offset, Some(10));
    assert!(list.symbol.is_none());
    let AnnotationsAction::Remove(remove) =
        annotations(&["codeskimr", "annotations", "remove", "7", "4"])
    else {
        panic!("remove expected");
    };
    assert_eq!((remove.id, remove.json), (4, false));
}

#[test]
fn alerts_list_dismiss_and_dismiss_all_are_parsed() {
    let AlertsAction::List(list) = alerts(&[
        "codeskimr",
        "alerts",
        "list",
        "7",
        "--state",
        "dismissed",
        "--kind",
        "removed",
        "--kind",
        "orphaned",
    ]) else {
        panic!("list expected");
    };
    assert_eq!(list.state.as_deref(), Some("dismissed"));
    assert_eq!(list.kinds, ["removed", "orphaned"]);
    let AlertsAction::Dismiss(dismiss) = alerts(&["codeskimr", "alerts", "dismiss", "7", "2"])
    else {
        panic!("dismiss expected");
    };
    assert_eq!(dismiss.id, 2);
    let AlertsAction::DismissAll(all) = alerts(&["codeskimr", "alerts", "dismiss-all", "7"]) else {
        panic!("dismiss-all expected");
    };
    assert!(!all.confirm, "without --yes nothing is confirmed");
    let AlertsAction::DismissAll(sure) =
        alerts(&["codeskimr", "alerts", "dismiss-all", "7", "--yes"])
    else {
        panic!("dismiss-all expected");
    };
    assert!(sure.confirm);
}

#[test]
fn the_help_exists_in_both_languages() {
    for language in ["en", "de"] {
        for arguments in [
            ["codeskimr", "annotations", "--help"],
            ["codeskimr", "alerts", "--help"],
        ] {
            let Parsed::Display(text) = run_in(language, &arguments).unwrap() else {
                panic!("help expected");
            };
            assert!(text.contains("list"), "{language}: {text}");
        }
    }
}
