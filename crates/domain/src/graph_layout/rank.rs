//! Rank assignment: which row of the picture every node belongs to.
//!
//! Nodes with a fixed rank (layer bands) keep their order relative to each other; unused ranks are closed up. All others get the longest path from a source.
//! Cycles are broken by ignoring the edges that point backwards in a depth-first order, so the
//! result is always defined.

use super::LayoutGraph;
use super::slots::{list, push, set, value};

/// Returns the rank of every node, indexed like the nodes of the graph.
pub fn assign(graph: &LayoutGraph) -> Vec<usize> {
    let count = graph.nodes().len();
    let (outgoing, incoming) = adjacency(graph);
    let order = reverse_postorder(&outgoing, &incoming);
    let mut position = vec![0; count];
    for (place, &node) in order.iter().enumerate() {
        set(&mut position, node, place);
    }
    let mut rank = vec![0; count];
    for &node in &order {
        let fixed = graph.nodes().get(node).and_then(|entry| entry.rank);
        let assigned = fixed.map_or_else(
            || {
                longest_path(
                    list(&incoming, node),
                    value(&position, node),
                    &position,
                    &rank,
                )
            },
            |wanted| usize::try_from(wanted).unwrap_or_default(),
        );
        set(&mut rank, node, assigned);
    }
    compact(&mut rank);
    rank
}

/// Renumbers the used ranks without gaps, keeping their order. A huge fixed rank therefore costs
/// no memory, and empty rows never take space in the picture.
fn compact(rank: &mut [usize]) {
    let mut used = rank.to_vec();
    used.sort_unstable();
    used.dedup();
    for entry in rank.iter_mut() {
        *entry = used.binary_search(entry).unwrap_or_default();
    }
}

/// One step further than the deepest predecessor that comes earlier in the order.
fn longest_path(predecessors: &[usize], own: usize, position: &[usize], rank: &[usize]) -> usize {
    predecessors
        .iter()
        .filter(|&&before| value(position, before) < own)
        .map(|&before| value(rank, before) + 1)
        .max()
        .unwrap_or_default()
}

type Adjacency = Vec<Vec<usize>>;

/// Outgoing and incoming neighbours per node; self loops are dropped.
fn adjacency(graph: &LayoutGraph) -> (Adjacency, Adjacency) {
    let count = graph.nodes().len();
    let mut outgoing = vec![Vec::new(); count];
    let mut incoming = vec![Vec::new(); count];
    for edge in graph.edges() {
        let (from, to) = (edge.from.get(), edge.to.get());
        if from != to {
            push(&mut outgoing, from, to);
            push(&mut incoming, to, from);
        }
    }
    (outgoing, incoming)
}

/// Depth-first bookkeeping.
struct Walk {
    visited: Vec<bool>,
    cursor: Vec<usize>,
    finished: Vec<usize>,
}

/// Nodes in reverse finishing order; sources are visited first so that they stay at the top.
fn reverse_postorder(outgoing: &[Vec<usize>], incoming: &[Vec<usize>]) -> Vec<usize> {
    let count = outgoing.len();
    let mut walk = Walk {
        visited: vec![false; count],
        cursor: vec![0; count],
        finished: Vec::with_capacity(count),
    };
    let sources = (0..count).filter(|&node| list(incoming, node).is_empty());
    let others = (0..count).filter(|&node| !list(incoming, node).is_empty());
    for root in sources.chain(others) {
        if !value(&walk.visited, root) {
            descend(root, outgoing, &mut walk);
        }
    }
    walk.finished.reverse();
    walk.finished
}

/// Iterative depth-first search from one root, so deep graphs cannot overflow the stack.
fn descend(root: usize, outgoing: &[Vec<usize>], walk: &mut Walk) {
    set(&mut walk.visited, root, true);
    let mut stack = vec![root];
    while let Some(&node) = stack.last() {
        let cursor = value(&walk.cursor, node);
        set(&mut walk.cursor, node, cursor + 1);
        match list(outgoing, node).get(cursor) {
            Some(&child) if !value(&walk.visited, child) => {
                set(&mut walk.visited, child, true);
                stack.push(child);
            }
            Some(_) => {}
            None => {
                walk.finished.push(node);
                stack.pop();
            }
        }
    }
}
