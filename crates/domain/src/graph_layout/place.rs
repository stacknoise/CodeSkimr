//! Coordinates: rows get their height, nodes get their horizontal place.
//!
//! The start is a centred packing of every row. A few alternating passes then pull each node
//! towards the middle of its neighbours in the adjacent row and push apart whatever overlaps,
//! never changing the order found by the crossing minimisation.

use super::consts::{HALF, PASS_CYCLE};
use super::layers::Layered;
use super::slots::{list, set, value};
use super::{LayoutParams, NodeSize};

/// Top-left corners of all nodes (real and invisible) and the size of the whole picture.
pub struct Geometry {
    /// Left edge per node.
    pub x: Vec<i64>,
    /// Top edge per node.
    pub y: Vec<i64>,
    /// Width of the picture.
    pub width: i64,
    /// Height of the picture.
    pub height: i64,
}

/// Computes the geometry of a layered graph whose rows are already in order.
pub fn place(layered: &Layered, sizes: &[NodeSize], params: &LayoutParams) -> Geometry {
    let width = extents(layered, sizes, |size| size.width);
    let height = extents(layered, sizes, |size| size.height);
    let gap = i64::from(params.node_gap);
    let mut x = initial_x(layered, &width, gap);
    for pass in 0..params.placement_passes {
        refine(layered, &width, &mut x, gap, pass % PASS_CYCLE == 0);
    }
    let (y, total_height) = rows(layered, &height, i64::from(params.rank_gap));
    let left = x.iter().copied().min().unwrap_or_default();
    let x: Vec<i64> = x.iter().map(|&value| value - left).collect();
    let total_width = x
        .iter()
        .zip(&width)
        .map(|(&at, &wide)| at + wide)
        .max()
        .unwrap_or_default();
    Geometry {
        x,
        y,
        width: total_width,
        height: total_height,
    }
}

/// One measure per node; invisible nodes take no space.
fn extents(layered: &Layered, sizes: &[NodeSize], measure: fn(&NodeSize) -> u32) -> Vec<i64> {
    (0..layered.rank.len())
        .map(|node| {
            if node < layered.real {
                sizes.get(node).map_or(0, |size| i64::from(measure(size)))
            } else {
                0
            }
        })
        .collect()
}

/// Total width of a row including the gaps between its nodes.
fn row_width(layer: &[usize], width: &[i64], gap: i64) -> i64 {
    let gaps = i64::try_from(layer.len().saturating_sub(1)).unwrap_or_default();
    layer.iter().map(|&node| value(width, node)).sum::<i64>() + gaps * gap
}

/// Rows packed from the left, each centred on the widest row.
fn initial_x(layered: &Layered, width: &[i64], gap: i64) -> Vec<i64> {
    let widest = layered
        .layers
        .iter()
        .map(|layer| row_width(layer, width, gap))
        .max()
        .unwrap_or_default();
    let mut x = vec![0; layered.rank.len()];
    for layer in &layered.layers {
        let mut cursor = (widest - row_width(layer, width, gap)) / HALF;
        for &node in layer {
            set(&mut x, node, cursor);
            cursor += value(width, node) + gap;
        }
    }
    x
}

/// One pass over all rows in sweep direction.
fn refine(layered: &Layered, width: &[i64], x: &mut [i64], gap: i64, downward: bool) {
    let anchors = layered.anchors(downward);
    for index in layered.sweep_rows(downward) {
        align(layered.row(index), anchors, width, x, gap);
    }
}

/// Moves each node of a row towards its neighbours, left to right, keeping the gap.
fn align(layer: &[usize], anchors: &[Vec<usize>], width: &[i64], x: &mut [i64], gap: i64) {
    let wanted: Vec<i64> = layer
        .iter()
        .map(|&node| wanted_x(node, list(anchors, node), width, x))
        .collect();
    let mut free_from: Option<i64> = None;
    for (&node, &want) in layer.iter().zip(&wanted) {
        let left = free_from.map_or(want, |edge| want.max(edge));
        set(x, node, left);
        free_from = Some(left + value(width, node) + gap);
    }
}

/// Left edge that centres the node over its neighbours; unchanged without neighbours.
fn wanted_x(node: usize, neighbours: &[usize], width: &[i64], x: &[i64]) -> i64 {
    let count = i64::try_from(neighbours.len()).unwrap_or(1);
    if neighbours.is_empty() {
        return value(x, node);
    }
    let centres: i64 = neighbours
        .iter()
        .map(|&other| value(x, other) + value(width, other) / HALF)
        .sum();
    centres / count - value(width, node) / HALF
}

/// Top edge of every node (centred in its row) and the height of the picture.
fn rows(layered: &Layered, height: &[i64], gap: i64) -> (Vec<i64>, i64) {
    let mut y = vec![0; layered.rank.len()];
    let mut top = 0;
    for (index, layer) in layered.layers.iter().enumerate() {
        let tallest = layer
            .iter()
            .map(|&node| value(height, node))
            .max()
            .unwrap_or_default();
        for &node in layer {
            set(&mut y, node, top + (tallest - value(height, node)) / HALF);
        }
        top += tallest;
        if index + 1 < layered.layers.len() {
            top += gap;
        }
    }
    (y, top)
}
