//! Cycle detection on the layer graph (pure, no database).

use std::collections::{BTreeSet, HashMap};

/// Finds the edges that lie on a cycle: an edge `a -> b` does when `b` can reach `a` again.
/// Nodes are layer positions; self-edges are not part of the input.
pub fn edges_in_cycles(edges: &[(usize, usize)]) -> BTreeSet<(usize, usize)> {
    let mut next: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(from, to) in edges {
        next.entry(from).or_default().push(to);
    }
    edges
        .iter()
        .copied()
        .filter(|&(from, to)| reaches(&next, to, from))
        .collect()
}

fn reaches(next: &HashMap<usize, Vec<usize>>, start: usize, goal: usize) -> bool {
    let mut seen = BTreeSet::new();
    let mut open = vec![start];
    while let Some(node) = open.pop() {
        if node == goal {
            return true;
        }
        if seen.insert(node) {
            open.extend(next.get(&node).into_iter().flatten().copied());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::edges_in_cycles;

    #[test]
    fn acyclic_graph_has_no_cycle_edges() {
        assert!(edges_in_cycles(&[(0, 1), (1, 2), (0, 2)]).is_empty());
    }

    #[test]
    fn only_the_edges_of_the_cycle_are_reported() {
        let found = edges_in_cycles(&[(0, 1), (1, 2), (2, 1), (2, 3)]);
        assert_eq!(found.into_iter().collect::<Vec<_>>(), vec![(1, 2), (2, 1)]);
    }
}
