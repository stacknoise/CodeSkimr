//! The ports: traits that the driven adapters implement (blueprint section 4.1).
//!
//! Every port is synchronous and object safe, except `CredentialStore`: its loan method takes a
//! closure (blueprint 12.3), so adapters and the composition root use it through generics.
//! Adapters that do I/O or CPU work decide themselves how to run it (Tokio, Rayon); the
//! application layer only sees finished results.

mod analysis;
mod catalog;
mod clock;
mod consent;
mod credentials;
mod indexer;
mod level2;
mod outline;
mod query;
mod rescan;
mod similarity;
mod source;
mod storage;
mod tools;
mod usages;
mod version;

pub use analysis::{
    AnalyzeRequest, FileAnalyzer, FileFacts, MetricsCalculator, PendingReference, Receiver,
    ReferenceResolver, SnippetFingerprinter, SymbolFingerprints,
};
pub use catalog::{LanguageShare, ProjectCatalog, ProjectOverview};
pub use clock::Clock;
pub use consent::{ConsentGrant, ConsentRecord, ConsentStore, RunPermit};
pub use credentials::{AuthError, Credential, CredentialStore};
pub use indexer::{
    IndexArtifact, IndexJob, IndexLimits, IndexRunner, IndexerSpec, IndexerStatus,
    PLACEHOLDER_OUTPUT, PLACEHOLDER_PROJECT, ToolIssue, ToolProblem, ToolSpec,
};
pub use level2::{ImportRequest, Imported, ReferenceImporter};
pub use outline::{CodeOutline, MemberRow, OutlineNode};
pub use query::{
    CodeIndex, CodeSpan, FileHit, MemberRef, SearchHit, SymbolDetail, SymbolKey, TypeFocus,
    TypeNeighbour,
};
pub use rescan::ProjectScanner;
pub use similarity::{SimilarHit, SimilarQuery, SimilarityIndex};
pub use source::{
    DiscoveredFile, Excerpt, FetchedSource, FileDiscovery, SourceProvider, SourceText,
};
pub use storage::{
    PreviousIndex, ProjectRepository, ReferenceReplacement, SnapshotData, SnapshotStore,
};
pub use tools::{
    ProjectToolCheck, RequirementReader, ToolCatalog, ToolPlan, ToolReportStore, ToolRequirement,
};
pub use usages::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, ImpactedSymbol, SymbolLabel,
    UsageEntry, UsageIndex, UsagePage,
};
pub use version::ToolVersion;
