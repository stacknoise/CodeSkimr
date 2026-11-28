//! `codeskimr scans`, `compare` and `change` over a project with two scans (ADR 0097): the JSON a
//! script reads, the text a person reads and the Markdown of `change --markdown`.

use std::fs;

use codeskimr_cli::{
    ChangeFilterArgs, CompareQuery, MetricsOptions, MetricsSide, QueryAction, QueryTarget,
    ScansQuery,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ScanMode;
use serde_json::Value;

use crate::query_cli_tests::World;

const CHANGED: &str = "namespace Demo;\n\npublic class Target\n{\n    public static int Do(int x, int y)\n    {\n        return x + y;\n    }\n}\n";
/// The first line of `change --markdown`.
const MARKER: &str = "<!-- codeskimr-change -->";

/// The demo project of the query tests, with a second scan in which `Target.Do` got another
/// parameter list.
fn two_scans() -> World {
    let world = World::new();
    fs::write(world.folder.join("Target.cs"), CHANGED).unwrap();
    world.session.scan_quick(ScanMode::Incremental).unwrap();
    world
}

impl World {
    fn text(&self, action: &QueryAction) -> String {
        self.run(action, "en").unwrap()
    }
}

fn target(json: bool) -> QueryTarget {
    QueryTarget {
        project: None,
        json,
    }
}

fn compare(json: bool) -> QueryAction {
    QueryAction::Compare(query(json, false))
}

fn change(json: bool, markdown: bool) -> QueryAction {
    QueryAction::Change(query(json, markdown))
}

fn query(json: bool, markdown: bool) -> CompareQuery {
    CompareQuery {
        filter: ChangeFilterArgs::default(),
        base: None,
        newer: None,
        markdown,
        metrics: None,
        target: target(json),
    }
}

#[test]
fn the_scans_are_listed_newest_first_in_json_and_text() {
    let world = two_scans();
    let scans = QueryAction::Scans(ScansQuery {
        target: target(true),
    });
    let answer = world.json(&scans);
    assert_eq!(answer["schema"], 1);
    assert_eq!(answer["scans"].as_array().unwrap().len(), 2);
    let text = world.text(&QueryAction::Scans(ScansQuery {
        target: target(false),
    }));
    assert!(text.contains("2 kept"), "{text}");
    assert!(text.contains("incremental"), "{text}");
}

#[test]
fn compare_lists_the_symbols_but_not_the_usages() {
    let world = two_scans();
    let answer = world.json(&compare(true));
    assert_eq!(answer["impact"], Value::Null);
    assert_eq!(
        answer["removed"]["items"][0]["qualified_name"],
        "Demo.Target.Do"
    );
    assert_eq!(answer["added"]["total"], 1);
    let text = world.text(&compare(false));
    assert!(text.contains("1 added, 1 removed"), "{text}");
    assert!(text.contains("Demo.Target.Do"), "{text}");
}

#[test]
fn change_names_the_usage_that_was_not_touched_and_the_limit_of_what_it_sees() {
    let world = two_scans();
    let answer = world.json(&change(true, false));
    assert_eq!(answer["direct_usages_only"], true);
    assert!(answer["note"].as_str().unwrap().contains("direct usages"));
    assert_eq!(answer["summary"]["usages_not_touched"], 1);
    let text = world.text(&change(false, false));
    assert!(text.contains("not touched"), "{text}");
    assert!(text.contains("Demo.Caller.Run"), "{text}");
    assert!(text.contains("Only direct usages are known"), "{text}");
}

#[test]
fn the_text_follows_the_language() {
    let world = two_scans();
    let german = world.run(&change(false, false), "de").unwrap();
    assert!(german.contains("nicht angefasst"), "{german}");
    assert!(
        german.contains("nur direkte Verwendungen")
            || german.contains("Bekannt sind nur direkte Verwendungen"),
        "{german}"
    );
}

#[test]
fn the_markdown_has_the_marker_the_numbers_the_checklist_and_the_limit() {
    let world = two_scans();
    let markdown = world.text(&change(false, true));
    let lines: Vec<&str> = markdown.lines().collect();
    assert_eq!(lines[0], MARKER);
    assert!(
        markdown.contains("- Changed symbols: 2, of them public: 2"),
        "{markdown}"
    );
    assert!(markdown.contains("- Usages: 1 sure"), "{markdown}");
    assert!(markdown.contains("- Not touched: 1"), "{markdown}");
    let item = lines
        .iter()
        .find(|line| line.starts_with("- [ ] "))
        .unwrap();
    assert!(item.contains("`Caller.cs:7`"), "{item}");
    assert!(item.contains("`Demo.Target.Do`"), "{item}");
    assert!(item.contains("`Demo.Caller.Run`"), "{item}");
    assert!(
        lines
            .last()
            .unwrap()
            .starts_with("> Only direct usages are known"),
        "{markdown}"
    );
    let german = world.run(&change(false, true), "de").unwrap();
    assert!(german.starts_with(MARKER), "{german}");
    assert!(german.contains("- [ ] "), "{german}");
}

#[test]
fn one_scan_has_nothing_to_compare() {
    let world = World::new();
    let failure = world.run(&compare(true), "en").unwrap_err();
    assert_eq!(failure.code(), ErrorCode::SnapshotNotFound);
    let scans = world.text(&QueryAction::Scans(ScansQuery {
        target: target(false),
    }));
    assert!(scans.contains("1 kept"), "{scans}");
}

fn metrics(json: bool, level: &str) -> QueryAction {
    let mut found = query(json, false);
    found.metrics = Some(MetricsOptions {
        side: MetricsSide::Both,
        only: Vec::new(),
        level: Some(level.to_owned()),
        include_tests: false,
    });
    QueryAction::Compare(found)
}

#[test]
fn compare_metrics_answers_in_json_and_text_and_refuses_an_unknown_level() {
    let world = two_scans();
    let answer = world.json(&metrics(true, "member"));
    assert_eq!(answer["schema"], 1);
    assert_eq!(answer["level"], "member");
    assert!(answer["worse"]["items"].is_array());
    let text = world.text(&metrics(false, "type"));
    assert!(text.contains("worse"), "{text}");
    let failure = world.run(&metrics(true, "galaxy"), "en").unwrap_err();
    assert_eq!(failure.code(), ErrorCode::RequestInvalid);
}

#[test]
fn compare_metrics_prints_the_markdown_for_a_comment() {
    let world = two_scans();
    let QueryAction::Compare(mut found) = metrics(false, "member") else {
        panic!("not a compare");
    };
    found.markdown = true;
    let text = world.text(&QueryAction::Compare(found));
    assert!(text.starts_with("<!-- codeskimr-metrics -->"), "{text}");
    assert!(text.contains("metric changes from scan"), "{text}");
}
