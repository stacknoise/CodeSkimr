//! Domain model, metrics and rules. Knows nothing about databases, file systems, networks or parsers.
//!
//! The entities follow blueprint section 6.1. IDs and validated values are newtypes (R-10),
//! so a `SymbolId` can never be passed where a `FileId` is expected.

mod access;
mod consts;
mod coupling;
mod error;
mod file;
mod fingerprint;
mod graph_layout;
mod layer;
mod layer_map;
mod measure;
mod metric;
mod metrics;
mod namespace_graph;
mod project;
mod qualified_name;
mod reference;
mod reverse_index;
mod secret_url;
mod similar;
mod snapshot;
mod span;
mod symbol;
mod tool;
mod type_neighbours;
mod vocabulary;

pub use access::{Permission, PermissionSet, Principal};
pub use coupling::coupling;
pub use error::ModelError;
pub use file::{ContentHash, FileId, LanguageId, Loc, RelativePath, SourceFile};
pub use fingerprint::{Fingerprint, FingerprintParams, Fingerprints, TokenId, winnow};
pub use graph_layout::{
    Layout, LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, NodeIndex, NodePlacement, NodeSize,
};
pub use layer::LayerEdge;
pub use layer_map::{LayerMap, layer_map};
pub use measure::{BodyCounts, SymbolCounts, measure};
pub use metric::{Metric, MetricKind, MetricValue};
pub use metrics::{
    HalsteadCounts, LineKind, cyclomatic_complexity, lines_of_code, maintainability_index,
};
pub use namespace_graph::{NamespaceEdge, NamespaceGraph, NamespaceNode, namespace_graph};
pub use project::{
    BranchName, CredentialId, FolderPath, GitUrl, Layer, LayerName, Project, ProjectId,
    ProjectName, ProjectSource,
};
pub use qualified_name::QualifiedName;
pub use reference::{Confidence, Reference, ReferenceId, ReferenceKind};
pub use reverse_index::{Direction, Reached, ReverseIndex, Usage, UsageQuery};
pub use secret_url::{without_secrets, without_userinfo};
pub use similar::{SimilarPair, similar_pairs};
pub use snapshot::{CommitHash, ScanMode, Snapshot, SnapshotId, SnapshotStats, Timestamp};
pub use span::{Position, Span};
pub use symbol::{Symbol, SymbolId, SymbolKind, SymbolName, Visibility};
pub use tool::{
    RequirementFormat, RequirementSpec, ToolEntry, ToolFinding, ToolReport, ToolRole, ToolState,
};
pub use type_neighbours::{TypeLink, TypeNeighbours, type_neighbours};
