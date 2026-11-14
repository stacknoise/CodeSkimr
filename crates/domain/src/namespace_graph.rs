//! The dependencies between namespaces: every reference between two symbols that sit in different
//! namespaces counts for the pair of their namespaces (blueprint 15, screen 3, level
//! "namespaces"). A symbol sits in the nearest enclosing namespace or module, found by its
//! qualified name; a symbol outside every namespace belongs to none and is not counted.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::consts::{MODULE_NAME_SEPARATOR, NAME_SEPARATOR};
use crate::qualified_name::QualifiedName;
use crate::symbol::{SymbolId, SymbolKind};

/// One namespace of the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceNode {
    /// Qualified name of the namespace.
    pub name: QualifiedName,
    /// Number of types that sit directly in it.
    pub types: u32,
}

/// The references from one namespace to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceEdge {
    /// Position of the depending namespace in [`NamespaceGraph::namespaces`].
    pub from: usize,
    /// Position of the namespace it depends on.
    pub to: usize,
    /// Number of references behind the edge.
    pub count: u32,
}

/// The namespaces of a snapshot and the dependencies between them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceGraph {
    /// All namespaces, ordered by name.
    pub namespaces: Vec<NamespaceNode>,
    /// The dependencies, ordered by their ends.
    pub edges: Vec<NamespaceEdge>,
}

/// The first `depth` segments of `name`; the whole name if it has no more segments.
fn leading_segments(name: &str, depth: usize) -> &str {
    let mut cuts: Vec<usize> = name
        .match_indices(NAME_SEPARATOR)
        .chain(name.match_indices(MODULE_NAME_SEPARATOR))
        .map(|(position, _)| position)
        .collect();
    cuts.sort_unstable();
    depth
        .checked_sub(1)
        .and_then(|last| cuts.get(last))
        .and_then(|&cut| name.get(..cut))
        .unwrap_or(name)
}

impl NamespaceGraph {
    /// The graph with every namespace merged into its first `depth` segments (`App.Views.Board`
    /// and `App.Views.Menu` become `App.Views` at depth 2): the types add up, the references
    /// between namespaces that merge into one disappear and the others add up. A depth of zero
    /// leaves the graph as it is.
    #[must_use]
    pub fn collapsed(&self, depth: usize) -> Self {
        if depth == 0 {
            return self.clone();
        }
        let mut merged: BTreeMap<&str, (&QualifiedName, u32)> = BTreeMap::new();
        for node in &self.namespaces {
            let key = leading_segments(node.name.as_str(), depth);
            let entry = merged.entry(key).or_insert((&node.name, 0));
            entry.1 += node.types;
        }
        let position: HashMap<&str, usize> = merged
            .keys()
            .enumerate()
            .map(|(index, name)| (*name, index))
            .collect();
        let target = |index: usize| {
            let node = self.namespaces.get(index)?;
            position
                .get(leading_segments(node.name.as_str(), depth))
                .copied()
        };
        let counts = self.merged_edges(&target);
        Self {
            namespaces: merged
                .into_iter()
                .map(|(key, (original, types))| NamespaceNode {
                    name: QualifiedName::new(key).unwrap_or_else(|_| original.clone()),
                    types,
                })
                .collect(),
            edges: counts
                .into_iter()
                .map(|((from, to), count)| NamespaceEdge { from, to, count })
                .collect(),
        }
    }

    /// The counts of the edges between the merged namespaces; `target` tells the new position of
    /// an old one, and an edge inside one merged namespace is dropped.
    fn merged_edges(
        &self,
        target: &impl Fn(usize) -> Option<usize>,
    ) -> BTreeMap<(usize, usize), u32> {
        let mut counts = BTreeMap::new();
        for edge in &self.edges {
            if let (Some(from), Some(to)) = (target(edge.from), target(edge.to))
                && from != to
            {
                *counts.entry((from, to)).or_default() += edge.count;
            }
        }
        counts
    }
}

/// The longest name that is a proper prefix of `name` up to a separator and is in `known`.
fn enclosing<'a>(known: &Known<'a>, name: &str) -> Option<&'a str> {
    let mut cuts: Vec<usize> = name
        .match_indices(NAME_SEPARATOR)
        .chain(name.match_indices(MODULE_NAME_SEPARATOR))
        .map(|(position, _)| position)
        .collect();
    cuts.sort_unstable();
    cuts.iter()
        .rev()
        .find_map(|&cut| known.get_key_value(name.get(..cut)?).map(|(key, _)| *key))
}

type Known<'a> = HashMap<&'a str, &'a QualifiedName>;
type Types<'a> = BTreeMap<&'a str, u32>;
type Owners<'a> = HashMap<SymbolId, &'a str>;

/// The namespace of every symbol that has one, and the number of types of every namespace.
fn assign<'a>(
    symbols: &[(SymbolId, SymbolKind, &'a QualifiedName)],
    known: &Known<'a>,
) -> (Types<'a>, Owners<'a>) {
    let mut types: BTreeMap<&str, u32> = known.keys().map(|name| (*name, 0)).collect();
    let mut owner = Owners::new();
    for (id, kind, name) in symbols {
        if let Some(found) = enclosing(known, name.as_str()) {
            owner.insert(*id, found);
            if kind.is_type() {
                *types.entry(found).or_default() += 1;
            }
        }
    }
    (types, owner)
}

/// Counts the references between different namespaces, by the positions of the two ends.
fn count_pairs(
    owner: &Owners<'_>,
    position: &HashMap<&str, usize>,
    references: impl IntoIterator<Item = (SymbolId, SymbolId)>,
) -> BTreeMap<(usize, usize), u32> {
    let mut counts = BTreeMap::new();
    for (from, to) in references {
        let ends = (owner.get(&from), owner.get(&to));
        if let (Some(a), Some(b)) = ends
            && a != b
            && let (Some(&a), Some(&b)) = (position.get(a), position.get(b))
        {
            *counts.entry((a, b)).or_default() += 1;
        }
    }
    counts
}

/// Builds the graph from the symbols (id, kind, qualified name) and the references (from, to). A
/// namespace that holds no type and has no dependency, such as the wrapper that a language pack
/// puts around a file, is left out.
#[must_use]
pub fn namespace_graph<'a>(
    symbols: impl IntoIterator<Item = (SymbolId, SymbolKind, &'a QualifiedName)>,
    references: impl IntoIterator<Item = (SymbolId, SymbolId)>,
) -> NamespaceGraph {
    let symbols: Vec<_> = symbols.into_iter().collect();
    let known: Known<'_> = symbols
        .iter()
        .filter(|(_, kind, _)| matches!(kind, SymbolKind::Namespace | SymbolKind::Module))
        .map(|(_, _, name)| (name.as_str(), *name))
        .collect();
    let (types, owner) = assign(&symbols, &known);
    let position: HashMap<&str, usize> = types
        .keys()
        .enumerate()
        .map(|(index, name)| (*name, index))
        .collect();
    let counts = count_pairs(&owner, &position, references);
    keep_used(&types, &known, counts)
}

/// Drops the namespaces without types and without edges and renumbers the edges to the rest.
fn keep_used(
    types: &Types<'_>,
    known: &Known<'_>,
    counts: BTreeMap<(usize, usize), u32>,
) -> NamespaceGraph {
    let linked: HashSet<usize> = counts.keys().flat_map(|(a, b)| [*a, *b]).collect();
    let mut kept = Vec::new();
    let mut moved: HashMap<usize, usize> = HashMap::new();
    for (index, (name, count)) in types.iter().enumerate() {
        if let Some(qualified) = known.get(name)
            && (*count > 0 || linked.contains(&index))
        {
            moved.insert(index, kept.len());
            kept.push(NamespaceNode {
                name: (*qualified).clone(),
                types: *count,
            });
        }
    }
    let edges = counts
        .into_iter()
        .filter_map(|((from, to), count)| {
            Some(NamespaceEdge {
                from: *moved.get(&from)?,
                to: *moved.get(&to)?,
                count,
            })
        })
        .collect();
    NamespaceGraph {
        namespaces: kept,
        edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> QualifiedName {
        QualifiedName::new(text).unwrap()
    }

    fn graph_of(
        names: &[(u64, SymbolKind, &str)],
        calls: impl IntoIterator<Item = (SymbolId, SymbolId)>,
    ) -> NamespaceGraph {
        let qualified: Vec<_> = names.iter().map(|(_, _, text)| name(text)).collect();
        let symbols = names
            .iter()
            .zip(&qualified)
            .map(|((id, kind, _), qualified)| (SymbolId::new(*id), *kind, qualified));
        namespace_graph(symbols, calls)
    }

    #[test]
    fn references_between_namespaces_are_counted_and_types_are_assigned() {
        let names = [
            (1, SymbolKind::Namespace, "App.Views"),
            (2, SymbolKind::Namespace, "App.Domain"),
            (3, SymbolKind::Class, "App.Views.Board"),
            (4, SymbolKind::Method, "App.Views.Board.Draw"),
            (5, SymbolKind::Class, "App.Domain.Piece"),
            (6, SymbolKind::Class, "Outside.Thing"),
        ];
        let calls = [(4, 5), (4, 5), (3, 4), (6, 5)]
            .map(|(from, to)| (SymbolId::new(from), SymbolId::new(to)));
        let graph = graph_of(&names, calls);
        let found: Vec<_> = graph
            .namespaces
            .iter()
            .map(|node| (node.name.as_str(), node.types))
            .collect();
        assert_eq!(found, [("App.Domain", 1), ("App.Views", 1)]);
        assert_eq!(
            graph.edges,
            [NamespaceEdge {
                from: 1,
                to: 0,
                count: 2
            }]
        );
    }

    #[test]
    fn module_paths_use_the_double_colon_and_empty_namespaces_are_left_out() {
        let names = [
            (1, SymbolKind::Module, "crate::store"),
            (2, SymbolKind::Struct, "crate::store::Store"),
            (3, SymbolKind::Module, "crate::empty"),
        ];
        let graph = graph_of(&names, []);
        let found: Vec<_> = graph
            .namespaces
            .iter()
            .map(|node| node.name.as_str())
            .collect();
        assert_eq!(found, ["crate::store"]);
    }

    fn collapsible() -> NamespaceGraph {
        let names = [
            (1, SymbolKind::Namespace, "App.Views"),
            (2, SymbolKind::Namespace, "App.Views.Menus"),
            (3, SymbolKind::Namespace, "App.Domain"),
            (4, SymbolKind::Class, "App.Views.Board"),
            (5, SymbolKind::Class, "App.Views.Menus.Menu"),
            (6, SymbolKind::Class, "App.Domain.Piece"),
        ];
        let calls =
            [(4, 5), (4, 6), (5, 6)].map(|(from, to)| (SymbolId::new(from), SymbolId::new(to)));
        graph_of(&names, calls)
    }

    #[test]
    fn a_depth_merges_the_namespaces_below_it_and_adds_up_their_references() {
        let top = collapsible().collapsed(1);
        let found: Vec<_> = top
            .namespaces
            .iter()
            .map(|n| (n.name.as_str(), n.types))
            .collect();
        assert_eq!(found, [("App", 3)]);
        assert!(
            top.edges.is_empty(),
            "references inside one box are not drawn"
        );
        let two = collapsible().collapsed(2);
        let found: Vec<_> = two
            .namespaces
            .iter()
            .map(|n| (n.name.as_str(), n.types))
            .collect();
        assert_eq!(found, [("App.Domain", 1), ("App.Views", 2)]);
        assert_eq!(
            two.edges,
            [NamespaceEdge {
                from: 1,
                to: 0,
                count: 2
            }]
        );
    }

    #[test]
    fn a_depth_of_zero_or_beyond_the_names_changes_nothing() {
        let graph = collapsible();
        assert_eq!(graph.collapsed(0), graph);
        assert_eq!(graph.collapsed(9), graph);
        let modules = [
            (1, SymbolKind::Module, "crate::store::cache"),
            (2, SymbolKind::Struct, "crate::store::cache::C"),
        ];
        let nested = graph_of(&modules, []);
        assert_eq!(
            nested.collapsed(2).namespaces[0].name.as_str(),
            "crate::store"
        );
    }
}
