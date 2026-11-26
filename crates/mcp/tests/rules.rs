//! `check_rules` against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{PermissionSet, Principal};
use codeskimr_mcp::RulesArgs;
use common::tools;

#[cfg(test)]
fn args(limit: Option<u32>) -> RulesArgs {
    RulesArgs {
        project: None,
        limit,
    }
}

#[test]
fn a_rule_lists_its_violations_with_places_and_marks_the_held_one_old() {
    let answer = tools()
        .check_rules(&Principal::local_owner(), &args(None))
        .unwrap();
    assert_eq!(answer["old"], 1);
    assert_eq!(answer["new"], 1);
    assert_eq!(answer["resolved"], 0);
    assert_eq!(answer["held_on"], "2026-11-25");
    let rule = &answer["rules"][0];
    assert_eq!(rule["id"], "forbid:Shop.Reports>Shop.Checkout");
    assert_eq!(rule["kind"], "forbid");
    assert_eq!(rule["total"], 2);
    assert_eq!(rule["new"], 1);
    let fresh = &rule["violations"][0];
    assert_eq!(fresh["code"], "CHK-107");
    assert_eq!(fresh["old"], false);
    assert_eq!(fresh["from"], "Shop.Reports.A.Fresh");
    assert_eq!(fresh["places"][0]["path"], "Reports/A.cs");
    assert_eq!(fresh["places"][0]["line"], 10);
    assert_eq!(rule["violations"][1]["old"], true);
}

#[test]
fn a_limit_cuts_the_list_of_a_rule_but_not_the_counts() {
    let answer = tools()
        .check_rules(&Principal::local_owner(), &args(Some(1)))
        .unwrap();
    let rule = &answer["rules"][0];
    assert_eq!(rule["total"], 2);
    assert_eq!(rule["violations"].as_array().unwrap().len(), 1);
    assert_eq!(
        answer["notice"]["code"],
        ErrorCode::ResponseLimitExceeded.id()
    );
}

#[test]
fn without_the_right_to_read_nothing_is_answered() {
    let nobody = Principal::limited(PermissionSet::none());
    let failure = tools().check_rules(&nobody, &args(None)).unwrap_err();
    assert_eq!(failure.code(), ErrorCode::PermissionDenied);
}
