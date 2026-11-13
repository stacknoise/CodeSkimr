//! Tests of the comparison logic.

use std::collections::BTreeMap;

use codeskimr_bench::compare::{Verdict, compare};
use codeskimr_bench::estimates::Medians;
use codeskimr_bench::report::has_failure;

fn medians(ns: f64) -> Medians {
    BTreeMap::from([("g/b".to_owned(), ns)])
}

#[test]
fn within_tolerance_passes() {
    let rows = compare(&medians(1_090.0), &medians(1_000.0), &BTreeMap::new(), 10.0);
    assert_eq!(rows.first().map(|r| r.verdict.clone()), Some(Verdict::Ok));
    assert!(!has_failure(&rows));
}

#[test]
fn regression_over_tolerance_fails() {
    let rows = compare(&medians(1_200.0), &medians(1_000.0), &BTreeMap::new(), 10.0);
    assert!(matches!(
        rows.first().map(|r| &r.verdict),
        Some(Verdict::Regression(_))
    ));
    assert!(has_failure(&rows));
}

#[test]
fn speedup_passes() {
    let rows = compare(&medians(500.0), &medians(1_000.0), &BTreeMap::new(), 10.0);
    assert!(!has_failure(&rows));
}

#[test]
fn over_budget_fails_even_without_base() {
    let budgets = BTreeMap::from([("g/b".to_owned(), 1.0)]);
    let rows = compare(&medians(2_000_000.0), &Medians::new(), &budgets, 10.0);
    assert!(matches!(
        rows.first().map(|r| &r.verdict),
        Some(Verdict::OverBudget(_))
    ));
}

#[test]
fn missing_base_is_not_a_failure() {
    let rows = compare(&medians(1_000.0), &Medians::new(), &BTreeMap::new(), 10.0);
    assert_eq!(
        rows.first().map(|r| r.verdict.clone()),
        Some(Verdict::NewBench)
    );
    assert!(!has_failure(&rows));
}

#[test]
fn benchmark_missing_in_change_is_reported_not_failed() {
    let rows = compare(&Medians::new(), &medians(1_000.0), &BTreeMap::new(), 10.0);
    assert_eq!(
        rows.first().map(|r| r.verdict.clone()),
        Some(Verdict::Removed)
    );
    assert!(!has_failure(&rows));
}
