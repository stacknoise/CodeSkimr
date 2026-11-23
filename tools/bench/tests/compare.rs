//! Tests of the comparison logic.

use std::collections::{BTreeMap, BTreeSet};

use codeskimr_bench::compare::{Row, Verdict, compare};
use codeskimr_bench::estimates::Medians;
use codeskimr_bench::report::has_failure;

const TOLERANCE: f64 = 10.0;
const ID: &str = "g/b";

fn medians(ns: Option<f64>) -> Medians {
    ns.map(|ns| (ID.to_owned(), ns)).into_iter().collect()
}

/// Compares `new` with `base` (`None`: no measurement) under an optional budget in milliseconds
/// and whether the slowdown is accepted on purpose.
fn run(new: Option<f64>, base: Option<f64>, budget_ms: Option<f64>, accepted: bool) -> Vec<Row> {
    let budgets: BTreeMap<String, f64> = budget_ms
        .map(|ms| (ID.to_owned(), ms))
        .into_iter()
        .collect();
    let accepted: BTreeSet<String> = accepted.then(|| ID.to_owned()).into_iter().collect();
    compare(
        &medians(new),
        &medians(base),
        (&budgets, &accepted),
        TOLERANCE,
    )
}

fn verdict(rows: &[Row]) -> Option<Verdict> {
    rows.first().map(|row| row.verdict.clone())
}

#[test]
fn within_tolerance_passes() {
    let rows = run(Some(1_090.0), Some(1_000.0), None, false);
    assert_eq!(verdict(&rows), Some(Verdict::Ok));
    assert!(!has_failure(&rows));
}

#[test]
fn regression_over_tolerance_fails() {
    let rows = run(Some(1_200.0), Some(1_000.0), None, false);
    assert!(matches!(verdict(&rows), Some(Verdict::Regression(_))));
    assert!(has_failure(&rows));
}

#[test]
fn speedup_passes() {
    assert!(!has_failure(&run(Some(500.0), Some(1_000.0), None, false)));
}

#[test]
fn over_budget_fails_even_without_base() {
    let rows = run(Some(2_000_000.0), None, Some(1.0), false);
    assert!(matches!(verdict(&rows), Some(Verdict::OverBudget(_))));
}

#[test]
fn missing_base_is_not_a_failure() {
    let rows = run(Some(1_000.0), None, None, false);
    assert_eq!(verdict(&rows), Some(Verdict::NewBench));
    assert!(!has_failure(&rows));
}

#[test]
fn benchmark_missing_in_change_is_reported_not_failed() {
    let rows = run(None, Some(1_000.0), None, false);
    assert_eq!(verdict(&rows), Some(Verdict::Removed));
    assert!(!has_failure(&rows));
}

#[test]
fn an_accepted_slowdown_passes_but_stays_visible() {
    let rows = run(Some(2_000.0), Some(1_000.0), None, true);
    assert!(matches!(verdict(&rows), Some(Verdict::Accepted(_))));
    assert!(!has_failure(&rows));
}

#[test]
fn an_accepted_slowdown_still_has_to_keep_its_budget() {
    assert!(has_failure(&run(
        Some(2_000_000.0),
        Some(1_000.0),
        Some(1.0),
        true
    )));
}
