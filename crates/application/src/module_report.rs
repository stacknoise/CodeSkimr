//! The numbers behind the modules of a project (ADR 0108), joined with its layers: the table of
//! modules, the cycles with the edges that end them, the dependencies against the layering and
//! the modules that stand out. The tab "Modules" of the UI and the MCP tool `get_dependencies`
//! both read this report, so the doors cannot drift apart.

use std::collections::BTreeSet;

use codeskimr_domain::{
    Layer, ModuleCoupling, ModuleCycle, NamespaceEdge, NamespaceGraph, RiskLimits, find_cycles,
    find_risks, layer_position, module_coupling,
};
use serde::Serialize;

/// One row of the table of modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleRow {
    /// Qualified name of the module.
    pub name: String,
    /// Name of the layer that owns it.
    pub layer: Option<String>,
    /// Places in other modules that use it.
    pub incoming: u32,
    /// Places in it that use other modules.
    pub outgoing: u32,
    /// `outgoing / (incoming + outgoing)` in thousandths; `None` without any dependency.
    pub stability: Option<u32>,
    /// Number of types that sit directly in it.
    pub types: u32,
    /// Whether it is part of a cycle.
    pub in_cycle: bool,
    /// Number of its outgoing dependencies that break the layering.
    pub violations: u32,
}

/// A dependency between two modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EdgeOut {
    /// The depending module.
    pub from: String,
    /// The module it depends on.
    pub to: String,
    /// Number of references behind the edge.
    pub count: u32,
}

/// A ring of modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CycleOut {
    /// The modules of the ring, by name.
    pub modules: Vec<String>,
    /// Every dependency between two modules of the ring.
    pub edges: Vec<EdgeOut>,
    /// The dependencies to remove to end the ring, the one with the fewest references first.
    pub resolution: Vec<EdgeOut>,
}

/// A dependency between two modules that breaks the layering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ViolationOut {
    /// The edge.
    pub edge: EdgeOut,
    /// Layer of the depending module.
    pub from_layer: String,
    /// Layer of the module it depends on.
    pub to_layer: String,
}

/// A module that stands out: used a lot and dependent on a clearly less stable one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RiskOut {
    /// The risky module.
    pub module: String,
    /// Places that use it.
    pub incoming: u32,
    /// Its stability in thousandths.
    pub stability: u32,
    /// The less stable module it depends on.
    pub depends_on: String,
    /// The stability of that module in thousandths.
    pub depends_on_stability: u32,
}

/// An edge that breaks the layering, with the positions of the two layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The edge between two modules.
    pub edge: NamespaceEdge,
    /// Position of the layer of the depending module.
    pub from_layer: usize,
    /// Position of the layer of the module it depends on.
    pub to_layer: usize,
}

/// Coupling, rings and layer violations of all modules of a graph.
#[derive(Debug, Clone)]
pub struct ModuleReport<'a> {
    /// The graph the report is about.
    pub graph: &'a NamespaceGraph,
    /// The layers of the project.
    pub layers: &'a [Layer],
    /// The coupling of every module, in the order of the graph.
    pub coupling: Vec<ModuleCoupling>,
    /// The rings among all modules.
    pub rings: Vec<ModuleCycle>,
    /// The dependencies against the layering among all modules.
    pub violations: Vec<Violation>,
    ring_members: BTreeSet<usize>,
}

impl<'a> ModuleReport<'a> {
    /// Finds the rings, the coupling and the layer violations of `graph`.
    #[must_use]
    pub fn of(graph: &'a NamespaceGraph, layers: &'a [Layer]) -> Self {
        let rings = find_cycles(graph);
        let ring_members = rings
            .iter()
            .flat_map(|ring| ring.modules.iter().copied())
            .collect();
        Self {
            graph,
            layers,
            coupling: module_coupling(graph),
            rings,
            violations: violations_of(graph, layers),
            ring_members,
        }
    }

    /// How many references the module has with all others, both directions.
    #[must_use]
    pub fn weight(&self, at: usize) -> u64 {
        self.coupling.get(at).map_or(0, |one| {
            u64::from(one.incoming).saturating_add(u64::from(one.outgoing))
        })
    }

    /// The `max` modules with the most references, as positions in graph order.
    #[must_use]
    pub fn kept(&self, max: usize) -> Vec<usize> {
        let mut all: Vec<usize> = (0..self.graph.namespaces.len()).collect();
        all.sort_by_key(|&at| (std::cmp::Reverse(self.weight(at)), at));
        all.truncate(max);
        all.sort_unstable();
        all
    }

    /// The table rows of `shown`, the most referenced module first.
    #[must_use]
    pub fn rows(&self, shown: &[usize]) -> Vec<ModuleRow> {
        let mut order: Vec<usize> = shown.to_vec();
        order.sort_by_key(|&at| (std::cmp::Reverse(self.weight(at)), at));
        order.iter().filter_map(|&at| self.row(at)).collect()
    }

    fn row(&self, at: usize) -> Option<ModuleRow> {
        let node = self.graph.namespaces.get(at)?;
        let own = self.coupling.get(at)?;
        let name = node.name.to_string();
        let layer = layer_position(self.layers, &name)
            .and_then(|found| self.layers.get(found))
            .map(|layer| layer.name.as_str().to_owned());
        let breaking = self.violations.iter().filter(|v| v.edge.from == at).count();
        Some(ModuleRow {
            name,
            layer,
            incoming: own.incoming,
            outgoing: own.outgoing,
            stability: own.instability(),
            types: node.types,
            in_cycle: self.ring_members.contains(&at),
            violations: u32::try_from(breaking).unwrap_or(u32::MAX),
        })
    }

    /// The rings with the modules and edges by name.
    #[must_use]
    pub fn cycles(&self) -> Vec<CycleOut> {
        let edges = |list: &[NamespaceEdge]| list.iter().map(|edge| self.edge_out(edge)).collect();
        self.rings
            .iter()
            .map(|ring| CycleOut {
                modules: ring
                    .modules
                    .iter()
                    .map(|&at| self.graph.name_at(at))
                    .collect(),
                edges: edges(&ring.edges),
                resolution: edges(&ring.resolution),
            })
            .collect()
    }

    /// The dependencies against the layering with the names of both layers.
    #[must_use]
    pub fn violation_rows(&self) -> Vec<ViolationOut> {
        let layer_name = |at: usize| {
            self.layers
                .get(at)
                .map(|layer| layer.name.as_str().to_owned())
                .unwrap_or_default()
        };
        self.violations
            .iter()
            .map(|found| ViolationOut {
                edge: self.edge_out(&found.edge),
                from_layer: layer_name(found.from_layer),
                to_layer: layer_name(found.to_layer),
            })
            .collect()
    }

    /// The modules that stand out, the most used first.
    #[must_use]
    pub fn risks(&self, limits: &RiskLimits) -> Vec<RiskOut> {
        let stability = |at: usize| {
            self.coupling
                .get(at)
                .and_then(ModuleCoupling::instability)
                .unwrap_or_default()
        };
        find_risks(self.graph, &self.coupling, limits)
            .into_iter()
            .map(|risk| RiskOut {
                module: self.graph.name_at(risk.module),
                incoming: self.coupling.get(risk.module).map_or(0, |one| one.incoming),
                stability: stability(risk.module),
                depends_on: self.graph.name_at(risk.depends_on),
                depends_on_stability: stability(risk.depends_on),
            })
            .collect()
    }

    fn edge_out(&self, edge: &NamespaceEdge) -> EdgeOut {
        EdgeOut {
            from: self.graph.name_at(edge.from),
            to: self.graph.name_at(edge.to),
            count: edge.count,
        }
    }
}

/// The edges that break the layering, with the layers of both ends.
fn violations_of(graph: &NamespaceGraph, layers: &[Layer]) -> Vec<Violation> {
    let layer_at = |at: usize| layer_position(layers, &graph.name_at(at));
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            let (from, to) = (layer_at(edge.from)?, layer_at(edge.to)?);
            let target = &layers.get(to)?.name;
            (!layers.get(from)?.allows(target)).then(|| Violation {
                edge: edge.clone(),
                from_layer: from,
                to_layer: to,
            })
        })
        .collect()
}
