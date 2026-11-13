//! Layered graph layout (Sugiyama style): nodes and edges in, positions out.
//!
//! The layout runs in four steps: ranks (rows), long edges split into one-row steps, median
//! sweeps against crossings, coordinates. It needs no dependency, is deterministic (same input,
//! same output) and runs in near-linear time per sweep, which keeps 200 nodes far below the
//! 50 ms budget of blueprint 16.1.
//!
//! Edges are not routed here: the picture draws them as straight lines or curves between the
//! nodes. Crossing counts and edge waypoints can be added without changing this interface.

mod consts;
mod layers;
mod order;
mod place;
mod rank;
mod slots;

use crate::error::ModelError;

/// Index of a node in the node list of a [`LayoutGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(usize);

impl NodeIndex {
    /// Wraps a position in the node list; [`LayoutGraph::new`] checks that it exists.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The position in the node list.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Size of a node box in layout units (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeSize {
    /// Horizontal extent.
    pub width: u32,
    /// Vertical extent.
    pub height: u32,
}

/// A node to place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutNode {
    /// Box size.
    pub size: NodeSize,
    /// Row the node must be in (for example its layer band, outer layers first); `None` lets the
    /// layout derive the row from the edges.
    pub rank: Option<u32>,
}

/// A directed edge between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutEdge {
    /// Source node.
    pub from: NodeIndex,
    /// Target node.
    pub to: NodeIndex,
}

/// Tunable spacing and effort of the layout; the values come from the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutParams {
    /// Vertical space between two rows.
    pub rank_gap: u32,
    /// Horizontal space between two neighbours in a row.
    pub node_gap: u32,
    /// Maximum number of crossing-reduction sweeps.
    pub sweeps: u32,
    /// Number of coordinate refinement passes.
    pub placement_passes: u32,
}

/// Where a node ended up. `x` and `y` are the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodePlacement {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Row index, starting at 0 at the top.
    pub rank: u32,
    /// Position inside the row, starting at 0 on the left.
    pub order: u32,
}

/// Result of a layout: one placement per node, in node order, and the size of the picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Placement of every node, indexed like the input nodes.
    pub placements: Vec<NodePlacement>,
    /// Width of the whole picture.
    pub width: u32,
    /// Height of the whole picture.
    pub height: u32,
}

/// A validated graph ready for layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutGraph {
    nodes: Vec<LayoutNode>,
    edges: Vec<LayoutEdge>,
}

impl LayoutGraph {
    /// Creates a graph.
    ///
    /// # Errors
    /// [`ModelError::UnknownNode`] when an edge names a node that is not in the list.
    pub fn new(nodes: Vec<LayoutNode>, edges: Vec<LayoutEdge>) -> Result<Self, ModelError> {
        let known = |index: NodeIndex| index.get() < nodes.len();
        if edges.iter().all(|edge| known(edge.from) && known(edge.to)) {
            Ok(Self { nodes, edges })
        } else {
            Err(ModelError::UnknownNode)
        }
    }

    /// The nodes, in input order.
    #[must_use]
    pub fn nodes(&self) -> &[LayoutNode] {
        &self.nodes
    }

    /// The edges, in input order.
    #[must_use]
    pub fn edges(&self) -> &[LayoutEdge] {
        &self.edges
    }

    /// Computes positions for all nodes. Never fails: cycles are tolerated and an empty graph
    /// gives an empty layout.
    #[must_use]
    pub fn layout(&self, params: &LayoutParams) -> Layout {
        let ranks = rank::assign(self);
        let mut layered = layers::build(&ranks, &self.edges);
        order::minimize(&mut layered, params.sweeps);
        let sizes: Vec<NodeSize> = self.nodes.iter().map(|node| node.size).collect();
        let geometry = place::place(&layered, &sizes, params);
        let position = layered.positions();
        let placements = (0..self.nodes.len())
            .map(|node| NodePlacement {
                x: pixels(slots::value(&geometry.x, node)),
                y: pixels(slots::value(&geometry.y, node)),
                rank: count(slots::value(&layered.rank, node)),
                order: count(slots::value(&position, node)),
            })
            .collect();
        Layout {
            placements,
            width: pixels(geometry.width),
            height: pixels(geometry.height),
        }
    }
}

/// Layout units are never negative; anything out of range saturates.
fn pixels(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
