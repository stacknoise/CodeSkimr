//! The command behind the tab "Modules" of the dependency map (ADR 0108): the modules of the
//! newest scan as a picture (one box per module, in the row of its layer), a table with coupling
//! and stability, the cycles between modules with the edges that end them, the dependencies that
//! break the layering, and the modules that stand out. All numbers come from the functions of
//! `codeskimr_domain::module_graph`; this file only joins them with the layers and the layout.

use std::collections::BTreeSet;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{
    Layer, Layout, LayoutEdge, LayoutGraph, LayoutNode, ModuleCoupling, ModuleCycle, NamespaceEdge,
    NamespaceGraph, NodeIndex, Permission, find_cycles, find_risks, module_coupling,
};
use serde::{Deserialize, Serialize};

use super::consts::{GRAPH_MODULES, GRAPH_MODULES_DESCRIPTION, GRAPH_MODULES_TITLE};
use super::graph::{BandOut, BoxOut, GraphPorts, LinkOut, bands_out, layer_index, newest};
use super::graph_focus::{name_of, namespace_box};
use super::graph_wrap::{Extent, arrange};

/// The input of `graph.modules`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModulesInput {
    /// Id of the project.
    pub project: u64,
    /// Merge the namespaces into their first `depth` segments; zero (or left out) keeps all.
    #[serde(default)]
    pub depth: u32,
}

/// One row of the table of modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleRow {
    /// Qualified name of the module; the key of its box.
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

/// The answer of `graph.modules`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ModulesOut {
    /// Id of the snapshot drawn, `None` before the first scan.
    pub snapshot: Option<u64>,
    /// Width of the picture.
    pub width: u32,
    /// Height of the picture.
    pub height: u32,
    /// One box per drawn module.
    pub boxes: Vec<BoxOut>,
    /// The stripes of the layers that have boxes.
    pub bands: Vec<BandOut>,
    /// The dependencies between the drawn modules.
    pub links: Vec<LinkOut>,
    /// The table, the most referenced module first.
    pub modules: Vec<ModuleRow>,
    /// How many modules the picture and the table leave out.
    pub modules_more: u32,
    /// The rings among all modules.
    pub cycles: Vec<CycleOut>,
    /// The dependencies against the layering among all modules.
    pub violations: Vec<ViolationOut>,
    /// The modules that stand out.
    pub risks: Vec<RiskOut>,
}

/// Shows the modules of a project with cycles and coupling.
pub struct GraphModules {
    pub(super) ports: Arc<GraphPorts>,
}

fn edge_out(graph: &NamespaceGraph, edge: &NamespaceEdge) -> EdgeOut {
    EdgeOut {
        from: name_of(graph, edge.from),
        to: name_of(graph, edge.to),
        count: edge.count,
    }
}

fn cycle_out(graph: &NamespaceGraph, cycle: &ModuleCycle) -> CycleOut {
    let edges = |list: &[NamespaceEdge]| list.iter().map(|edge| edge_out(graph, edge)).collect();
    CycleOut {
        modules: cycle.modules.iter().map(|&at| name_of(graph, at)).collect(),
        edges: edges(&cycle.edges),
        resolution: edges(&cycle.resolution),
    }
}

/// The layer of every module, by position, and the edges that break the layering.
fn violations_of(graph: &NamespaceGraph, layers: &[Layer]) -> Vec<(NamespaceEdge, usize, usize)> {
    let layer_at = |at: usize| layer_index(layers, &name_of(graph, at));
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            let (from, to) = (layer_at(edge.from)?, layer_at(edge.to)?);
            let target = &layers.get(to)?.name;
            (!layers.get(from)?.allows(target)).then(|| (edge.clone(), from, to))
        })
        .collect()
}

fn violation_out(
    graph: &NamespaceGraph,
    layers: &[Layer],
    found: &(NamespaceEdge, usize, usize),
) -> ViolationOut {
    let layer_name = |at: usize| {
        layers
            .get(at)
            .map(|layer| layer.name.as_str().to_owned())
            .unwrap_or_default()
    };
    ViolationOut {
        edge: edge_out(graph, &found.0),
        from_layer: layer_name(found.1),
        to_layer: layer_name(found.2),
    }
}

/// The modules the picture and the table keep: the `max` with the most references, by position.
fn kept(graph: &NamespaceGraph, coupling: &[ModuleCoupling], max: usize) -> Vec<usize> {
    let mut all: Vec<usize> = (0..graph.namespaces.len()).collect();
    all.sort_by_key(|&at| (std::cmp::Reverse(weight(coupling, at)), at));
    all.truncate(max);
    all.sort_unstable();
    all
}

/// What the rows need to know about the whole graph.
struct Facts<'a> {
    graph: &'a NamespaceGraph,
    layers: &'a [Layer],
    coupling: &'a [ModuleCoupling],
    ring_members: BTreeSet<usize>,
    violations: &'a [(NamespaceEdge, usize, usize)],
}

fn row_of(facts: &Facts<'_>, at: usize) -> Option<ModuleRow> {
    let node = facts.graph.namespaces.get(at)?;
    let own = facts.coupling.get(at)?;
    let name = node.name.to_string();
    let layer = layer_index(facts.layers, &name)
        .and_then(|found| facts.layers.get(found))
        .map(|layer| layer.name.as_str().to_owned());
    let breaking = facts
        .violations
        .iter()
        .filter(|one| one.0.from == at)
        .count();
    Some(ModuleRow {
        name,
        layer,
        incoming: own.incoming,
        outgoing: own.outgoing,
        stability: own.instability(),
        types: node.types,
        in_cycle: facts.ring_members.contains(&at),
        violations: u32::try_from(breaking).unwrap_or(u32::MAX),
    })
}

fn risks_out(
    graph: &NamespaceGraph,
    coupling: &[ModuleCoupling],
    limits: &codeskimr_domain::RiskLimits,
) -> Vec<RiskOut> {
    let stability = |at: usize| {
        coupling
            .get(at)
            .and_then(ModuleCoupling::instability)
            .unwrap_or_default()
    };
    find_risks(graph, coupling, limits)
        .into_iter()
        .map(|risk| RiskOut {
            module: name_of(graph, risk.module),
            incoming: coupling.get(risk.module).map_or(0, |one| one.incoming),
            stability: stability(risk.module),
            depends_on: name_of(graph, risk.depends_on),
            depends_on_stability: stability(risk.depends_on),
        })
        .collect()
}

/// The layout of the kept modules; the rows are the layers in their order, and a module of no
/// layer is placed by its edges.
fn layout_of(
    ports: &GraphPorts,
    facts: &Facts<'_>,
    shown: &[usize],
    drawn: &[&NamespaceEdge],
) -> Result<Layout, CodedError> {
    let look = &ports.look;
    let nodes = shown
        .iter()
        .map(|&at| LayoutNode {
            size: look.node_size(),
            rank: layer_index(facts.layers, &name_of(facts.graph, at))
                .map(|row| u32::try_from(row).unwrap_or(u32::MAX)),
        })
        .collect();
    let slot = |at: usize| shown.iter().position(|&one| one == at);
    let edges = drawn
        .iter()
        .filter_map(|edge| {
            Some(LayoutEdge {
                from: NodeIndex::new(slot(edge.from)?),
                to: NodeIndex::new(slot(edge.to)?),
            })
        })
        .collect();
    Ok(LayoutGraph::new(nodes, edges)
        .map_err(CodedError::caused_by)?
        .layout(&look.layout_params()))
}

/// Places the kept modules and the dependencies between them.
fn picture(
    ports: &GraphPorts,
    facts: &Facts<'_>,
    shown: &[usize],
    rings: &[ModuleCycle],
) -> Result<(Vec<BoxOut>, Vec<LinkOut>, Extent), CodedError> {
    let look = &ports.look;
    let drawn: Vec<&NamespaceEdge> = facts
        .graph
        .edges
        .iter()
        .filter(|edge| shown.contains(&edge.from) && shown.contains(&edge.to))
        .collect();
    let layout = layout_of(ports, facts, shown, &drawn)?;
    let mut boxes: Vec<BoxOut> = shown
        .iter()
        .zip(&layout.placements)
        .map(|(&at, spot)| namespace_box(facts.graph, facts.layers, at, spot, look.node_size()))
        .collect();
    let links = links_of(facts, &drawn, rings);
    let size = arrange(
        &mut boxes,
        Extent {
            width: layout.width,
            height: layout.height,
        },
        look,
    );
    Ok((boxes, links, size))
}

fn links_of(facts: &Facts<'_>, drawn: &[&NamespaceEdge], rings: &[ModuleCycle]) -> Vec<LinkOut> {
    let on_ring = |edge: &NamespaceEdge| {
        rings
            .iter()
            .any(|ring| ring.edges.iter().any(|one| one == edge))
    };
    let breaks = |edge: &NamespaceEdge| facts.violations.iter().any(|one| &one.0 == edge);
    drawn
        .iter()
        .map(|edge| LinkOut {
            from: name_of(facts.graph, edge.from),
            to: name_of(facts.graph, edge.to),
            count: edge.count,
            violates_layering: breaks(edge),
            in_cycle: on_ring(edge),
        })
        .collect()
}

impl Command for GraphModules {
    type Input = ModulesInput;
    type Output = ModulesOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(GRAPH_MODULES),
        title_key: GRAPH_MODULES_TITLE,
        description_key: GRAPH_MODULES_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ModulesOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let Some((project, snapshot)) = newest(ports, request.input.project)? else {
            return Ok(ModulesOut::default());
        };
        let depth = usize::try_from(request.input.depth).unwrap_or_default();
        let graph = ports.index.namespace_graph(snapshot.id)?.collapsed(depth);
        let mut out = answer(ports, &graph, &project.layers)?;
        out.snapshot = Some(snapshot.id.get());
        Ok(out)
    }
}

/// How many references a module has with all others, both directions.
fn weight(coupling: &[ModuleCoupling], at: usize) -> u64 {
    coupling.get(at).map_or(0, |one| {
        u64::from(one.incoming).saturating_add(u64::from(one.outgoing))
    })
}

/// The table rows of the kept modules, the most referenced first.
fn rows_of(facts: &Facts<'_>, shown: &[usize]) -> Vec<ModuleRow> {
    let mut order: Vec<usize> = shown.to_vec();
    order.sort_by_key(|&at| (std::cmp::Reverse(weight(facts.coupling, at)), at));
    order.iter().filter_map(|&at| row_of(facts, at)).collect()
}

fn answer(
    ports: &GraphPorts,
    graph: &NamespaceGraph,
    layers: &[Layer],
) -> Result<ModulesOut, CodedError> {
    let coupling = module_coupling(graph);
    let rings = find_cycles(graph);
    let violations = violations_of(graph, layers);
    let facts = Facts {
        graph,
        layers,
        coupling: &coupling,
        ring_members: rings
            .iter()
            .flat_map(|ring| ring.modules.iter().copied())
            .collect(),
        violations: &violations,
    };
    let max = usize::try_from(ports.look.max_modules).unwrap_or(usize::MAX);
    let shown = kept(graph, &coupling, max);
    let (boxes, links, size) = picture(ports, &facts, &shown, &rings)?;
    let bands = bands_out(layers, &boxes, &ports.look);
    let reach = bands.iter().map(|band| band.y + band.height).max();
    let left_out = graph.namespaces.len().saturating_sub(shown.len());
    Ok(ModulesOut {
        snapshot: None,
        width: size.width,
        height: size.height.max(reach.unwrap_or(0)),
        boxes,
        bands,
        links,
        modules: rows_of(&facts, &shown),
        modules_more: u32::try_from(left_out).unwrap_or(u32::MAX),
        cycles: rings.iter().map(|ring| cycle_out(graph, ring)).collect(),
        violations: violations
            .iter()
            .map(|found| violation_out(graph, layers, found))
            .collect(),
        risks: risks_out(graph, &coupling, &ports.look.risk_limits()),
    })
}

#[cfg(test)]
mod tests {
    use super::super::graph_focus::tests::graph as namespaces;
    use super::*;

    fn graph(count: usize, edges: &[(usize, usize, u32)]) -> NamespaceGraph {
        let names: Vec<String> = (0..count).map(|at| format!("M{at}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        namespaces(&refs, edges)
    }

    #[test]
    fn the_kept_modules_are_the_most_referenced_ones_in_graph_order() {
        let all = graph(4, &[(0, 1, 1), (2, 1, 9), (2, 3, 4)]);
        let coupling = module_coupling(&all);
        assert_eq!(kept(&all, &coupling, 2), [1, 2]);
        assert_eq!(kept(&all, &coupling, 10), [0, 1, 2, 3]);
    }

    #[test]
    fn a_cut_leaves_the_rings_and_the_rows_of_the_rest_alone() {
        let all = graph(3, &[(0, 1, 5), (1, 0, 4), (2, 0, 1)]);
        let coupling = module_coupling(&all);
        let rings = find_cycles(&all);
        let facts = Facts {
            graph: &all,
            layers: &[],
            coupling: &coupling,
            ring_members: rings.iter().flat_map(|r| r.modules.clone()).collect(),
            violations: &[],
        };
        let rows = rows_of(&facts, &kept(&all, &coupling, 2));
        let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["M0", "M1"]);
        assert!(rows.iter().all(|row| row.in_cycle));
        assert_eq!(rings.len(), 1);
    }
}
