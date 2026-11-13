//! The read models: usage counts, quick search, type summary and the layer graph.

#[cfg(test)]
mod common;

use codeskimr_domain::{MetricKind, SnapshotId, SymbolId, SymbolKind};
use common::{metric, reference, scanned, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn names(hits: &[codeskimr_storage::SearchHit]) -> Vec<&str> {
    hits.iter().map(|hit| hit.name.as_str()).collect()
}

#[cfg(test)]
fn searchable() -> codeskimr_storage::SqliteStore {
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.Order", None),
        symbol(22, 11, SymbolKind::Class, "App.OrderService", None),
        symbol(23, 11, SymbolKind::Method, "App.Reorder", None),
        symbol(24, 11, SymbolKind::Method, "App.Other_x", None),
        symbol(25, 11, SymbolKind::Parameter, "App.order", None),
    ];
    scanned(&[], symbols, vec![], vec![])
}

#[test]
fn search_ranks_exact_then_prefix_then_substring_then_fuzzy() {
    let hits = searchable().search(SNAPSHOT, "order", 10).unwrap();
    assert_eq!(names(&hits), ["Order", "OrderService", "Reorder"]);
}

#[test]
fn search_matches_characters_in_order_and_ignores_case() {
    let hits = searchable().search(SNAPSHOT, "OSV", 10).unwrap();
    assert_eq!(names(&hits), ["OrderService"]);
}

#[test]
fn search_treats_like_wildcards_as_text_and_skips_parameters() {
    let store = searchable();
    assert_eq!(
        names(&store.search(SNAPSHOT, "_x", 10).unwrap()),
        ["Other_x"]
    );
    assert_eq!(store.search(SNAPSHOT, "%", 10).unwrap(), Vec::new());
    assert_eq!(store.search(SNAPSHOT, "", 10).unwrap(), Vec::new());
}

#[test]
fn search_respects_the_limit_and_returns_where_the_symbol_lives() {
    let hits = searchable().search(SNAPSHOT, "order", 1).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path.as_str(), "src/order.cs");
    assert_eq!(hits[0].kind, SymbolKind::Class);
    assert_eq!(hits[0].qualified_name.as_str(), "App.Order");
}

#[test]
fn usage_counts_count_references_per_target() {
    let symbols = vec![
        symbol(21, 11, SymbolKind::Method, "App.A.Run", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Stop", None),
    ];
    let references = vec![
        reference(31, 11, 21, 22),
        reference(32, 11, 21, 22),
        reference(33, 11, 22, 22),
    ];
    let store = scanned(&[], symbols, references, vec![]);
    assert_eq!(store.usage_count(SNAPSHOT, SymbolId::new(22)).unwrap(), 3);
    assert_eq!(store.usage_count(SNAPSHOT, SymbolId::new(21)).unwrap(), 0);
}

#[test]
fn type_summary_has_members_and_metrics_for_types_only() {
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.A", None),
        symbol(22, 11, SymbolKind::Method, "App.A.Run", Some(21)),
        symbol(23, 11, SymbolKind::Method, "App.A.Stop", Some(21)),
    ];
    let metrics = vec![
        metric(21, MetricKind::Loc, 40.0),
        metric(21, MetricKind::MaintainabilityIndex, 61.5),
        metric(22, MetricKind::CyclomaticComplexity, 4.0),
        metric(23, MetricKind::CyclomaticComplexity, 9.0),
    ];
    let store = scanned(&[], symbols, vec![], metrics);
    let summary = store
        .type_summary(SNAPSHOT, SymbolId::new(21))
        .unwrap()
        .unwrap();
    assert_eq!(summary.members, 2);
    assert_eq!(summary.loc, Some(40.0));
    assert_eq!(summary.max_complexity, Some(9.0));
    assert_eq!(summary.maintainability, Some(61.5));
    assert!(
        store
            .type_summary(SNAPSHOT, SymbolId::new(22))
            .unwrap()
            .is_none()
    );
}

#[cfg(test)]
fn layered() -> codeskimr_storage::SqliteStore {
    let layers = [
        ("Api", &["App.Api"][..]),
        ("Core", &["App.Core"]),
        ("Data", &["App.Data"]),
    ];
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.Api.Controller", None),
        symbol(22, 11, SymbolKind::Class, "App.Core.Service", None),
        symbol(23, 11, SymbolKind::Class, "App.Data.Repo", None),
        symbol(24, 11, SymbolKind::Class, "App.Core.Helper", None),
        symbol(25, 11, SymbolKind::Class, "App.ApiExtras.Other", None),
    ];
    let references = vec![
        reference(31, 11, 21, 22),
        reference(32, 11, 21, 22),
        reference(33, 11, 22, 23),
        reference(34, 11, 23, 22),
        reference(35, 11, 22, 24),
        reference(36, 11, 25, 22),
    ];
    scanned(&layers, symbols, references, vec![])
}

#[test]
fn layer_graph_aggregates_references_between_layers() {
    let edges = layered().layer_edges(SNAPSHOT).unwrap();
    let shown: Vec<_> = edges
        .iter()
        .map(|edge| {
            (
                edge.from.as_str(),
                edge.to.as_str(),
                edge.count,
                edge.in_cycle,
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            ("Api", "Core", 2, false),
            ("Core", "Data", 1, true),
            ("Data", "Core", 1, true)
        ],
        "same-layer references and names outside every layer are not edges"
    );
}

#[test]
fn project_without_layers_has_no_layer_graph() {
    let store = searchable();
    assert_eq!(store.layer_edges(SNAPSHOT).unwrap(), Vec::new());
}
