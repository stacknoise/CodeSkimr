//! The command behind the dependency map: the layers of the newest scan, placed by the graph
//! layout, with the aggregated dependencies between them. The layout is computed here, in Rust;
//! the UI only draws the result (blueprint 15.5).

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::Graph;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{LayerEdge, LayerMap, Permission, ProjectId, layer_map};
use serde::{Deserialize, Serialize};

use super::consts::{GRAPH_DESCRIPTION, GRAPH_LAYERS, GRAPH_TITLE};

/// The input of `graph.layers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct GraphInput {
    /// Id of the project.
    pub project: u64,
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
    /// The placed layers, in layout order of the project.
    pub boxes: Vec<BoxOut>,
    /// The dependencies between the layers.
    pub links: Vec<LinkOut>,
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
        let edges = ports.index.layer_edges(snapshot.id)?;
        let look = &ports.look;
        let map = layer_map(
            &project.layers,
            &edges,
            look.node_size(),
            &look.layout_params(),
        )
        .map_err(CodedError::caused_by)?;
        Ok(GraphOut {
            snapshot: Some(snapshot.id.get()),
            width: map.layout.width,
            height: map.layout.height,
            boxes: boxes_out(&map, look),
            links: edges.iter().map(link_out).collect(),
        })
    }
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
