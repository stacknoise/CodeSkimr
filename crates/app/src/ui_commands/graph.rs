//! The command behind the dependency map: the layers of the newest scan, placed by the graph
//! layout, with the aggregated dependencies between them. The layout is computed here, in Rust;
//! the UI only draws the result (blueprint 15.5).

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::Graph;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{
    Layer, LayerEdge, LayerMap, Layout, LayoutEdge, LayoutGraph, LayoutNode, NamespaceGraph,
    NodeIndex, Permission, ProjectId, SnapshotId, layer_map,
};
use serde::{Deserialize, Serialize};

use super::consts::{BAND_PAD_DIVISOR, GRAPH_DESCRIPTION, GRAPH_LAYERS, GRAPH_TITLE};
use super::graph_types::types_out;

/// What a box of the map stands for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// One box per layer.
    #[default]
    Layers,
    /// One box per namespace, in the row of its layer.
    Namespaces,
    /// One type in the middle, the types that use it above and the types it uses below.
    Types,
}

/// The input of `graph.layers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct GraphInput {
    /// Id of the project.
    pub project: u64,
    /// What a box stands for; the layers if left out.
    #[serde(default)]
    pub level: Level,
    /// On the level `namespaces`: merge the namespaces into their first `depth` segments; zero (or
    /// left out) keeps every namespace.
    #[serde(default)]
    pub depth: u32,
    /// On the level `types`: id of the type in the middle.
    #[serde(default)]
    pub focus: Option<u64>,
}

/// One layer box of the map. `x` and `y` are the top-left corner in layout units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoxOut {
    /// Name of the layer.
    pub name: String,
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width of the box.
    pub width: u32,
    /// Height of the box.
    pub height: u32,
    /// Row, counted from 0 at the top.
    pub rank: u32,
    /// Position in the row, counted from 0 on the left.
    pub order: u32,
    /// Name of the layer that owns the box; for a layer box its own name.
    pub layer: Option<String>,
    /// Number of types in a namespace box; `None` on the layer level.
    pub types: Option<u32>,
    /// Id of the type of a box on the level `types`; `None` on the other levels.
    pub symbol: Option<u64>,
}

/// The background of one layer: a stripe over the whole width behind the rows of its boxes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BandOut {
    /// Name of the layer.
    pub name: String,
    /// Row of the layer, counted from 0 at the top.
    pub rank: u32,
    /// Top edge.
    pub y: u32,
    /// Height of the stripe.
    pub height: u32,
}

/// One dependency between two layers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkOut {
    /// The depending layer.
    pub from: String,
    /// The layer it depends on.
    pub to: String,
    /// Number of references behind the edge.
    pub count: u32,
    /// Whether the edge breaks the declared layering.
    pub violates_layering: bool,
    /// Whether the edge is part of a dependency cycle.
    pub in_cycle: bool,
}

/// The answer of `graph.layers`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GraphOut {
    /// Id of the snapshot drawn, `None` before the first scan.
    pub snapshot: Option<u64>,
    /// Width of the whole picture.
    pub width: u32,
    /// Height of the whole picture.
    pub height: u32,
    /// The placed boxes: the layers in layout order of the project, or the namespaces.
    pub boxes: Vec<BoxOut>,
    /// The stripes of the layers that have boxes, outer layers first.
    pub bands: Vec<BandOut>,
    /// The dependencies between the layers.
    pub links: Vec<LinkOut>,
    /// On the level `types`: the type in the middle and what the picture leaves out.
    pub focus: Option<FocusOut>,
}

/// The type in the middle of a map on the level `types`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FocusOut {
    /// Id of the type.
    pub symbol: u64,
    /// Qualified name of the type, as the name of its box.
    pub name: String,
    /// Number of types that use it and are not drawn.
    pub users_more: u32,
    /// Number of types it uses that are not drawn.
    pub used_more: u32,
}

/// What the map reads: the project repository for the layers, the index for the layer graph, and
/// the configured look of the picture.
pub struct GraphPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) look: Graph,
}

/// Places the layers of a project and the dependencies between them.
pub struct GraphLayers {
    pub(super) ports: Arc<GraphPorts>,
}

fn link_out(edge: &LayerEdge) -> LinkOut {
    LinkOut {
        from: edge.from.as_str().to_owned(),
        to: edge.to.as_str().to_owned(),
        count: edge.count,
        violates_layering: edge.violates_layering,
        in_cycle: edge.in_cycle,
    }
}

fn boxes_out(map: &LayerMap, look: &Graph) -> Vec<BoxOut> {
    let size = look.node_size();
    map.names
        .iter()
        .zip(&map.layout.placements)
        .map(|(name, place)| BoxOut {
            name: name.as_str().to_owned(),
            x: place.x,
            y: place.y,
            width: size.width,
            height: size.height,
            rank: place.rank,
            order: place.order,
            layer: Some(name.as_str().to_owned()),
            types: None,
            symbol: None,
        })
        .collect()
}

/// The stripes behind the rows: one per layer that owns a box, as high as its boxes plus half
/// the gap between rows above and below.
fn bands_out(layers: &[Layer], boxes: &[BoxOut], look: &Graph) -> Vec<BandOut> {
    let pad = look.rank_gap / BAND_PAD_DIVISOR;
    layers
        .iter()
        .zip(0..)
        .filter_map(|(layer, rank)| {
            let owned = || {
                boxes
                    .iter()
                    .filter(|found| found.layer.as_deref() == Some(layer.name.as_str()))
            };
            let top = owned().map(|found| found.y).min()?;
            let bottom = owned().map(|found| found.y + found.height).max()?;
            let from = top.saturating_sub(pad);
            Some(BandOut {
                name: layer.name.as_str().to_owned(),
                rank,
                y: from,
                height: bottom + pad - from,
            })
        })
        .collect()
}

impl Command for GraphLayers {
    type Input = GraphInput;
    type Output = GraphOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(GRAPH_LAYERS),
        title_key: GRAPH_TITLE,
        description_key: GRAPH_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, GraphInput>) -> Result<GraphOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let id = ProjectId::new(request.input.project);
        let project = ports.projects.project(id)?;
        let Some(snapshot) = ports.index.latest_snapshot(id)? else {
            return Ok(GraphOut::default());
        };
        let look = &ports.look;
        let mut out = match request.input.level {
            Level::Layers => layers_out(ports, &project.layers, snapshot.id)?,
            Level::Namespaces => {
                namespaces_out(ports, &project.layers, snapshot.id, request.input.depth)?
            }
            Level::Types => types_out(ports, &project, snapshot.id, request.input.focus)?,
        };
        out.snapshot = Some(snapshot.id.get());
        if request.input.level != Level::Types {
            out.bands = bands_out(&project.layers, &out.boxes, look);
        }
        let reach = out.bands.iter().map(|band| band.y + band.height).max();
        out.height = out.height.max(reach.unwrap_or(0));
        Ok(out)
    }
}

fn layers_out(
    ports: &GraphPorts,
    layers: &[Layer],
    snapshot: SnapshotId,
) -> Result<GraphOut, CodedError> {
    let edges = ports.index.layer_edges(snapshot)?;
    let look = &ports.look;
    let map = layer_map(layers, &edges, look.node_size(), &look.layout_params())
        .map_err(CodedError::caused_by)?;
    Ok(GraphOut {
        width: map.layout.width,
        height: map.layout.height,
        boxes: boxes_out(&map, look),
        links: edges.iter().map(link_out).collect(),
        ..GraphOut::default()
    })
}

fn layer_index(layers: &[Layer], name: &str) -> Option<usize> {
    layers.iter().position(|layer| layer.owns(name))
}

/// Places the namespaces, each in the row of the layer that owns it; a namespace of no layer is
/// placed by its edges.
fn namespace_layout(
    look: &Graph,
    graph: &NamespaceGraph,
    owners: &[Option<usize>],
) -> Result<Layout, CodedError> {
    let nodes = owners
        .iter()
        .map(|owner| LayoutNode {
            size: look.node_size(),
            rank: owner.and_then(|position| u32::try_from(position).ok()),
        })
        .collect();
    let links = graph
        .edges
        .iter()
        .map(|edge| LayoutEdge {
            from: NodeIndex::new(edge.from),
            to: NodeIndex::new(edge.to),
        })
        .collect();
    Ok(LayoutGraph::new(nodes, links)
        .map_err(CodedError::caused_by)?
        .layout(&look.layout_params()))
}

fn namespace_boxes(
    look: &Graph,
    graph: &NamespaceGraph,
    layout: &Layout,
    owners: &[(Option<usize>, Option<&Layer>)],
) -> Vec<BoxOut> {
    let size = look.node_size();
    graph
        .namespaces
        .iter()
        .zip(&layout.placements)
        .zip(owners)
        .map(|((node, place), (_, layer))| BoxOut {
            name: node.name.to_string(),
            x: place.x,
            y: place.y,
            width: size.width,
            height: size.height,
            rank: place.rank,
            order: place.order,
            layer: layer.map(|found| found.name.as_str().to_owned()),
            types: Some(node.types),
            symbol: None,
        })
        .collect()
}

fn namespace_links(graph: &NamespaceGraph) -> Vec<LinkOut> {
    let name_of = |position: usize| {
        graph
            .namespaces
            .get(position)
            .map(|node| node.name.to_string())
    };
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            Some(LinkOut {
                from: name_of(edge.from)?,
                to: name_of(edge.to)?,
                count: edge.count,
                violates_layering: false,
                in_cycle: false,
            })
        })
        .collect()
}

fn namespaces_out(
    ports: &GraphPorts,
    layers: &[Layer],
    snapshot: SnapshotId,
    depth: u32,
) -> Result<GraphOut, CodedError> {
    let depth = usize::try_from(depth).unwrap_or_default();
    let graph = ports.index.namespace_graph(snapshot)?.collapsed(depth);
    let positions: Vec<Option<usize>> = graph
        .namespaces
        .iter()
        .map(|node| layer_index(layers, node.name.as_str()))
        .collect();
    let layout = namespace_layout(&ports.look, &graph, &positions)?;
    let owners: Vec<_> = positions
        .iter()
        .map(|position| (*position, position.and_then(|at| layers.get(at))))
        .collect();
    Ok(GraphOut {
        width: layout.width,
        height: layout.height,
        boxes: namespace_boxes(&ports.look, &graph, &layout, &owners),
        links: namespace_links(&graph),
        ..GraphOut::default()
    })
}

#[cfg(test)]
mod tests {
    use codeskimr_domain::LayerName;

    use super::*;

    #[test]
    fn a_dependency_keeps_its_counts_and_flags() {
        let edge = LayerEdge {
            from: LayerName::new("ui").unwrap(),
            to: LayerName::new("core").unwrap(),
            count: 7,
            violates_layering: true,
            in_cycle: false,
        };
        let link = link_out(&edge);
        assert_eq!((link.from.as_str(), link.to.as_str()), ("ui", "core"));
        assert_eq!(link.count, 7);
        assert!(link.violates_layering);
        assert!(!link.in_cycle);
    }
}
