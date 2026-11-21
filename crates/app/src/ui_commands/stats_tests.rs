//! The commands of the statistics screen (ADR 0081), through the path the server takes.

use std::fs;

use serde_json::{Value, json};

use super::tests::Fixture;

/// `Calculator.Add` is used by `Shop.Total`; `Calculator.Spare`, the private `Shop.Hidden` and
/// the test class have no usage. `Total` branches twice.
const SHOP: &str = "namespace Demo;\n\npublic class Shop\n{\n    public int Total(Calculator c, int a)\n    {\n        if (a > 1) { return c.Add(a, 1); }\n        if (a < 0) { return 0; }\n        return c.Add(a, 2);\n    }\n    private int Hidden() { return 1; }\n}\n";
const TEST: &str =
    "namespace Demo.Tests;\n\npublic class ShopChecks\n{\n    public void Runs() { }\n}\n";

fn project(fixture: &Fixture) -> u64 {
    let app = fixture.root.path().join("app");
    fs::write(app.join("Shop.cs"), SHOP).unwrap();
    fs::create_dir_all(app.join("tests")).unwrap();
    fs::write(app.join("tests/ShopChecks.cs"), TEST).unwrap();
    fixture.scanned()
}

fn ranking(fixture: &Fixture, project: u64, extra: &Value) -> Value {
    let mut input = json!({ "project": project });
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    fixture.call("stats.ranking", input).unwrap()
}

fn names(answer: &Value) -> Vec<&str> {
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
