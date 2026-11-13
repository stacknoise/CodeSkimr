//! Order inside the rows: median sweeps that reduce edge crossings.
//!
//! Every sweep sorts one row by the median position of each node's neighbours in the previous
//! row, going down and then up again. The order with the fewest crossings seen is kept.

use super::consts::{KEY_SCALE, MIDDLE_DIVISOR, SWEEP_CYCLE};
use super::layers::Layered;
use super::slots::{list, set, value};

/// Reorders the rows of `layered` for at most `sweeps` sweeps.
pub fn minimize(layered: &mut Layered, sweeps: u32) {
    let mut position = layered.positions();
    let mut best = crossings(layered, &position);
    let mut best_layers = layered.layers.clone();
    for sweep in 0..sweeps {
        if best == 0 {
            break;
        }
        sweep_once(layered, &mut position, sweep % SWEEP_CYCLE == 0);
        let now = crossings(layered, &position);
        if now < best {
            best = now;
            best_layers.clone_from(&layered.layers);
        }
    }
    layered.layers = best_layers;
}

/// Sorts every row once, in sweep direction.
fn sweep_once(layered: &mut Layered, position: &mut [usize], downward: bool) {
    // The rows are taken out so that the anchors of `layered` can be read while a row is sorted.
    let rows = layered.sweep_rows(downward);
    let mut layers = std::mem::take(&mut layered.layers);
    let anchors = layered.anchors(downward);
    for index in rows {
        if let Some(layer) = layers.get_mut(index) {
            reorder(layer, anchors, position);
        }
    }
    layered.layers = layers;
}

/// Sorts one row by the median of the anchors; nodes without anchors keep their place.
fn reorder(layer: &mut [usize], anchors: &[Vec<usize>], position: &mut [usize]) {
    let mut keyed: Vec<(usize, usize, usize)> = layer
        .iter()
        .map(|&node| {
            let own = value(position, node);
            let key = median_key(list(anchors, node), position).unwrap_or(own * KEY_SCALE);
            (key, own, node)
        })
        .collect();
    keyed.sort_unstable();
    for (place, &(_, _, node)) in keyed.iter().enumerate() {
        set(layer, place, node);
        set(position, node, place);
    }
}

/// Median position of the anchors, doubled so that two middle values average without rounding.
fn median_key(anchors: &[usize], position: &[usize]) -> Option<usize> {
    let mut spots: Vec<usize> = anchors.iter().map(|&node| value(position, node)).collect();
    spots.sort_unstable();
    let middle = spots.len() / MIDDLE_DIVISOR;
    let upper = *spots.get(middle)?;
    if spots.len() % MIDDLE_DIVISOR == 1 {
        return Some(upper * KEY_SCALE);
    }
    Some(upper + *spots.get(middle.checked_sub(1)?)?)
}

/// Number of crossing edge pairs in the whole layered graph.
pub fn crossings(layered: &Layered, position: &[usize]) -> usize {
    (0..layered.layers.len())
        .map(|index| crossings_below(layered, index, position))
        .sum()
}

/// Crossings between one row and the next: inversions of the target positions of its edges.
fn crossings_below(layered: &Layered, index: usize, position: &[usize]) -> usize {
    let width = list(&layered.layers, index + 1).len();
    let mut edges: Vec<(usize, usize)> = layered
        .row(index)
        .iter()
        .flat_map(|&upper| {
            list(&layered.down, upper)
                .iter()
                .map(move |&lower| (value(position, upper), value(position, lower)))
        })
        .collect();
    edges.sort_unstable();
    let targets: Vec<usize> = edges.iter().map(|&(_, target)| target).collect();
    inversions(&targets, width)
}

/// Counts pairs that are in decreasing order, with a Fenwick tree in O(n log n).
fn inversions(sequence: &[usize], width: usize) -> usize {
    let mut tree = vec![0_usize; width + 1];
    let mut total = 0;
    for (seen, &target) in sequence.iter().enumerate() {
        total += seen - prefix_sum(&tree, target + 1);
        add_one(&mut tree, target + 1);
    }
    total
}

fn prefix_sum(tree: &[usize], mut index: usize) -> usize {
    let mut sum = 0;
    while index > 0 {
        sum += value(tree, index);
        index -= lowest_bit(index);
    }
    sum
}

fn add_one(tree: &mut [usize], mut index: usize) {
    while index < tree.len() {
        if let Some(slot) = tree.get_mut(index) {
            *slot += 1;
        }
        index += lowest_bit(index);
    }
}

/// The lowest set bit, which is the step size of the Fenwick tree.
fn lowest_bit(index: usize) -> usize {
    index & index.wrapping_neg()
}
