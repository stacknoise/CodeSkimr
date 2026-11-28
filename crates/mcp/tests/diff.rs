//! `impact_of_diff` against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{PermissionSet, Principal};
use codeskimr_mcp::DiffArgs;
use common::tools;

#[cfg(test)]
fn args(base: Option<u64>, target: Option<u64>) -> DiffArgs {
    DiffArgs {
        project: None,
        base,
        target,
        metrics: None,
        metrics_level: None,
        include_tests: None,
        name: None,
        cause: None,
        kind: None,
        public_only: None,
        only_open: None,
    }
}

#[test]
fn the_answer_names_both_scans_and_the_limit_of_what_it_sees() {
    let answer = tools()
        .impact_of_diff(&Principal::local_owner(), &args(None, None))
        .unwrap();
    assert_eq!(answer["schema"], 1);
    assert_eq!(answer["base"]["snapshot"], 1);
    assert_eq!(answer["target"]["snapshot"], 2);
    assert_eq!(answer["direct_usages_only"], true);
    assert!(
        answer["note"]
            .as_str()
            .unwrap()
            .contains("Only direct usages are known"),
        "the answer says what the comparison does not see"
    );
}

#[test]
fn the_checklist_lists_the_usages_that_were_not_touched() {
    let answer = tools()
        .impact_of_diff(&Principal::local_owner(), &args(Some(5), Some(6)))
        .unwrap();
    assert_eq!(answer["base"]["snapshot"], 5);
    assert_eq!(answer["target"]["snapshot"], 6);
    let impact = &answer["impact"]["items"][0];
    assert_eq!(impact["cause"], "removed");
    assert_eq!(impact["symbol"]["qualified_name"], "Shop.Checkout.Cart.Pay");
    assert_eq!(impact["usages"]["total"], 2);
    assert_eq!(impact["usages"]["is_cut"], true);
    assert_eq!(impact["usages"]["items"][0]["status"], "not_touched");
    assert_eq!(impact["usages"]["items"][0]["confidence"], "certain");
    assert_eq!(impact["usages"]["items"][0]["line"], 11);
    assert_eq!(impact["not_touched"], 1);
    assert_eq!(answer["summary"]["changed_symbols"], 1);
    assert_eq!(answer["summary"]["public_symbols"], 1);
    assert_eq!(answer["summary"]["usages_not_touched"], 1);
    assert_eq!(answer["is_cut"], true);
}

#[test]
fn a_base_that_is_not_older_is_refused_with_its_code() {
    let failure = tools()
        .impact_of_diff(&Principal::local_owner(), &args(Some(3), Some(3)))
        .unwrap_err();
    assert_eq!(failure.code(), ErrorCode::ComparisonOrderInvalid);
}

#[test]
fn without_the_right_to_read_nothing_is_answered() {
    let nobody = Principal::limited(PermissionSet::none());
    let failure = tools()
        .impact_of_diff(&nobody, &args(None, None))
        .unwrap_err();
    assert_eq!(failure.code(), ErrorCode::PermissionDenied);
}

#[test]
fn the_metric_changes_come_only_when_they_are_asked_for() {
    let plain = tools()
        .impact_of_diff(&Principal::local_owner(), &args(None, None))
        .unwrap();
    assert!(plain.get("metrics").is_none());
    let asked = DiffArgs {
        metrics: Some(true),
        ..args(Some(5), Some(6))
    };
    let answer = tools()
        .impact_of_diff(&Principal::local_owner(), &asked)
        .unwrap();
    let metrics = &answer["metrics"];
    assert_eq!(metrics["level"], "member");
    assert_eq!(metrics["target"]["snapshot"], 6);
    assert_eq!(
        metrics["worse"]["items"][0]["name"],
        "Shop.Checkout.Cart.Pay"
    );
    assert_eq!(metrics["worse"]["items"][0]["moves"][0]["metric"], "cc");
    assert_eq!(metrics["worse"]["items"][0]["crosses"], true);
    assert_eq!(metrics["better"]["total"], 0);
}

#[test]
fn an_unknown_level_of_the_metric_changes_is_refused() {
    let asked = DiffArgs {
        metrics: Some(true),
        metrics_level: Some("galaxy".to_owned()),
        ..args(None, None)
    };
    let error = tools()
        .impact_of_diff(&Principal::local_owner(), &asked)
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::RequestInvalid);
}

#[test]
fn a_filter_keeps_the_symbols_it_names_and_counts_only_those() {
    let kept = DiffArgs {
        name: Some("cart.PAY".to_owned()),
        cause: Some("removed".to_owned()),
        ..args(Some(5), Some(6))
    };
    let answer = tools()
        .impact_of_diff(&Principal::local_owner(), &kept)
        .unwrap();
    assert_eq!(answer["impact"]["total"], 1);
    let none = DiffArgs {
        name: Some("no such symbol".to_owned()),
        ..args(Some(5), Some(6))
    };
    let answer = tools()
        .impact_of_diff(&Principal::local_owner(), &none)
        .unwrap();
    assert_eq!(answer["impact"]["total"], 0);
}

#[test]
fn an_unknown_cause_or_kind_is_refused() {
    for asked in [
        DiffArgs {
            cause: Some("vanished".to_owned()),
            ..args(None, None)
        },
        DiffArgs {
            kind: Some("galaxy".to_owned()),
            ..args(None, None)
        },
    ] {
        let error = tools()
            .impact_of_diff(&Principal::local_owner(), &asked)
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::RequestInvalid);
    }
}
