//! `get_dependencies` and `rescan` against fake ports.

#[cfg(test)]
mod common;

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Direction, Permission, PermissionSet, Principal};
use codeskimr_mcp::{DependenciesArgs, RescanArgs};
use common::{FakeIndex, FakeScanner, FakeUsages, PROJECT, tools, tools_scanning, tools_using};
use serde_json::Value;

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn of_symbol(symbol: &str) -> DependenciesArgs {
    DependenciesArgs {
        symbol: Some(symbol.to_owned()),
        layer: None,
        direction: None,
        include_tests: None,
        project: None,
        limit: None,
    }
}

#[cfg(test)]
fn of_layer(layer: &str) -> DependenciesArgs {
    DependenciesArgs {
        symbol: None,
        layer: Some(layer.to_owned()),
        ..of_symbol("")
    }
}

#[cfg(test)]
fn list(value: &Value, field: &str) -> Vec<Value> {
    value[field].as_array().unwrap().clone()
}

#[test]
fn dependencies_go_outgoing_by_default_and_mark_what_breaks_the_layering() {
    let mut args = of_symbol("1");
    args.limit = Some(3);
    let answer = tools().get_dependencies(&owner(), &args).unwrap();
    assert_eq!(answer["direction"], "outgoing");
    assert_eq!(answer["target"]["kind"], "symbol");
    assert_eq!(answer["target"]["name"], "App.Domain.Order");
    assert_eq!(answer["snapshot"]["id"], 10);
    assert_eq!(answer["total"], 3);
    assert_eq!(answer["violations"], 1);
    let found = list(&answer, "dependencies");
    assert_eq!(found[0]["qualified_name"], "App.Domain.Invoice");
    assert_eq!(found[0]["violates_layering"], false);
    assert_eq!(found[1]["layer"], "Application");
    assert_eq!(found[1]["violates_layering"], true);
    assert_eq!(found[2]["layer"], Value::Null);
}

#[test]
fn the_layers_add_up_the_other_layers_only() {
    let answer = tools().get_dependencies(&owner(), &of_symbol("1")).unwrap();
    let layers = list(&answer, "layers");
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0]["layer"], "Application");
    assert_eq!(layers[0]["count"], 2);
    assert_eq!(layers[0]["violates_layering"], true);
    assert_eq!(layers[0]["in_cycle"], true);
}

#[test]
fn incoming_dependencies_read_the_layer_edge_the_other_way() {
    let mut args = of_symbol("App.Domain.Order");
    args.direction = Some("incoming".to_owned());
    let answer = tools().get_dependencies(&owner(), &args).unwrap();
    assert_eq!(answer["direction"], "incoming");
    assert_eq!(answer["violations"], 0);
    assert_eq!(answer["layers"][0]["violates_layering"], false);
}

#[test]
fn the_question_reaches_the_index_with_the_symbol_and_the_test_flag() {
    let usages = Arc::new(FakeUsages::default());
    let tools = tools_using(
        FakeIndex {
            scanned: true,
            fail: false,
        },
        Arc::clone(&usages),
    );
    let mut args = of_symbol("1");
    args.include_tests = Some(true);
    tools.get_dependencies(&owner(), &args).unwrap();
    let asked = usages.dependencies_asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].symbol.get(), 1);
    assert_eq!(asked[0].direction, Direction::Outgoing);
    assert!(asked[0].include_tests);
}

#[test]
fn a_long_list_is_cut_with_a_notice_but_the_layers_stay_complete() {
    let mut args = of_symbol("1");
    args.limit = Some(1);
    let answer = tools().get_dependencies(&owner(), &args).unwrap();
    assert_eq!(list(&answer, "dependencies").len(), 1);
    assert_eq!(answer["total"], 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["notice"]["params"]["total"], "3");
    assert_eq!(list(&answer, "layers").len(), 1);
}

#[test]
fn a_layer_shows_its_recorded_links() {
    let answer = tools()
        .get_dependencies(&owner(), &of_layer("Domain"))
        .unwrap();
    assert_eq!(answer["target"]["kind"], "layer");
    assert_eq!(answer["violations"], 1);
    assert_eq!(answer["layers"][0]["layer"], "Application");
    assert_eq!(answer["layers"][0]["count"], 2);
    assert!(list(&answer, "dependencies").is_empty());
}

#[test]
fn an_unknown_layer_and_a_missing_or_double_target_are_invalid_arguments() {
    let mut both = of_symbol("1");
    both.layer = Some("Domain".to_owned());
    for args in [
        of_layer("Nowhere"),
        both,
        DependenciesArgs {
            symbol: None,
            ..of_symbol("")
        },
    ] {
        let error = tools().get_dependencies(&owner(), &args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
    }
    let mut bad = of_symbol("1");
    bad.direction = Some("sideways".to_owned());
    let error = tools().get_dependencies(&owner(), &bad).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
}

#[test]
fn an_unknown_symbol_and_an_unknown_project_have_their_own_codes() {
    let error = tools()
        .get_dependencies(&owner(), &of_symbol("App.Missing"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::SymbolNotFound);
    let mut elsewhere = of_symbol("1");
    elsewhere.project = Some(99);
    let error = tools().get_dependencies(&owner(), &elsewhere).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ProjectNotRegistered);
}

#[test]
fn a_caller_without_read_gets_the_same_denial_for_valid_and_invalid_arguments() {
    let nobody = Principal::limited(PermissionSet::none());
    for args in [of_symbol("1"), of_layer("Nowhere"), of_symbol("")] {
        let error = tools().get_dependencies(&nobody, &args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::PermissionDenied);
    }
}

#[test]
fn rescan_runs_the_scanner_and_names_snapshot_and_duration() {
    let scanner = Arc::new(FakeScanner::default());
    let answer = tools_scanning(Arc::clone(&scanner))
        .rescan(&owner(), &RescanArgs { project: None })
        .unwrap();
    assert_eq!(answer["snapshot"]["id"], 11);
    assert_eq!(answer["snapshot"]["age_seconds"], 5);
    assert_eq!(answer["mode"], "incremental");
    assert_eq!(answer["files"], 3);
    assert_eq!(answer["symbols"], 40);
    assert_eq!(answer["references"], 90);
    assert!(answer["duration_ms"].is_u64());
    let asked = scanner.asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].1, PROJECT);
}

#[test]
fn rescan_without_the_scan_permission_starts_nothing() {
    let scanner = Arc::new(FakeScanner::default());
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let error = tools_scanning(Arc::clone(&scanner))
        .rescan(&reader, &RescanArgs { project: None })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
    assert!(scanner.asked.lock().unwrap().is_empty());
}

#[test]
fn rescan_of_an_unknown_project_starts_nothing() {
    let scanner = Arc::new(FakeScanner::default());
    let error = tools_scanning(Arc::clone(&scanner))
        .rescan(&owner(), &RescanArgs { project: Some(99) })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ProjectNotRegistered);
    assert!(scanner.asked.lock().unwrap().is_empty());
}

#[test]
fn rescan_keeps_the_code_of_a_failed_scan() {
    let scanner = Arc::new(FakeScanner {
        fail: true,
        ..FakeScanner::default()
    });
    let error = tools_scanning(scanner)
        .rescan(&owner(), &RescanArgs { project: None })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnexpectedInternalError);
}
