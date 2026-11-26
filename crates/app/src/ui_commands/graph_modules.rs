//! The command behind the tab "Modules" of the dependency map (ADR 0108): the modules of the
//! newest scan as a picture (one box per module, in the row of its layer), a table with coupling
//! and stability, the cycles between modules with the edges that end them, the dependencies that
//! break the layering, and the modules that stand out. All numbers come from the functions of
//! `codeskimr_domain::module_graph`; this file only joins them with the layers and the layout.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::module_report::{
    CycleOut, ModuleReport, ModuleRow, RiskOut, ViolationOut,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{
    Layer, Layout, LayoutEdge, LayoutGraph, LayoutNode, NamespaceEdge, NamespaceGraph, NodeIndex,
    Permission, layer_position,
};
use serde::{Deserialize, Serialize};

use super::consts::{GRAPH_MODULES, GRAPH_MODULES_DESCRIPTION, GRAPH_MODULES_TITLE};
use super::graph::{BandOut, BoxOut, GraphPorts, LinkOut, bands_out, newest};
use super::graph_focus::namespace_box;
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

/// The layout of the kept modules; the rows are the layers in their order, and a module of no
/// layer is placed by its edges.
fn layout_of(
    ports: &GraphPorts,
    report: &ModuleReport<'_>,
    shown: &[usize],
    drawn: &[&NamespaceEdge],
) -> Result<Layout, CodedError> {
    let look = &ports.look;
    let nodes = shown
        .iter()
        .map(|&at| LayoutNode {
            size: look.node_size(),
            rank: layer_position(report.layers, &report.graph.name_at(at))
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
    report: &ModuleReport<'_>,
    shown: &[usize],
) -> Result<(Vec<BoxOut>, Vec<LinkOut>, Extent), CodedError> {
    let look = &ports.look;
    let drawn: Vec<&NamespaceEdge> = report
        .graph
        .edges
        .iter()
        .filter(|edge| shown.contains(&edge.from) && shown.contains(&edge.to))
        .collect();
    let layout = layout_of(ports, report, shown, &drawn)?;
    let mut boxes: Vec<BoxOut> = shown
        .iter()
        .zip(&layout.placements)
        .map(|(&at, spot)| namespace_box(report.graph, report.layers, at, spot, look.node_size()))
        .collect();
    let links = links_of(report, &drawn);
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

fn links_of(report: &ModuleReport<'_>, drawn: &[&NamespaceEdge]) -> Vec<LinkOut> {
    let on_ring = |edge: &NamespaceEdge| {
        report
            .rings
            .iter()
            .any(|ring| ring.edges.iter().any(|one| one == edge))
    };
    let breaks = |edge: &NamespaceEdge| report.violations.iter().any(|one| &one.edge == edge);
    drawn
        .iter()
        .map(|edge| LinkOut {
            from: report.graph.name_at(edge.from),
            to: report.graph.name_at(edge.to),
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

fn answer(
    ports: &GraphPorts,
    graph: &NamespaceGraph,
    layers: &[Layer],
) -> Result<ModulesOut, CodedError> {
    let report = ModuleReport::of(graph, layers);
    let max = usize::try_from(ports.look.max_modules).unwrap_or(usize::MAX);
    let shown = report.kept(max);
    let (boxes, links, size) = picture(ports, &report, &shown)?;
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
        modules: report.rows(&shown),
        modules_more: u32::try_from(left_out).unwrap_or(u32::MAX),
        cycles: report.cycles(),
        violations: report.violation_rows(),
        risks: report.risks(&ports.look.risk_limits()),
    })
}
