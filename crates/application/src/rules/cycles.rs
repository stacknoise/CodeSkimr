//! The rule `cycles` (ADR 0105): every ring of modules in the module graph is a violation.

use std::collections::{BTreeMap, HashMap, VecDeque};

use codeskimr_domain::SnapshotId;
use codeskimr_domain::{ModuleCycle, NamespaceEdge, NamespaceGraph, QualifiedName, find_cycles};

use super::consts::ID_CYCLES;
use super::model::{Outcome, Place, Ring, RuleKind, Violation};
use crate::config::{CycleRule, Rules};
use crate::error::CodedError;
use crate::ports::{ReferencePlace, ReferencePlaces};

/// A ring and the edge that is cheapest to cut.
struct Found<'a> {
    cycle: &'a ModuleCycle,
    weakest: &'a NamespaceEdge,
}

/// The way round a ring that starts with `weakest`: its two ends, then the shortest way back from
/// the module it points to. Positions in the namespaces of the graph.
fn way_round(cycle: &ModuleCycle, weakest: &NamespaceEdge) -> Vec<usize> {
    let mut before: BTreeMap<usize, usize> = BTreeMap::new();
    let mut queue = VecDeque::from([weakest.to]);
    while let Some(at) = queue.pop_front() {
        if at == weakest.from {
            break;
        }
        for edge in cycle.edges.iter().filter(|edge| edge.from == at) {
            if edge.to != weakest.to && !before.contains_key(&edge.to) {
                before.insert(edge.to, at);
                queue.push_back(edge.to);
            }
        }
    }
    let mut way = vec![weakest.from];
    let mut at = weakest.from;
    while at != weakest.to {
        let Some(&previous) = before.get(&at) else {
            break;
        };
        way.push(previous);
        at = previous;
    }
    way.reverse();
    way.insert(0, weakest.from);
    way
}

fn name_at(graph: &NamespaceGraph, at: usize) -> Option<QualifiedName> {
    graph.namespaces.get(at).map(|node| node.name.clone())
}

fn ring_of(graph: &NamespaceGraph, found: &Found<'_>) -> Option<Ring> {
    Some(Ring {
        modules: found
            .cycle
            .modules
            .iter()
            .map(|&at| name_at(graph, at))
            .collect::<Option<Vec<_>>>()?,
        chain: way_round(found.cycle, found.weakest)
            .into_iter()
            .map(|at| name_at(graph, at))
            .collect::<Option<Vec<_>>>()?,
        weakest_count: found.weakest.count,
        resolution_edges: found.cycle.resolution.len(),
    })
}

/// The places of the weakest edge of every ring: the references that go from a symbol of the one
/// module into the other.
fn places_of(
    graph: &NamespaceGraph,
    found: &[Found<'_>],
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
) -> Result<Vec<Vec<Place>>, CodedError> {
    let owners = graph.owners();
    let wanted: HashMap<(usize, usize), usize> = found
        .iter()
        .enumerate()
        .map(|(at, one)| ((one.weakest.from, one.weakest.to), at))
        .collect();
    let mut places: Vec<Vec<Place>> = found.iter().map(|_| Vec::new()).collect();
    references.for_each_reference(snapshot, &mut |reference: ReferencePlace| {
        let ends = (
            owners.owner_of(reference.from.as_str()),
            owners.owner_of(reference.to.as_str()),
        );
        if let (Some(from), Some(to)) = ends
            && let Some(&at) = wanted.get(&(from, to))
            && let Some(list) = places.get_mut(at)
        {
            list.push(Place {
                path: reference.path,
                span: reference.span,
            });
        }
    })?;
    Ok(places)
}

/// The rings of the graph with their weakest edge; a ring without an edge to cut cannot occur.
fn found_in(cycles: &[ModuleCycle]) -> Vec<Found<'_>> {
    cycles
        .iter()
        .filter_map(|cycle| {
            let weakest = cycle.resolution.first()?;
            Some(Found { cycle, weakest })
        })
        .collect()
}

/// The outcome of the rule `cycles`; `None` if the project did not switch it on. Modules are the
/// nodes of the namespace graph as the store has it, not collapsed.
///
/// # Errors
/// A coded failure of the `STO` group.
pub(super) fn cycles_outcome(
    rules: &Rules,
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
) -> Result<Option<Outcome>, CodedError> {
    if rules.cycles != CycleRule::Deny {
        return Ok(None);
    }
    let graph = references.namespaces(snapshot)?;
    let cycles = find_cycles(&graph);
    let found = found_in(&cycles);
    let places = if found.is_empty() {
        Vec::new()
    } else {
        places_of(&graph, &found, references, snapshot)?
    };
    let violations = found
        .iter()
        .zip(places)
        .filter_map(|(one, places)| {
            Some(Violation {
                from: name_at(&graph, one.weakest.from)?,
                to: name_at(&graph, one.weakest.to)?,
                places,
                old: false,
                ring: Some(ring_of(&graph, one)?),
            })
        })
        .collect();
    Ok(Some(Outcome {
        id: ID_CYCLES.to_owned(),
        kind: RuleKind::Cycles,
        violations,
    }))
}
