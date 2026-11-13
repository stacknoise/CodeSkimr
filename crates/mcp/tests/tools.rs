//! The three tools against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_mcp::{ArchitectureArgs, McpError, SearchArgs, SymbolArgs};
use common::{FakeIndex, NOW, SCANNED_AT, tools, tools_with};
use serde_json::Value;

#[cfg(test)]
fn search(query: &str, kind: Option<&str>, limit: Option<u32>) -> SearchArgs {
    SearchArgs {
        query: query.to_owned(),
        kind: kind.map(str::to_owned),
        project: None,
        limit,
    }
}

#[cfg(test)]
fn symbol(text: &str, project: Option<u64>) -> SymbolArgs {
    SymbolArgs {
        symbol: text.to_owned(),
        project,
    }
}

#[cfg(test)]
fn count(value: &Value, field: &str) -> usize {
    value[field].as_array().unwrap().len()
}

#[test]
fn search_uses_the_default_limit_and_reports_the_snapshot_age() {
    let answer = tools()
        .search_symbols(&search("order", None, None))
        .unwrap();
    assert_eq!(count(&answer, "hits"), 2);
    assert_eq!(answer["snapshot"]["id"], 10);
    assert_eq!(answer["snapshot"]["age_seconds"], NOW - SCANNED_AT);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["hits"][0]["qualified_name"], "App.Order");
    assert_eq!(answer["hits"][0]["kind"], "class");
}

#[test]
fn search_says_so_when_more_hits_exist_than_the_limit() {
    let answer = tools()
        .search_symbols(&search("order", None, Some(1)))
        .unwrap();
    assert_eq!(count(&answer, "hits"), 1);
    assert_eq!(answer["notice"]["code"], "MCP-501");
}

#[test]
fn get_symbol_takes_only_digits_as_an_id() {
    let error = tools().get_symbol(&symbol("+1", None)).unwrap_err();
    assert_eq!(error.report().code, "MCP-202");
}

#[test]
fn search_clamps_the_limit_and_says_so() {
    let answer = tools()
        .search_symbols(&search("order", None, Some(50)))
        .unwrap();
    assert_eq!(count(&answer, "hits"), 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["notice"]["params"]["limit"], "3");
}

#[test]
fn search_filters_by_kind_and_still_fills_the_limit() {
    let answer = tools()
        .search_symbols(&search("order", Some("class"), Some(2)))
        .unwrap();
    let kinds: Vec<_> = answer["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["class", "class"]);
    assert!(answer["notice"].is_null());
}

#[test]
fn search_rejects_an_empty_query_and_an_unknown_kind() {
    for args in [
        search("  ", None, None),
        search("order", Some("nonsense"), None),
    ] {
        let error = tools().search_symbols(&args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
    }
}

#[test]
fn get_symbol_by_qualified_name_shows_layer_metrics_and_one_based_lines() {
    let answer = tools()
        .get_symbol(&symbol("App.Domain.Order", None))
        .unwrap();
    assert_eq!(answer["layer"], "Domain");
    assert_eq!(answer["first_line"], 5);
    assert_eq!(answer["last_line"], 10);
    assert_eq!(answer["metrics"]["loc"], 40.0);
    assert_eq!(answer["signature"], "Order()");
    assert_eq!(answer["visibility"], "public");
}

#[test]
fn get_symbol_by_id_cuts_members_at_the_limit() {
    let answer = tools().get_symbol(&symbol("1", None)).unwrap();
    assert_eq!(answer["member_count"], 4);
    assert_eq!(count(&answer, "members"), 3);
    assert_eq!(answer["notice"]["code"], "MCP-501");
    assert_eq!(answer["notice"]["params"]["total"], "4");
}

#[test]
fn get_symbol_reports_unknown_symbols_and_projects() {
    let missing = tools().get_symbol(&symbol("App.Nope", None)).unwrap_err();
    assert_eq!(missing.report().code, "MCP-202");
    assert_eq!(missing.report().params["symbol"], "App.Nope");
    let unknown = tools().get_symbol(&symbol("1", Some(9))).unwrap_err();
    assert_eq!(unknown.report().code, "MCP-201");
    assert_eq!(unknown.report().params["project"], "9");
}

#[test]
fn an_unscanned_project_is_reported_as_such() {
    let tools = tools_with(FakeIndex {
        scanned: false,
        fail: false,
    });
    let error = tools
        .get_architecture(&ArchitectureArgs { project: None })
        .unwrap_err();
    assert_eq!(error.report().code, "MCP-203");
}

#[test]
fn a_failing_port_keeps_its_own_code() {
    let tools = tools_with(FakeIndex {
        scanned: true,
        fail: true,
    });
    let error = tools
        .search_symbols(&search("order", None, None))
        .unwrap_err();
    assert!(matches!(error, McpError::Port(_)));
    assert_eq!(error.report().code, ErrorCode::DatabaseFailed.id());
}

#[test]
fn get_architecture_lists_layers_and_dependencies() {
    let answer = tools()
        .get_architecture(&ArchitectureArgs { project: Some(1) })
        .unwrap();
    assert_eq!(answer["project"], "Shop");
    assert_eq!(count(&answer, "layers"), 2);
    assert_eq!(answer["layers"][0]["namespaces"][0], "App.Domain");
    assert_eq!(answer["dependencies"][0]["from"], "Application");
    assert_eq!(answer["dependencies"][0]["count"], 7);
    assert_eq!(answer["dependencies"][0]["violates_layering"], false);
}
