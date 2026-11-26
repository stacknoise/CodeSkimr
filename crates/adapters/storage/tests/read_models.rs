//! The read models: usage counts, quick search, type summary and the layer graph.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{ProjectRepository, ReferencePlaces, ReferenceSites};
use codeskimr_domain::{
    Confidence, MetricKind, ProjectId, Reference, ReferenceKind, RelativePath, SnapshotId,
    SymbolId, SymbolKind,
};
use common::{metric, project, reference, scanned, scanned_project, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn names(hits: &[codeskimr_storage::SearchHit]) -> Vec<&str> {
    hits.iter().map(|hit| hit.name.as_str()).collect()
}

#[cfg(test)]
fn searchable() -> codeskimr_storage::ProjectDb {
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.Order", None),
        symbol(22, 11, SymbolKind::Class, "App.OrderService", None),
        symbol(23, 11, SymbolKind::Method, "App.Reorder", Some(22)),
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
fn a_hit_names_its_enclosing_symbol() {
    let hits = searchable().search(SNAPSHOT, "reorder", 10).unwrap();
    assert_eq!(hits[0].parent, Some(SymbolId::new(22)));
    let hits = searchable().search(SNAPSHOT, "orderservice", 10).unwrap();
    assert_eq!(hits[0].parent, None);
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
fn equal_names_are_told_apart_by_symbol_id_when_the_limit_cuts_them_off() {
    let symbols = (21..=25)
        .rev()
        .map(|id| symbol(id, 11, SymbolKind::Method, &format!("App.C{id}.get"), None))
        .collect();
    let store = scanned(&[], symbols, vec![], vec![]);
    let hits = store.search(SNAPSHOT, "get", 3).unwrap();
    let ids: Vec<u64> = hits.iter().map(|hit| hit.symbol.get()).collect();
    assert_eq!(ids, [21, 22, 23]);
}

fn from_a_test(reference: Reference) -> Reference {
    Reference {
        kind: ReferenceKind::Test,
        ..reference
    }
}

fn probable(reference: Reference) -> Reference {
    Reference {
        confidence: Confidence::Probable,
        ..reference
    }
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
        // References of kind `test` are not counted as usages (issue #430) ...
        from_a_test(reference(34, 11, 21, 22)),
    ];
    let store = scanned(&[], symbols, references, vec![]);
    assert_eq!(store.usage_count(SNAPSHOT, SymbolId::new(22)).unwrap(), 3);
    assert_eq!(store.usage_count(SNAPSHOT, SymbolId::new(21)).unwrap(), 0);
    // ... but the total of the snapshot is a count of what is stored.
    let latest = store.latest_snapshot(ProjectId::new(1)).unwrap().unwrap();
    assert_eq!(latest.stats.references, 4);
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
fn layered() -> codeskimr_storage::ProjectDb {
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
        // A reference of kind `test` is no edge of the graph (issue #430).
        from_a_test(reference(37, 11, 22, 21)),
        // A reference that only probably points there is no edge either (issue #464).
        probable(reference(38, 11, 21, 23)),
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
        "same-layer, outside-layer, test and probable references are no edges"
    );
}

#[test]
fn project_without_layers_has_no_layer_graph() {
    let store = searchable();
    assert_eq!(store.layer_edges(SNAPSHOT).unwrap(), Vec::new());
}

#[test]
fn an_edge_against_the_allowed_directions_violates_the_layering() {
    use codeskimr_domain::LayerName;
    let name = |text: &str| LayerName::new(text).unwrap();
    let mut project = project(
        1,
        "/work/app",
        &[("Api", &["App.Api"]), ("Core", &["App.Core"])],
    );
    project.layers[0].may_use = Some(Vec::new());
    project.layers[1].may_use = Some(vec![name("Api")]);
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.Api.Controller", None),
        symbol(22, 11, SymbolKind::Class, "App.Core.Service", None),
    ];
    let references = vec![reference(31, 11, 21, 22), reference(32, 11, 22, 21)];
    let store = scanned_project(&project, symbols, references, vec![]);
    let edges = store.layer_edges(SNAPSHOT).unwrap();
    let shown: Vec<_> = edges
        .iter()
        .map(|edge| (edge.from.as_str(), edge.to.as_str(), edge.violates_layering))
        .collect();
    assert_eq!(shown, [("Api", "Core", true), ("Core", "Api", false)]);
    let layers = store.project(ProjectId::new(1)).unwrap().layers;
    assert_eq!(
        layers[0].may_use,
        Some(Vec::new()),
        "an empty list stays a limit"
    );
    assert_eq!(layers[1].may_use, Some(vec![name("Api")]));
}

#[test]
fn a_layer_without_a_limit_never_violates() {
    let edges = layered().layer_edges(SNAPSHOT).unwrap();
    assert!(edges.iter().all(|edge| !edge.violates_layering));
}

#[test]
fn namespace_edges_leave_out_probable_and_test_references() {
    let symbols = vec![
        symbol(41, 11, SymbolKind::Namespace, "App.Api", None),
        symbol(42, 11, SymbolKind::Namespace, "App.Core", None),
        symbol(43, 11, SymbolKind::Namespace, "App.Data", None),
        symbol(44, 11, SymbolKind::Class, "App.Api.Controller", None),
        symbol(45, 11, SymbolKind::Class, "App.Core.Service", None),
        symbol(46, 11, SymbolKind::Class, "App.Data.Repo", None),
    ];
    let references = vec![
        reference(51, 11, 44, 45),
        probable(reference(52, 11, 44, 46)),
        from_a_test(reference(53, 11, 45, 44)),
    ];
    let graph = scanned(&[], symbols, references, vec![])
        .namespace_graph(SNAPSHOT)
        .unwrap();
    let named = |index: usize| graph.namespaces[index].name.as_str().to_owned();
    let edges: Vec<_> = graph
        .edges
        .iter()
        .map(|edge| (named(edge.from), named(edge.to), edge.count))
        .collect();
    assert_eq!(edges, [("App.Api".to_owned(), "App.Core".to_owned(), 1)]);
}

#[test]
fn the_certain_references_of_a_file_come_with_both_names() {
    let file = RelativePath::new("src/order.cs").unwrap();
    let other = RelativePath::new("src/none.cs").unwrap();
    let sites = layered()
        .references_from(SNAPSHOT, &[file.clone(), other])
        .unwrap();
    let shown: Vec<_> = sites
        .iter()
        .map(|site| (site.path.as_str(), site.from.as_str(), site.to.as_str()))
        .collect();
    assert_eq!(shown.len(), 6, "test and probable references are left out");
    assert!(shown.contains(&("src/order.cs", "App.Api.Controller", "App.Core.Service")));
    assert!(shown.iter().all(|(path, _, _)| *path == file.as_str()));
}

#[test]
fn every_certain_reference_of_the_snapshot_comes_with_its_own_place() {
    let mut found = Vec::new();
    layered()
        .for_each_reference(SNAPSHOT, &mut |place| found.push(place))
        .unwrap();
    assert!(found.iter().all(|place| !place.path.as_str().is_empty()));
    let shown: Vec<_> = found
        .iter()
        .map(|place| (place.from.as_str(), place.to.as_str()))
        .collect();
    assert!(shown.contains(&("App.Api.Controller", "App.Core.Service")));
    let in_file: Vec<_> = found
        .iter()
        .filter(|place| place.path.as_str() == "src/order.cs")
        .collect();
    assert_eq!(
        in_file.len(),
        6,
        "test and probable references are left out"
    );
}
