//! The layered graph: nodes sorted into rows, long edges split into one-row steps.
//!
//! A long edge becomes a chain of invisible nodes, one per crossed row. They take part in the
//! ordering, so edges are routed around nodes instead of through them, and they give the
//! placement the waypoints of the edge.

use super::LayoutEdge;
use super::slots::{list, push, set, value};

/// Nodes by row with links between neighbouring rows only. Real nodes come first (ids below `real`).
pub struct Layered {
    /// Row of every node, real and invisible.
    pub rank: Vec<usize>,
    /// Neighbours in the row above.
    pub up: Vec<Vec<usize>>,
    /// Neighbours in the row below.
    pub down: Vec<Vec<usize>>,
    /// Node ids per row, in drawing order.
    pub layers: Vec<Vec<usize>>,
    /// Number of real nodes.
    pub real: usize,
}

/// Builds the layered graph from the ranks of the real nodes.
pub fn build(ranks: &[usize], edges: &[LayoutEdge]) -> Layered {
    let real = ranks.len();
    let mut layered = Layered {
        rank: ranks.to_vec(),
        up: vec![Vec::new(); real],
        down: vec![Vec::new(); real],
        layers: Vec::new(),
        real,
    };
    for edge in edges {
        layered.add_edge(edge.from.get(), edge.to.get());
    }
    layered.fill_layers();
    layered
}

impl Layered {
    /// Links two real nodes through invisible nodes on every row between them.
    fn add_edge(&mut self, first: usize, second: usize) {
        let (low, high) = if value(&self.rank, first) <= value(&self.rank, second) {
            (first, second)
        } else {
            (second, first)
        };
        let (from_rank, to_rank) = (value(&self.rank, low), value(&self.rank, high));
        if from_rank >= to_rank {
            return;
        }
        let mut previous = low;
        for rank in from_rank + 1..to_rank {
            let step = self.add_step(rank);
            self.link(previous, step);
            previous = step;
        }
        self.link(previous, high);
    }

    /// Adds an invisible node on the given row.
    fn add_step(&mut self, rank: usize) -> usize {
        self.rank.push(rank);
        self.up.push(Vec::new());
        self.down.push(Vec::new());
        self.rank.len() - 1
    }

    fn link(&mut self, upper: usize, lower: usize) {
        push(&mut self.down, upper, lower);
        push(&mut self.up, lower, upper);
    }

    fn fill_layers(&mut self) {
        let rows = self.rank.iter().max().map_or(0, |&deepest| deepest + 1);
        self.layers = vec![Vec::new(); rows];
        for (node, &rank) in self.rank.iter().enumerate() {
            push(&mut self.layers, rank, node);
        }
    }

    /// Position of every node inside its row.
    pub fn positions(&self) -> Vec<usize> {
        let mut position = vec![0; self.rank.len()];
        for layer in &self.layers {
            for (place, &node) in layer.iter().enumerate() {
                set(&mut position, node, place);
            }
        }
        position
    }

    /// Rows in the order a sweep visits them: top to bottom, or bottom to top.
    pub fn sweep_rows(&self, downward: bool) -> Vec<usize> {
        let rows = self.layers.len();
        if downward {
            (1..rows).collect()
        } else {
            (0..rows.saturating_sub(1)).rev().collect()
        }
    }

    /// The neighbours a sweep looks at: the row above when sweeping down, otherwise below.
    pub fn anchors(&self, downward: bool) -> &[Vec<usize>] {
        if downward { &self.up } else { &self.down }
    }

    /// Nodes of one row.
    pub fn row(&self, index: usize) -> &[usize] {
        list(&self.layers, index)
    }
}
