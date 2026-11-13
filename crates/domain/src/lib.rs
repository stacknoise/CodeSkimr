//! Domain model, metrics and rules. Knows nothing about databases, file systems, networks or parsers.
//!
//! The entities follow blueprint section 6.1. IDs and validated values are newtypes (R-10),
//! so a `SymbolId` can never be passed where a `FileId` is expected.

mod consts;
mod error;
mod file;
mod fingerprint;
mod graph_layout;
mod layer;
mod measure;
mod metric;
mod metrics;
mod project;
mod qualified_name;
mod reference;
mod secret_url;
mod snapshot;
mod span;
mod symbol;
mod vocabulary;

pub use error::ModelError;
pub use file::{ContentHash, FileId, LanguageId, Loc, RelativePath, SourceFile};
pub use fingerprint::{Fingerprint, FingerprintParams, Fingerprints, TokenId, winnow};
pub use graph_layout::{
    Layout, LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, NodeIndex, NodePlacement, NodeSize,
};
pub use layer::LayerEdge;
pub use measure::{BodyCounts, SymbolCounts, measure};
pub use metric::{Metric, MetricKind, MetricValue};
pub use metrics::{
    HalsteadCounts, LineKind, cyclomatic_complexity, lines_of_code, maintainability_index,
};
pub use project::{
    BranchName, CredentialId, FolderPath, GitUrl, Layer, LayerName, Project, ProjectId,
    ProjectName, ProjectSource,
};
pub use qualified_name::QualifiedName;
pub use reference::{Confidence, Reference, ReferenceId, ReferenceKind};
pub use secret_url::{without_secrets, without_userinfo};
pub use snapshot::{CommitHash, ScanMode, Snapshot, SnapshotId, SnapshotStats, Timestamp};
pub use span::{Position, Span};
pub use symbol::{Symbol, SymbolId, SymbolKind, SymbolName, Visibility};
