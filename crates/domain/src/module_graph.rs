//! Cycles and coupling between modules (ADR 0108). A module is a node of the [`NamespaceGraph`]: a
//! namespace, a Rust module or a package, at the depth the graph was collapsed to.
//!
//! * A **cycle** is a group of modules that depend on each other in a ring (a strongly connected
//!   component of more than one module). For each one, [`find_cycles`] names the edges that end it
//!   when they are removed: the ones with the fewest references first, until no ring is left.
//! * **Coupling** counts references, not modules: `incoming` is the number of places in other
//!   modules that use a module, `outgoing` the number of places in it that use other modules.
//!   The stability (here: instability) is `outgoing / (incoming + outgoing)`. 0 means many depend
//!   on the module and it depends on little, 1 the opposite.
//! * A **risk** is a module that many places depend on and that itself depends on a module that
//!   is clearly less stable than it is.

use std::collections::{BTreeMap, BTreeSet};

use crate::consts::PERMILLE;
use crate::namespace_graph::{NamespaceEdge, NamespaceGraph};

/// One ring of modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCycle {
    /// Positions (in [`NamespaceGraph::namespaces`]) of the modules in the ring, ascending.
    pub modules: Vec<usize>,
    /// Every dependency between two modules of the ring.
    pub edges: Vec<NamespaceEdge>,
    /// The dependencies to remove to end the ring, the one with the fewest references first.
    pub resolution: Vec<NamespaceEdge>,
}

/// The references that go into a module and out of it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModuleCoupling {
    /// Places in other modules that use this one.
    pub incoming: u32,
    /// Places in this module that use other modules.
    pub outgoing: u32,
}

impl ModuleCoupling {
    /// `outgoing / (incoming + outgoing)` in thousandths; `None` for a module without any
    /// dependency in either direction.
    #[must_use]
    pub fn instability(&self) -> Option<u32> {
        let total = u64::from(self.incoming) + u64::from(self.outgoing);
        let share = (u64::from(self.outgoing) * u64::from(PERMILLE)).checked_div(total)?;
        u32::try_from(share).ok()
    }
}

/// When a module counts as a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskLimits {
    /// Fewest incoming references of a risky module.
    pub min_incoming: u32,
    /// How much less stable (in thousandths) the module it depends on must be.
    pub instability_gap: u32,
    /// Most risks to report; the ones with the most incoming references come first.
    pub max_risks: usize,
}

/// A module that is used a lot and depends on a clearly less stable one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleRisk {
    /// Position of the risky module.
    pub module: usize,
    /// Position of the less stable module it depends on (the least stable one if several).
    pub depends_on: usize,
}

/// Strongly connected components of the graph that the edges span, found without recursion
/// (Tarjan). Nodes that no edge touches do not appear. Every component is sorted.
fn components(edges: &[NamespaceEdge]) -> Vec<Vec<usize>> {
    let mut next: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for edge in edges {
        next.entry(edge.from).or_default().push(edge.to);
        next.entry(edge.to).or_default();
    }
    let mut state = Tarjan::default();
    for &start in next.keys() {
        if !state.index.contains_key(&start) {
            state.walk(start, &next);
        }
    }
    state.found
}

/// The bookkeeping of the walk.
#[derive(Default)]
struct Tarjan {
    index: BTreeMap<usize, usize>,
    low: BTreeMap<usize, usize>,
    on_stack: BTreeSet<usize>,
    stack: Vec<usize>,
    found: Vec<Vec<usize>>,
}

impl Tarjan {
    fn enter(&mut self, node: usize) {
        let at = self.index.len();
        self.index.insert(node, at);
        self.low.insert(node, at);
        self.stack.push(node);
        self.on_stack.insert(node);
    }

    fn lower(&mut self, node: usize, to: usize) {
        if let Some(low) = self.low.get_mut(&node) {
            *low = (*low).min(to);
        }
    }

    fn leave(&mut self, node: usize) {
        if self.low.get(&node) != self.index.get(&node) {
            return;
        }
        let mut component = Vec::new();
        while let Some(member) = self.stack.pop() {
            self.on_stack.remove(&member);
            component.push(member);
            if member == node {
                break;
            }
        }
        component.sort_unstable();
        self.found.push(component);
    }

    /// Visits everything reachable from `start`; the open nodes sit on an explicit stack.
    fn walk(&mut self, start: usize, next: &BTreeMap<usize, Vec<usize>>) {
        let mut open: Vec<(usize, usize)> = vec![(start, 0)];
        self.enter(start);
        while let Some(&(node, child)) = open.last() {
            let target = next.get(&node).and_then(|list| list.get(child)).copied();
            let Some(target) = target else {
                open.pop();
                self.leave(node);
                if let Some(&(parent, _)) = open.last() {
                    let low = self.low.get(&node).copied().unwrap_or_default();
                    self.lower(parent, low);
                }
                continue;
            };
            if let Some(top) = open.last_mut() {
                top.1 += 1;
            }
            if !self.index.contains_key(&target) {
                self.enter(target);
                open.push((target, 0));
            } else if self.on_stack.contains(&target) {
                let at = self.index.get(&target).copied().unwrap_or_default();
                self.lower(node, at);
            }
        }
    }
}

/// The edges that lie on a ring: both ends sit in the same component of more than one module.
fn ring_edges(edges: &[NamespaceEdge]) -> Vec<NamespaceEdge> {
    let mut ring_of: BTreeMap<usize, usize> = BTreeMap::new();
    for (at, component) in components(edges).iter().enumerate() {
        if component.len() > 1 {
            ring_of.extend(component.iter().map(|&node| (node, at)));
        }
    }
    edges
        .iter()
        .filter(|edge| {
            let ends = (ring_of.get(&edge.from), ring_of.get(&edge.to));
            matches!(ends, (Some(a), Some(b)) if a == b)
        })
        .cloned()
        .collect()
}

/// The edges to remove from a ring so that none is left: again and again the one with the fewest
/// references among the edges that still lie on a ring (ties by position).
fn resolution(ring: &[NamespaceEdge]) -> Vec<NamespaceEdge> {
    let mut left = ring.to_vec();
    let mut cut = Vec::new();
    while let Some(weakest) = ring_edges(&left)
        .into_iter()
        .min_by_key(|edge| (edge.count, edge.from, edge.to))
    {
        left.retain(|edge| (edge.from, edge.to) != (weakest.from, weakest.to));
        cut.push(weakest);
    }
    cut
}

/// The rings of the graph, ordered by the position of their first module. A module that only
/// refers to itself is no ring (the graph has no such edge).
#[must_use]
pub fn find_cycles(graph: &NamespaceGraph) -> Vec<ModuleCycle> {
    let mut found: Vec<ModuleCycle> = components(&graph.edges)
        .into_iter()
        .filter(|component| component.len() > 1)
        .map(|modules| {
            let members: BTreeSet<usize> = modules.iter().copied().collect();
            let edges: Vec<NamespaceEdge> = graph
                .edges
                .iter()
                .filter(|edge| members.contains(&edge.from) && members.contains(&edge.to))
                .cloned()
                .collect();
            let resolution = resolution(&edges);
            ModuleCycle {
                modules,
                edges,
                resolution,
            }
        })
        .collect();
    found.sort_by_key(|cycle| cycle.modules.first().copied());
    found
}

/// The coupling of every module, in the order of [`NamespaceGraph::namespaces`].
#[must_use]
pub fn module_coupling(graph: &NamespaceGraph) -> Vec<ModuleCoupling> {
    let mut all = vec![ModuleCoupling::default(); graph.namespaces.len()];
    for edge in &graph.edges {
        if let Some(from) = all.get_mut(edge.from) {
            from.outgoing = from.outgoing.saturating_add(edge.count);
        }
        if let Some(to) = all.get_mut(edge.to) {
            to.incoming = to.incoming.saturating_add(edge.count);
        }
    }
    all
}

/// The risky modules, the most used first. `coupling` is the answer of [`module_coupling`] for the
/// same graph.
#[must_use]
pub fn find_risks(
    graph: &NamespaceGraph,
    coupling: &[ModuleCoupling],
    limits: &RiskLimits,
) -> Vec<ModuleRisk> {
    let instability = |at: usize| coupling.get(at).and_then(ModuleCoupling::instability);
    let mut risks: Vec<ModuleRisk> = Vec::new();
    for (at, own) in coupling.iter().enumerate() {
        let Some(mine) = instability(at).filter(|_| own.incoming >= limits.min_incoming) else {
            continue;
        };
        let worst = graph
            .edges
            .iter()
            .filter(|edge| edge.from == at)
            .filter_map(|edge| Some((edge.to, instability(edge.to)?)))
            .filter(|&(_, theirs)| theirs >= mine.saturating_add(limits.instability_gap))
            .max_by_key(|&(to, theirs)| (theirs, std::cmp::Reverse(to)));
        if let Some((depends_on, _)) = worst {
            risks.push(ModuleRisk {
                module: at,
                depends_on,
            });
        }
    }
    risks.sort_by_key(|risk| {
        let used = coupling.get(risk.module).map_or(0, |one| one.incoming);
        (std::cmp::Reverse(used), risk.module)
    });
    risks.truncate(limits.max_risks);
    risks
}
