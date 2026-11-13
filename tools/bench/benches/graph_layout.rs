//! Benchmark of the graph layout (blueprint 16.1: at most 200 nodes in under 50 ms).
//! The graphs are layered like real dependency maps: most edges go to the next rows, a few jump.

use std::hint::black_box;

use codeskimr_bench::random::Lcg;
use codeskimr_domain::{
    Layer, LayerEdge, LayerName, LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, ModelError,
    NodeIndex, NodeSize, QualifiedName, layer_map,
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

fn dependency_graph(nodes: usize) -> Result<LayoutGraph, ModelError> {
    let mut random = Lcg::new(42);
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

/// Layers as the command `graph.layers` gets them: each in its own row, most edges to a layer
/// close below, a few that jump.
fn layer_inputs(count: usize) -> Result<(Vec<Layer>, Vec<LayerEdge>), ModelError> {
    let mut random = Lcg::new(7);
    let layers = (0..count)
        .map(|index| {
            let name = format!("layer{index}");
            Ok(Layer {
                name: LayerName::new(&name)?,
                namespaces: vec![QualifiedName::new(&name)?],
            })
        })
        .collect::<Result<Vec<_>, ModelError>>()?;
    let mut edges = Vec::new();
    for from in 0..count - 1 {
        for _ in 0..EDGES_PER_NODE {
            let jump = 1 + usize::from(random.below(8) == 0) * random.below(3);
            let to = (from + jump).min(count - 1);
            edges.push(LayerEdge {
                from: LayerName::new(format!("layer{from}"))?,
                to: LayerName::new(format!("layer{to}"))?,
                count: 1,
                violates_layering: false,
                in_cycle: false,
            });
        }
    }
    Ok((layers, edges))
}

fn layer_maps(c: &mut Criterion) -> Result<(), ModelError> {
    let node = NodeSize {
        width: 160,
        height: 44,
    };
    let (layers, edges) = layer_inputs(200)?;
    let mut group = c.benchmark_group("graph_layers");
    group.bench_function("layers_200", |b| {
        b.iter(|| black_box(layer_map(&layers, &edges, node, &PARAMS)));
    });
    group.finish();
    Ok(())
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
    layer_maps(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
