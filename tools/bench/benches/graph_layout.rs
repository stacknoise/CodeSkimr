//! Benchmark of the graph layout (blueprint 16.1: at most 200 nodes in under 50 ms).
//! The graphs are layered like real dependency maps: most edges go to the next rows, a few jump.

use std::hint::black_box;

use codeskimr_domain::{
    LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, ModelError, NodeIndex, NodeSize,
};
use criterion::Criterion;

const PARAMS: LayoutParams = LayoutParams {
    rank_gap: 60,
    node_gap: 24,
    sweeps: 12,
    placement_passes: 6,
};
const ROWS: usize = 8;
const EDGES_PER_NODE: usize = 3;

/// Deterministic pseudo-random numbers, so every run measures the same graph.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, limit: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).unwrap_or_default() % limit
    }
}

fn dependency_graph(nodes: usize) -> Result<LayoutGraph, ModelError> {
    let mut random = Lcg(42);
    let per_row = nodes / ROWS;
    let list = (0..nodes)
        .map(|_| LayoutNode {
            size: NodeSize {
                width: 160,
                height: 44,
            },
            rank: None,
        })
        .collect();
    let mut edges = Vec::new();
    for from in 0..nodes - per_row {
        for _ in 0..EDGES_PER_NODE {
            let row = from / per_row + 1 + usize::from(random.below(8) == 0) * random.below(3);
            let target = (row * per_row + random.below(per_row)).min(nodes - 1);
            edges.push(LayoutEdge {
                from: NodeIndex::new(from),
                to: NodeIndex::new(target),
            });
        }
    }
    LayoutGraph::new(list, edges)
}

fn layouts(c: &mut Criterion) -> Result<(), ModelError> {
    let mut group = c.benchmark_group("graph_layout");
    let small = dependency_graph(200)?;
    group.bench_function("nodes_200", |b| b.iter(|| black_box(small.layout(&PARAMS))));
    let large = dependency_graph(1_000)?;
    group.bench_function("nodes_1000", |b| {
        b.iter(|| black_box(large.layout(&PARAMS)));
    });
    group.finish();
    Ok(())
}

fn main() -> Result<(), ModelError> {
    let mut criterion = Criterion::default().configure_from_args();
    layouts(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
