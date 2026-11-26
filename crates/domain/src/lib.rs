//! Domain model, metrics and rules. Knows nothing about databases, file systems, networks or parsers.
//!
//! The entities follow blueprint section 6.1. IDs and validated values are newtypes (R-10),
//! so a `SymbolId` can never be passed where a `FileId` is expected.

mod access;
mod alert;
mod annotation;
mod consts;
mod coupling;
mod error;
mod file;
mod fingerprint;
mod graph_layout;
mod id_hasher;
mod layer;
mod layer_map;
mod measure;
mod metric;
mod metrics;
mod namespace_graph;
mod project;
mod qualified_name;
mod reference;
mod report;
mod reverse_index;
mod scope;
mod secret_url;
mod signature;
mod similar;
mod snapshot;
mod span;
mod statistics;
mod symbol;
mod tool;
mod type_neighbours;
mod vocabulary;

pub use access::{Permission, PermissionSet, Principal};
pub use alert::{
    Alert, AlertChange, AlertId, AlertKind, AlertSighting, AlertState, AlertTrigger, SymbolBaseline,
};
pub use annotation::{
    Annotation, AnnotationContent, AnnotationId, AnnotationKind, AnnotationLimits,
    AnnotationTarget, AnnotationText, AnnotationTitle, Assignee, Author, NewAnnotation, TaskStatus,
};
pub use coupling::{SymbolPair, coupling, coupling_with};
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
pub use namespace_graph::{
    NamespaceEdge, NamespaceGraph, NamespaceNode, name_parts, namespace_graph,
};
pub use project::{
    BranchName, CredentialId, FolderPath, GitUrl, Layer, LayerName, Project, ProjectId,
    ProjectName, ProjectSource,
};
pub use qualified_name::QualifiedName;
pub use reference::{Confidence, Reference, ReferenceId, ReferenceKind};
pub use report::{Level2Stage, OutcomeKind, ScanLevel, SkipReason};
pub use reverse_index::{Direction, Reached, ReverseIndex, Usage, UsageQuery};
pub use scope::{Scope, lies_below};
pub use secret_url::{without_secrets, without_userinfo};
pub use signature::signature_parameters;
pub use similar::{SimilarPair, similar_pairs};
pub use snapshot::{
    CommitHash, ScanMode, ScanOrigin, Snapshot, SnapshotId, SnapshotStats, Timestamp,
};
pub use span::{Position, Span};
pub use statistics::{
    Assessment, KindGroup, Measures, Ranked, Ranking, RankingQuery, SymbolFacts, UsageOrder,
    UsageTally, complexity_bars, complexity_histogram, mark_tests, rank,
};
pub use symbol::{Symbol, SymbolId, SymbolKind, SymbolName, SymbolOrigin, Visibility};
pub use tool::{
    Ceiling, RequirementFormat, RequirementSpec, RollForwardSpec, ToolEntry, ToolFinding,
    ToolReport, ToolRole, ToolState,
};
pub use type_neighbours::{TypeLink, TypeNeighbours, TypeOwners, TypeSymbol, type_neighbours};
