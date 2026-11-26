//! The level "namespaces" of the dependency map: one namespace in the middle, the namespaces that
//! use it in the row above, the namespaces it uses in the row below. It is built like the type
//! focus (`graph_types`); the name of a namespace is split at its dots into the label of the box
//! and a breadcrumb of the namespaces in front of it.

use std::collections::BTreeSet;

use codeskimr_application::config::Graph;
use codeskimr_application::error::CodedError;
use codeskimr_domain::{Layer, NamespaceGraph, NodePlacement, NodeSize, QualifiedName, SnapshotId};

use super::consts::{NAMESPACE_SIDE_MAX, TYPE_ROW_FOCUS, TYPE_ROW_USED, TYPE_ROW_USERS};
use super::graph::{BoxOut, FocusOut, GraphOut, GraphPorts, LinkOut, layer_index};
use super::graph_types::{breadcrumb, layout_rows, short_name};
use super::graph_wrap::centre;

/// A namespace that is drawn: its position in the graph and its row.
struct Drawn {
    at: usize,
    row: u32,
}

/// The references a namespace has with all others, the sum of its edges.
fn degree(graph: &NamespaceGraph, at: usize) -> u32 {
    graph
        .edges
        .iter()
        .filter(|edge| edge.from == at || edge.to == at)
        .map(|edge| edge.count)
        .sum()
}

/// Whether two names stand for the same place at different depths: one is the other or a parent
/// of it. A map with another depth names the same namespace with fewer or more segments.
fn related(one: &QualifiedName, other: &str) -> bool {
    one.is_inside(other) || QualifiedName::new(other).is_ok_and(|name| name.is_inside(one.as_str()))
}

/// The position of the node with the most references among `candidates`.
fn busiest(graph: &NamespaceGraph, candidates: impl Iterator<Item = usize>) -> Option<usize> {
    candidates.max_by_key(|&at| {
        let types = graph.namespaces.get(at).map_or(0, |node| node.types);
        (degree(graph, at), types)
    })
}

/// The namespace in the middle: the one asked for, else the busiest one that is the same place at
/// another depth (a parent or a child of the name), else the one with the most references.
fn middle(graph: &NamespaceGraph, asked: Option<&str>) -> Option<usize> {
    let all = || 0..graph.namespaces.len();
    let named = asked.and_then(|name| {
        let exact = all().find(|&at| name_of(graph, at) == name);
        exact.or_else(|| {
            busiest(
                graph,
                all().filter(|&at| {
                    graph
                        .namespaces
                        .get(at)
                        .is_some_and(|node| related(&node.name, name))
                }),
            )
        })
    });
    named.or_else(|| busiest(graph, all()))
}

/// The neighbours of `at` on one side with the references, the most referenced first.
fn side(graph: &NamespaceGraph, at: usize, users: bool) -> Vec<(usize, u32)> {
    let mut found: Vec<(usize, u32)> = graph
        .edges
        .iter()
        .filter_map(|edge| match users {
            true if edge.to == at => Some((edge.from, edge.count)),
            false if edge.from == at => Some((edge.to, edge.count)),
            _ => None,
        })
        .collect();
    found.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    found
}

pub(super) fn name_of(graph: &NamespaceGraph, at: usize) -> String {
    graph.name_at(at)
}

/// The drawn namespaces (the middle first, then the users, then the used ones that are not users
/// as well) and how many neighbours each side leaves out.
fn drawn(graph: &NamespaceGraph, at: usize) -> (Vec<Drawn>, usize, usize) {
    let (users, used) = (side(graph, at, true), side(graph, at, false));
    let mut seen = BTreeSet::from([at]);
    let mut all = vec![Drawn {
        at,
        row: TYPE_ROW_FOCUS,
    }];
    for (list, row) in [(&users, TYPE_ROW_USERS), (&used, TYPE_ROW_USED)] {
        for &(other, _) in list.iter().take(NAMESPACE_SIDE_MAX) {
            if seen.insert(other) {
                all.push(Drawn { at: other, row });
            }
        }
    }
    let hidden = |list: &[(usize, u32)]| list.len().saturating_sub(NAMESPACE_SIDE_MAX);
    (all, hidden(&users), hidden(&used))
}

/// The links between the middle and the drawn neighbours, both directions.
fn links(graph: &NamespaceGraph, drawn: &[Drawn]) -> Vec<LinkOut> {
    let shown = |at: usize| drawn.iter().any(|one| one.at == at);
    let middle = drawn.first().map_or(0, |one| one.at);
    graph
        .edges
        .iter()
        .filter(|edge| {
            (edge.from == middle || edge.to == middle) && shown(edge.from) && shown(edge.to)
        })
        .map(|edge| LinkOut {
            from: name_of(graph, edge.from),
            to: name_of(graph, edge.to),
            count: edge.count,
            violates_layering: false,
            in_cycle: false,
        })
        .collect()
}

/// The box of the namespace at `at`: the last name segment as the label, the rest as the
/// breadcrumb, and the layer that owns it.
pub(super) fn namespace_box(
    graph: &NamespaceGraph,
    layers: &[Layer],
    at: usize,
    spot: &NodePlacement,
    size: NodeSize,
) -> BoxOut {
    let name = name_of(graph, at);
    let holder = layer_index(layers, &name)
        .and_then(|found| layers.get(found))
        .map(|layer| layer.name.as_str().to_owned());
    let types = graph.namespaces.get(at).map(|node| node.types);
    BoxOut {
        label: short_name(&name),
        context: breadcrumb(&name),
        ..BoxOut::placed(&name, spot, size, holder, types)
    }
}

fn place(
    look: &Graph,
    graph: &NamespaceGraph,
    layers: &[Layer],
    drawn: &[Drawn],
    links: &[LinkOut],
) -> Result<Vec<BoxOut>, CodedError> {
    let rows: Vec<u32> = drawn.iter().map(|one| one.row).collect();
    let owned: Vec<String> = drawn.iter().map(|one| name_of(graph, one.at)).collect();
    let names: Vec<&str> = owned.iter().map(String::as_str).collect();
    let layout = layout_rows(look, &rows, &names, links)?;
    let size = look.node_size();
    Ok(drawn
        .iter()
        .zip(&layout.placements)
        .map(|(one, spot)| namespace_box(graph, layers, one.at, spot, size))
        .collect())
}

/// The map around one namespace. `choice` is the depth that merges namespaces into their first
/// segments (0 keeps all) and the name of the namespace in the middle.
pub(super) fn namespaces_out(
    ports: &GraphPorts,
    layers: &[Layer],
    snapshot: SnapshotId,
    choice: (u32, Option<&str>),
) -> Result<GraphOut, CodedError> {
    let depth = usize::try_from(choice.0).unwrap_or_default();
    let graph = ports.index.namespace_graph(snapshot)?.collapsed(depth);
    let Some(at) = middle(&graph, choice.1) else {
        return Ok(GraphOut::default());
    };
    let (drawn, users_more, used_more) = drawn(&graph, at);
    let links = links(&graph, &drawn);
    let mut boxes = place(&ports.look, &graph, layers, &drawn, &links)?;
    let size = centre(&mut boxes, &ports.look);
    Ok(GraphOut {
        width: size.width,
        height: size.height,
        focus: Some(FocusOut {
            symbol: None,
            name: name_of(&graph, at),
            users_more: u32::try_from(users_more).unwrap_or(u32::MAX),
            used_more: u32::try_from(used_more).unwrap_or(u32::MAX),
        }),
        boxes,
        links,
        ..GraphOut::default()
    })
}

#[cfg(test)]
pub(super) mod tests {
    use codeskimr_domain::{NamespaceEdge, NamespaceNode, QualifiedName};

    use super::*;

    /// A graph of namespaces with one type each.
    pub(in crate::ui_commands) fn graph(
        names: &[&str],
        edges: &[(usize, usize, u32)],
    ) -> NamespaceGraph {
        NamespaceGraph {
            namespaces: names
                .iter()
                .map(|name| NamespaceNode {
                    name: QualifiedName::new(*name).unwrap(),
                    types: 1,
                })
                .collect(),
            edges: edges
                .iter()
                .map(|&(from, to, count)| NamespaceEdge { from, to, count })
                .collect(),
        }
    }

    #[test]
    fn the_namespace_with_the_most_references_is_the_default_middle() {
        let all = graph(&["A", "B", "C"], &[(0, 1, 1), (2, 1, 5)]);
        assert_eq!(middle(&all, None), Some(1));
        assert_eq!(middle(&all, Some("C")), Some(2));
        assert_eq!(middle(&all, Some("Unknown")), Some(1));
        assert_eq!(middle(&graph(&[], &[]), None), None);
    }

    #[test]
    fn the_asked_namespace_is_found_again_at_another_depth() {
        let merged = graph(&["Shop", "Core"], &[(0, 1, 1)]);
        assert_eq!(middle(&merged, Some("Shop.Ui.Cart")), Some(0));
        let deep = graph(&["Shop.Ui", "Shop.App", "Core"], &[(0, 2, 1), (1, 2, 3)]);
        assert_eq!(middle(&deep, Some("Shop")), Some(1));
        assert_eq!(middle(&deep, Some("Sho")), Some(2));
    }

    #[test]
    fn users_come_above_and_the_used_below_and_a_namespace_on_both_sides_stays_above() {
        let all = graph(
            &["A", "B", "C", "D"],
            &[(0, 1, 3), (1, 2, 2), (1, 0, 1), (1, 3, 4)],
        );
        let (shown, users_more, used_more) = drawn(&all, 1);
        let rows: Vec<(usize, u32)> = shown.iter().map(|one| (one.at, one.row)).collect();
        assert_eq!(
            rows,
            [
                (1, TYPE_ROW_FOCUS),
                (0, TYPE_ROW_USERS),
                (3, TYPE_ROW_USED),
                (2, TYPE_ROW_USED)
            ]
        );
        assert_eq!((users_more, used_more), (0, 0));
        assert_eq!(links(&all, &shown).len(), 4);
    }

    #[test]
    fn a_long_side_is_cut_and_counted() {
        let names: Vec<String> = (0..40).map(|n| format!("N{n}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let edges: Vec<(usize, usize, u32)> = (1..40).map(|n| (n, 0, 1)).collect();
        let (shown, users_more, _) = drawn(&graph(&refs, &edges), 0);
        assert_eq!(shown.len(), 1 + NAMESPACE_SIDE_MAX);
        assert_eq!(users_more, 39 - NAMESPACE_SIDE_MAX);
    }
}
