//! The dependency map of the layers as a picture: one box per layer, placed by the layered graph
//! layout (blueprint 15, screen 3). The layers keep the order they are declared in, outer layers
//! first (ADR 0031), so a reference that points against that order shows as an upward edge.

use std::collections::{HashMap, HashSet};

use crate::error::ModelError;
use crate::graph_layout::{
    Layout, LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, NodeIndex, NodeSize,
};
use crate::layer::LayerEdge;
use crate::project::{Layer, LayerName};

/// The placed layers of one dependency map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerMap {
    /// The name of every box, indexed like the placements of the layout: the declared layers in
    /// their order, then layers that only the edges name (declared at scan time, gone since).
    pub names: Vec<LayerName>,
    /// Where every box is, and how large the picture is.
    pub layout: Layout,
}

/// Places the layers and the dependencies between them. Every declared layer sits in its own row
/// in its declared position; a layer that only the edges name is placed by its edges.
///
/// # Errors
/// [`ModelError`] if the graph cannot be built, which would be a bug of this function.
pub fn layer_map(
    layers: &[Layer],
    edges: &[LayerEdge],
    node: NodeSize,
    params: &LayoutParams,
) -> Result<LayerMap, ModelError> {
    let (names, declared) = layer_names(layers, edges);
    let nodes = (0..names.len())
        .map(|position| LayoutNode {
            size: node,
            rank: (position < declared).then(|| rank_of(position)),
        })
        .collect();
    let positions: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect();
    let links = edges
        .iter()
        .filter_map(|edge| {
            Some(LayoutEdge {
                from: NodeIndex::new(*positions.get(edge.from.as_str())?),
                to: NodeIndex::new(*positions.get(edge.to.as_str())?),
            })
        })
        .collect();
    let layout = LayoutGraph::new(nodes, links)?.layout(params);
    Ok(LayerMap { names, layout })
}

/// The distinct names, declared layers first, and how many of them are declared.
fn layer_names(layers: &[Layer], edges: &[LayerEdge]) -> (Vec<LayerName>, usize) {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for name in layers.iter().map(|layer| &layer.name) {
        push_new(&mut names, &mut seen, name);
    }
    let declared = names.len();
    for name in edges.iter().flat_map(|edge| [&edge.from, &edge.to]) {
        push_new(&mut names, &mut seen, name);
    }
    (names, declared)
}

fn push_new<'a>(names: &mut Vec<LayerName>, seen: &mut HashSet<&'a str>, name: &'a LayerName) {
    if seen.insert(name.as_str()) {
        names.push(name.clone());
    }
}

fn rank_of(position: usize) -> u32 {
    u32::try_from(position).unwrap_or(u32::MAX)
}
