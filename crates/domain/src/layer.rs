//! Aggregated dependencies between layers.

use crate::project::LayerName;

/// Aggregated dependency between two namespaces or layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerEdge {
    /// Depending side.
    pub from: LayerName,
    /// Depended-upon side.
    pub to: LayerName,
    /// Number of references behind the edge.
    pub count: u32,
    /// Whether the edge breaks the declared layering.
    pub violates_layering: bool,
    /// Whether the edge is part of a dependency cycle.
    pub in_cycle: bool,
}
