//! The layer rules that a scan breaks and the scan before did not (ADR 0097, decision 4).

use std::collections::HashSet;

use codeskimr_domain::{LayerEdge, LayerName};

use super::report::{LayerChange, LayerChanges};

/// The edges the target has with `flag` set that the base does not have with it set.
fn newly(
    base: &[LayerEdge],
    target: &[LayerEdge],
    flag: impl Fn(&LayerEdge) -> bool,
) -> Vec<LayerChange> {
    let known: HashSet<(&LayerName, &LayerName)> = base
        .iter()
        .filter(|edge| flag(edge))
        .map(|edge| (&edge.from, &edge.to))
        .collect();
    target
        .iter()
        .filter(|edge| flag(edge) && !known.contains(&(&edge.from, &edge.to)))
        .map(|edge| LayerChange {
            from: edge.from.clone(),
            to: edge.to.clone(),
            count: edge.count,
        })
        .collect()
}

/// The new layer violations and the new cycles of the dependency map of the target.
pub(super) fn changes(base: &[LayerEdge], target: &[LayerEdge]) -> LayerChanges {
    LayerChanges {
        new_violations: newly(base, target, |edge| edge.violates_layering),
        new_cycles: newly(base, target, |edge| edge.in_cycle),
    }
}
