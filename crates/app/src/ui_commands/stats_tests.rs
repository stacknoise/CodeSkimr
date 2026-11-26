//! The commands of the statistics screen (ADR 0081), through the path the server takes.

use std::fs;

use serde_json::{Value, json};

use super::tests::Fixture;

/// `Calculator.Add` is used by `Shop.Total`; `Calculator.Spare`, the private `Shop.Hidden` and
/// the test class have no usage. `Total` branches twice.
const SHOP: &str = "namespace Demo;\n\npublic class Shop\n{\n    public int Total(Calculator c, int a)\n    {\n        if (a > 1) { return c.Add(a, 1); }\n        if (a < 0) { return 0; }\n        return c.Add(a, 2);\n    }\n    private int Hidden() { return 1; }\n}\n";
const TEST: &str =
    "namespace Demo.Tests;\n\npublic class ShopChecks\n{\n    public void Runs() { }\n}\n";

pub(super) fn project(fixture: &Fixture) -> u64 {
    let app = fixture.root.path().join("app");
    fs::write(app.join("Shop.cs"), SHOP).unwrap();
    fs::create_dir_all(app.join("tests")).unwrap();
    fs::write(app.join("tests/ShopChecks.cs"), TEST).unwrap();
    fixture.scanned()
}

pub(super) fn ranking(fixture: &Fixture, project: u64, extra: &Value) -> Value {
    let mut input = json!({ "project": project });
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    fixture.call("stats.ranking", input).unwrap()
}

pub(super) fn names(answer: &Value) -> Vec<&str> {
    answer["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect()
}

#[test]
fn the_most_used_ranking_starts_with_the_symbol_that_is_used_most() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let answer = ranking(
        &fixture,
        id,
        &json!({ "ranking": "most_used", "kinds": "functions" }),
    );
    let first = &answer["rows"][0];
    assert_eq!(first["name"], "Add");
    assert_eq!(first["total"], 2);
    assert!(first["certain"].as_u64().unwrap() <= 2);
    assert_eq!(first["assessment"], Value::Null);
    assert!(first["parent"].is_u64());
}

#[test]
fn symbols_without_a_usage_carry_an_assessment_and_tests_are_left_out() {
    // C# is a language whose usages are an approximation in the default configuration.
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let answer = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions" }),
    );
    let found = names(&answer);
    assert!(found.contains(&"Hidden"), "{found:?}");
    assert!(!found.contains(&"Runs"), "{found:?}");
    assert!(!found.contains(&"Add"), "{found:?}");
    assert_eq!(answer["rows"][0]["assessment"], "approximate");
    assert_eq!(answer["rows"][0]["reason"], "approximate");
    assert_eq!(answer["unused"]["tests_hidden"], 1);
    let with_tests = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions", "include_tests": true }),
    );
    assert!(names(&with_tests).contains(&"Runs"));
    assert_eq!(with_tests["unused"]["tests_hidden"], 0);
}

#[test]
fn the_assessment_follows_the_visibility_in_a_language_with_exact_usages() {
    let exact = "[ui]\napproximate_languages = [\"python\"]\n";
    let fixture = Fixture::installed_with(None, Some(exact));
    let id = project(&fixture);
    let likely = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions", "assessment": "likely" }),
    );
    assert_eq!(names(&likely), ["Hidden"]);
    assert_eq!(likely["rows"][0]["reason"], "private");
    assert_eq!(likely["total"], 1);
    assert!(likely["unused"]["public"].as_u64().unwrap() >= 1);
}

#[test]
fn the_complexity_ranking_holds_callables_with_their_rating_and_keeps_the_limit() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let answer = ranking(
        &fixture,
        id,
        &json!({ "ranking": "complexity", "limit": 1 }),
    );
    assert_eq!(names(&answer), ["Total"]);
    assert_eq!(answer["rows"][0]["complexity"], 3.0);
    assert_eq!(answer["rows"][0]["complexity_rating"], "good");
    assert!(answer["total"].as_u64().unwrap() > 1);
}

#[test]
fn the_overview_counts_usages_unused_symbols_and_complexity() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let answer = fixture
        .call("stats.overview", json!({ "project": id }))
        .unwrap();
    assert!(answer["snapshot"].is_u64());
    assert_eq!(answer["files"], 3);
    let references = &answer["references"];
    assert!(
        references["certain"].as_u64().unwrap() + references["probable"].as_u64().unwrap() >= 2
    );
    assert!(answer["unused"]["approximate"].as_u64().unwrap() >= 2);
    assert_eq!(answer["complexity"]["limit"], 10);
    assert_eq!(answer["complexity"]["over_limit"], 0);
    assert_eq!(answer["top"]["complexity"][0]["name"], "Total");
    let bars = answer["complexity"]["histogram"].as_array().unwrap();
    assert!(
        bars.iter()
            .any(|bar| bar["complexity"] == 3 && bar["count"] == 1)
    );
}

#[test]
fn a_project_without_a_scan_has_empty_statistics() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    let answer = ranking(&fixture, project, &json!({ "ranking": "unused" }));
    assert_eq!(answer["snapshot"], Value::Null);
    assert_eq!(answer["rows"], json!([]));
    assert_eq!(answer["total"], 0);
    let overview = fixture
        .call("stats.overview", json!({ "project": project }))
        .unwrap();
    assert_eq!(overview["snapshot"], Value::Null);
    assert_eq!(overview["top"]["most_used"], json!([]));
    assert_eq!(overview["complexity"]["histogram"], json!([]));
}

#[test]
fn an_unknown_project_is_refused() {
    let fixture = Fixture::installed(None);
    let input = json!({ "project": 99, "ranking": "size" });
    assert_eq!(fixture.call("stats.ranking", input), Err("PRJ-201"));
    assert_eq!(
        fixture.call("stats.overview", json!({ "project": 99 })),
        Err("PRJ-201")
    );
}

#[test]
fn commands_that_run_at_once_answer_as_they_do_one_after_the_other() {
    // The screen asks for the overview and a ranking together; the commands load the symbols one
    // at a time (a copy of them is several gigabytes for a large project) and must not disturb
    // each other.
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let rank = json!({ "ranking": "most_used", "kinds": "functions" });
    let alone = (
        fixture
            .call("stats.overview", json!({ "project": id }))
            .unwrap(),
        ranking(&fixture, id, &rank),
    );
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|turn| {
                let (fixture, rank) = (&fixture, &rank);
                scope.spawn(move || {
                    if turn % 2 == 0 {
                        fixture
                            .call("stats.overview", json!({ "project": id }))
                            .unwrap()
                    } else {
                        ranking(fixture, id, rank)
                    }
                })
            })
            .collect();
        for (turn, handle) in handles.into_iter().enumerate() {
            let answer = handle.join().unwrap();
            let expected = if turn % 2 == 0 { &alone.0 } else { &alone.1 };
            assert_eq!(&answer, expected);
        }
    });
}

#[test]
fn an_entry_of_the_exception_list_takes_the_symbols_out_of_the_ranking_and_the_counts() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let before = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions" }),
    );
    assert!(names(&before).contains(&"Hidden"));
    assert_eq!(before["left_out"], json!([]));
    let list = "[[unused.ignore]]\nsymbols = [\"*.Hidden\"]\nreason = \"called by reflection\"\n";
    fs::write(fixture.root.path().join("app/.codeskimr.toml"), list).unwrap();
    let after = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions" }),
    );
    assert!(!names(&after).contains(&"Hidden"), "{after}");
    assert_eq!(after["total"], before["total"].as_u64().unwrap() - 1);
    assert_eq!(after["unused"]["left_out"], 1);
    assert_eq!(after["left_out"][0]["row"]["name"], "Hidden");
    assert_eq!(
        after["left_out"][0]["rule"]["reason"],
        "called by reflection"
    );
    let overview = fixture
        .call("stats.overview", json!({ "project": id }))
        .unwrap();
    assert_eq!(overview["unused"]["left_out"], 1);
}

#[test]
fn the_overview_drops_a_symbol_of_the_exception_list_from_its_cards_and_first_rows() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let plain = fixture
        .call("stats.overview", json!({ "project": id }))
        .unwrap();
    let list = "[[unused.ignore]]\nsymbols = [\"*.Hidden\"]\nreason = \"called by reflection\"\n";
    fs::write(fixture.root.path().join("app/.codeskimr.toml"), list).unwrap();
    let overview = fixture
        .call("stats.overview", json!({ "project": id }))
        .unwrap();
    // The cards and the first rows of the overview drop the symbol as well.
    let sum = |value: &Value| {
        ["likely", "public", "approximate"]
            .iter()
            .map(|key| value["unused"][key].as_u64().unwrap())
            .sum::<u64>()
    };
    assert_eq!(sum(&overview), sum(&plain) - 1);
    let listed = |value: &Value| {
        value["top"]["unused"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["name"] == "Hidden")
    };
    assert!(listed(&plain));
    assert!(!listed(&overview));
}

#[test]
fn the_first_entry_that_covers_a_symbol_names_it_and_both_lists_of_an_entry_must_match() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let list = "[[unused.ignore]]\npaths = [\"tests/*\"]\nsymbols = [\"*.Hidden\"]\nreason = \"first\"\n\n[[unused.ignore]]\nsymbols = [\"*.Hidden\"]\nreason = \"second\"\n\n[[unused.ignore]]\nsymbols = [\"*.Hidden\"]\nreason = \"third\"\n";
    fs::write(fixture.root.path().join("app/.codeskimr.toml"), list).unwrap();
    let answer = ranking(
        &fixture,
        id,
        &json!({ "ranking": "unused", "kinds": "functions" }),
    );
    // `Hidden` is not under tests/, so the first entry does not match, the second does.
    assert_eq!(answer["left_out"][0]["rule"]["reason"], "second");
    assert_eq!(answer["unused"]["left_out"], 1);
}

#[test]
fn a_broken_exception_list_fails_with_the_code_of_the_scan_and_hides_nothing() {
    let fixture = Fixture::installed(None);
    let id = project(&fixture);
    let file = fixture.root.path().join("app/.codeskimr.toml");
    // An entry needs a reason (CFG-101), a file that is no TOML is CFG-102.
    for (text, code) in [
        ("[[unused.ignore]]\nsymbols = [\"*.Hidden\"]\n", "CFG-101"),
        ("[[unused.ignore]\n", "CFG-102"),
    ] {
        fs::write(&file, text).unwrap();
        let input = json!({ "project": id, "ranking": "unused", "kinds": "functions" });
        assert_eq!(fixture.call("stats.ranking", input), Err(code), "{text}");
        assert_eq!(
            fixture.call("stats.overview", json!({ "project": id })),
            Err(code),
            "{text}"
        );
    }
}
