//! `find_usages` and `impact_of_change` against fake ports.

#[cfg(test)]
mod common;

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Direction, Principal};
use codeskimr_mcp::{ImpactArgs, UsagesArgs};
use common::{FakeIndex, FakeUsages, tools, tools_using};
use serde_json::Value;

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn usages(symbol: &str) -> UsagesArgs {
    UsagesArgs {
        symbol: symbol.to_owned(),
        direction: None,
        depth: None,
        include_tests: None,
        project: None,
        limit: None,
    }
}

#[cfg(test)]
fn impact(symbol: &str, limit: Option<u32>) -> ImpactArgs {
    ImpactArgs {
        symbol: symbol.to_owned(),
        project: None,
        limit,
    }
}

#[cfg(test)]
fn list(value: &Value, field: &str) -> Vec<Value> {
    value[field].as_array().unwrap().clone()
}

#[cfg(test)]
fn scanned() -> FakeIndex {
    FakeIndex {
        scanned: true,
        fail: false,
    }
}

#[test]
fn usages_show_kind_confidence_and_one_based_location() {
    let answer = tools().find_usages(&owner(), &usages("1")).unwrap();
    let first = &answer["usages"][0];
    assert_eq!(first["kind"], "call");
    assert_eq!(first["confidence"], "certain");
    assert_eq!(first["path"], "src/app/service.cs");
    assert_eq!(first["line"], 12);
    assert_eq!(first["column"], 8);
    assert_eq!(
        first["from"]["qualified_name"],
        "App.Application.Service.Run"
    );
    assert_eq!(first["to"]["kind"], "class");
    assert_eq!(answer["symbol"]["qualified_name"], "App.Domain.Order");
    assert_eq!(answer["snapshot"]["id"], 10);
}

#[test]
fn a_probable_usage_is_marked_as_such() {
    let answer = tools().find_usages(&owner(), &usages("1")).unwrap();
    assert_eq!(answer["usages"][1]["confidence"], "probable");
}

#[test]
fn usages_default_to_incoming_depth_one_without_tests() {
    let used = Arc::new(FakeUsages::default());
    let answer = tools_using(scanned(), used.clone())
        .find_usages(&owner(), &usages("App.Domain.Order"))
        .unwrap();
    let (query, limit) = used.asked.lock().unwrap()[0];
    assert_eq!(query.direction, Direction::Incoming);
    assert_eq!((query.depth, query.include_tests, limit), (1, false, 2));
    assert_eq!(answer["direction"], "incoming");
    assert_eq!(answer["depth"], 1);
    assert_eq!(answer["total"], 3);
}

#[test]
fn usages_pass_direction_depth_and_tests_on() {
    let used = Arc::new(FakeUsages::default());
    let args = UsagesArgs {
        direction: Some("outgoing".to_owned()),
        depth: Some(2),
        include_tests: Some(true),
        ..usages("1")
    };
    let answer = tools_using(scanned(), used.clone())
        .find_usages(&owner(), &args)
        .unwrap();
    let (query, _) = used.asked.lock().unwrap()[0];
    assert_eq!(query.direction, Direction::Outgoing);
    assert_eq!((query.depth, query.include_tests), (2, true));
    assert_eq!(answer["direction"], "outgoing");
    assert_eq!(answer["include_tests"], true);
    assert_eq!(list(&answer, "usages"), Vec::<Value>::new());
}

#[test]
fn usages_cut_at_the_limit_and_say_so() {
    let answer = tools().find_usages(&owner(), &usages("1")).unwrap();
    assert_eq!(list(&answer, "usages").len(), 2);
    assert_eq!(answer["total"], 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["notice"]["params"]["total"], "3");
}

#[test]
fn usages_hold_depth_and_limit_to_the_maximum_and_say_so() {
    let used = Arc::new(FakeUsages::default());
    let args = UsagesArgs {
        depth: Some(50),
        limit: Some(50),
        ..usages("1")
    };
    let answer = tools_using(scanned(), used.clone())
        .find_usages(&owner(), &args)
        .unwrap();
    let (query, limit) = used.asked.lock().unwrap()[0];
    assert_eq!((query.depth, limit), (3, 3));
    assert_eq!(answer["depth"], 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["notice"]["params"]["depth"], "3");
}

#[test]
fn a_complete_answer_has_no_notice() {
    let args = UsagesArgs {
        limit: Some(3),
        ..usages("1")
    };
    let answer = tools().find_usages(&owner(), &args).unwrap();
    assert!(answer["notice"].is_null());
}

#[test]
fn usages_reject_bad_arguments_with_the_argument_name() {
    let bad = [
        (
            UsagesArgs {
                depth: Some(0),
                ..usages("1")
            },
            "depth",
        ),
        (
            UsagesArgs {
                direction: Some("sideways".to_owned()),
                ..usages("1")
            },
            "direction",
        ),
        (usages(""), "symbol"),
    ];
    for (args, name) in bad {
        let error = tools().find_usages(&owner(), &args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
        assert_eq!(error.report().params["detail"], name);
    }
}

#[test]
fn usages_of_an_unknown_symbol_are_an_error_not_an_empty_list() {
    let error = tools().find_usages(&owner(), &usages("999")).unwrap_err();
    assert_eq!(error.code(), ErrorCode::SymbolNotFound);
    let project = UsagesArgs {
        project: Some(9),
        ..usages("1")
    };
    let error = tools().find_usages(&owner(), &project).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ProjectNotRegistered);
}

#[test]
fn impact_groups_by_layer_and_marks_public_interfaces() {
    let used = Arc::new(FakeUsages::default());
    let answer = tools_using(scanned(), used.clone())
        .impact_of_change(&owner(), &impact("1", Some(3)))
        .unwrap();
    assert_eq!(used.impact_asked.lock().unwrap()[0], (3, 3));
    let symbols = list(&answer, "symbols");
    assert_eq!(symbols[0]["layer"], "Application");
    assert_eq!(symbols[0]["public_interface"], true);
    assert_eq!(symbols[1]["public_interface"], false);
    assert_eq!(symbols[2]["layer"], "Domain");
    assert_eq!(
        answer["layers"][0],
        serde_json::json!({"layer": "Application", "count": 2})
    );
    assert_eq!(
        answer["layers"][1],
        serde_json::json!({"layer": "Domain", "count": 1})
    );
    assert_eq!(answer["public_interfaces"], 2);
    assert_eq!(answer["total"], 4);
    assert_eq!(answer["depth"], 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
}

#[test]
fn impact_of_an_unknown_symbol_is_an_error() {
    let error = tools()
        .impact_of_change(&owner(), &impact("999", None))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SymbolNotFound);
    let error = tools()
        .impact_of_change(&owner(), &impact("", None))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
}

#[test]
fn an_unscanned_project_has_no_usages() {
    let index = FakeIndex {
        scanned: false,
        fail: false,
    };
    let error = tools_using(index, Arc::new(FakeUsages::default()))
        .find_usages(&owner(), &usages("1"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ProjectNotScanned);
}
