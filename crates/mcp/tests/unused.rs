//! `find_unused` against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, PermissionSet, Principal};
use codeskimr_mcp::UnusedArgs;
use common::{BROKEN_LIST, tools};

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn args() -> UnusedArgs {
    UnusedArgs {
        kind: None,
        language: None,
        assessment: None,
        include_tests: None,
        project: None,
        limit: None,
    }
}

#[test]
fn symbols_without_a_usage_come_with_assessment_reason_and_counts() {
    let answer = tools().find_unused(&owner(), &args()).unwrap();
    assert_eq!(answer["total"], 2);
    assert_eq!(answer["counts"]["likely"], 1);
    assert_eq!(answer["counts"]["public"], 1);
    assert_eq!(answer["counts"]["tests_hidden"], 1);
    assert_eq!(answer["unused"][0]["name"], "Orphan");
    assert_eq!(answer["unused"][0]["assessment"], "likely");
    assert_eq!(answer["unused"][0]["reason"], "private");
    assert_eq!(answer["unused"][1]["reason"], "public");
    assert_eq!(answer["snapshot"]["id"], 10);
}

#[test]
fn a_symbol_of_the_exception_list_is_listed_apart_with_its_entry() {
    let answer = tools().find_unused(&owner(), &args()).unwrap();
    assert_eq!(answer["counts"]["left_out"], 1);
    let left = &answer["left_out"][0];
    assert_eq!(left["name"], "Migrate");
    assert_eq!(left["rule"]["reason"], "called by the migration tool");
    assert_eq!(left["rule"]["symbols"][0], "*.Migrate");
    let names: Vec<_> = answer["unused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"Migrate"));
}

#[test]
fn the_assessment_cuts_the_list_but_not_the_counts() {
    let wanted = UnusedArgs {
        assessment: Some("public".to_owned()),
        ..args()
    };
    let answer = tools().find_unused(&owner(), &wanted).unwrap();
    assert_eq!(answer["total"], 1);
    assert_eq!(answer["unused"][0]["name"], "Spare");
    assert_eq!(answer["counts"]["likely"], 1);
    // The filter does not touch the symbols the exception list took out.
    assert_eq!(answer["left_out"][0]["name"], "Migrate");
}

#[test]
fn test_code_is_only_there_if_the_caller_asks_for_it() {
    let with = UnusedArgs {
        include_tests: Some(true),
        ..args()
    };
    let answer = tools().find_unused(&owner(), &with).unwrap();
    assert_eq!(answer["total"], 3);
    assert_eq!(answer["counts"]["tests_hidden"], 0);
    assert_eq!(answer["unused"].as_array().unwrap().len(), 2);
    assert_eq!(answer["notice"]["code"], "MCP-501");
}

#[test]
fn an_unknown_label_is_an_invalid_argument_and_a_caller_without_read_is_refused() {
    for wrong in [
        UnusedArgs {
            kind: Some("everything".to_owned()),
            ..args()
        },
        UnusedArgs {
            assessment: Some("dead".to_owned()),
            ..args()
        },
    ] {
        let error = tools().find_unused(&owner(), &wrong).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
    }
    let nobody = Principal::limited(PermissionSet::none());
    let error = tools().find_unused(&nobody, &args()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    assert!(tools().find_unused(&reader, &args()).is_ok());
}

#[test]
fn a_broken_exception_list_fails_the_tool_with_its_code() {
    let asked = UnusedArgs {
        language: Some(BROKEN_LIST.to_owned()),
        ..args()
    };
    let error = tools().find_unused(&owner(), &asked).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ConfigInvalidValue);
}
