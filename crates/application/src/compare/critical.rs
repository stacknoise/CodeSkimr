//! The critical symbols that a comparison touches (ADR 0097, decision 4; marks: ADR 0068).

use std::collections::HashMap;

use codeskimr_domain::{Annotation, Comparison, SymbolIdentity};

use super::report::{CriticalChange, CriticalOutcome};

/// What the comparison did to each symbol of the base scan, by key.
fn outcomes(comparison: &Comparison) -> HashMap<SymbolIdentity, CriticalOutcome> {
    let removed = comparison
        .removed
        .iter()
        .map(|fact| (fact.identity(), CriticalOutcome::Removed));
    let changed = comparison
        .changed
        .iter()
        .map(|matched| (matched.base.identity(), CriticalOutcome::Changed));
    let moved = comparison
        .moved
        .iter()
        .map(|moved| (moved.base.identity(), CriticalOutcome::Moved));
    removed.chain(changed).chain(moved).collect()
}

/// The critical annotations whose symbol the comparison removed, moved or changed.
pub(super) fn touched(critical: &[Annotation], comparison: &Comparison) -> Vec<CriticalChange> {
    let outcomes = outcomes(comparison);
    critical
        .iter()
        .filter_map(|annotation| {
            let target = &annotation.target;
            let key = SymbolIdentity {
                kind: target.kind,
                name: target.name.clone(),
                parameters: target.parameters.clone(),
            };
            outcomes.get(&key).map(|outcome| CriticalChange {
                annotation: annotation.id,
                kind: target.kind,
                name: target.name.clone(),
                outcome: *outcome,
            })
        })
        .collect()
}
