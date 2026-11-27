//! The rule `cycles` (ADR 0105): every ring of modules in the module graph is a violation.

use std::collections::{BTreeMap, HashMap, VecDeque};

use codeskimr_domain::{
    ModuleCycle, NamespaceEdge, NamespaceGraph, NamespaceOwners, QualifiedName, find_cycles,
};

use super::consts::ID_CYCLES;
use super::model::{Outcome, Place, Ring, RuleKind, Violation};
use crate::ports::ReferencePlace;

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
    let mut modules = found
        .cycle
        .modules
        .iter()
        .map(|&at| name_at(graph, at))
        .collect::<Option<Vec<_>>>()?;
    // The identity in the held state is the sorted names, whatever order the graph has.
    modules.sort_unstable();
    Some(Ring {
        modules,
        chain: way_round(found.cycle, found.weakest)
            .into_iter()
            .map(|at| name_at(graph, at))
            .collect::<Option<Vec<_>>>()?,
        weakest_count: found.weakest.count,
        resolution_edges: found.cycle.resolution.len(),
    })
}

/// The rings of the module graph, found before the walk over the references, and the places of
/// their weakest edges, collected during it, together with the other rules.
pub(super) struct Rings<'g> {
    owners: NamespaceOwners<'g>,
    /// Weakest edge (positions of its two modules) to the position of its ring in `violations`.
    wanted: HashMap<(usize, usize), usize>,
    violations: Vec<Violation>,
}

impl<'g> Rings<'g> {
    /// Finds the rings of `graph`: the modules are its nodes, not collapsed.
    pub(super) fn new(graph: &'g NamespaceGraph) -> Self {
        let cycles = find_cycles(graph);
        let mut wanted = HashMap::new();
        let mut violations = Vec::new();
        for cycle in &cycles {
            let Some(weakest) = cycle.resolution.first() else {
                continue;
            };
            let found = Found { cycle, weakest };
            if let (Some(from), Some(to), Some(ring)) = (
                name_at(graph, weakest.from),
                name_at(graph, weakest.to),
                ring_of(graph, &found),
            ) {
                wanted.insert((weakest.from, weakest.to), violations.len());
                violations.push(Violation {
                    from,
                    to,
                    places: Vec::new(),
                    old: false,
                    ring: Some(ring),
                });
            }
        }
        Self {
            owners: graph.owners(),
            wanted,
            violations,
        }
    }

    /// Notes the reference if it goes along the weakest edge of a ring.
    pub(super) fn visit(&mut self, reference: &ReferencePlace) {
        if self.wanted.is_empty() {
            return;
        }
        let ends = (
            self.owners.owner_of(reference.from.as_str()),
            self.owners.owner_of(reference.to.as_str()),
        );
        if let (Some(from), Some(to)) = ends
            && let Some(&at) = self.wanted.get(&(from, to))
            && let Some(one) = self.violations.get_mut(at)
        {
            one.places.push(Place {
                path: reference.path.clone(),
                span: reference.span,
            });
        }
    }

    /// The outcome of the rule `cycles`.
    pub(super) fn into_outcome(self) -> Outcome {
        Outcome {
            id: ID_CYCLES.to_owned(),
            kind: RuleKind::Cycles,
            violations: self.violations,
        }
    }
}
