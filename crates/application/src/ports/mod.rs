//! The ports: traits that the driven adapters implement (blueprint section 4.1).
//!
//! Every port is synchronous and object safe, except `CredentialStore`: its loan method takes a
//! closure (blueprint 12.3), so adapters and the composition root use it through generics.
//! Adapters that do I/O or CPU work decide themselves how to run it (Tokio, Rayon); the
//! application layer only sees finished results.

mod analysis;
mod annotations;
mod catalog;
mod clock;
mod compare;
mod confirmation;
mod consent;
mod credentials;
mod digest;
mod highlight;
mod indexer;
mod keys;
mod level2;
mod memory;
mod mentions;
mod outline;
mod query;
mod references;
mod report;
mod rescan;
mod scope;
mod similarity;
mod source;
mod states;
mod statistics;
mod storage;
mod tools;
mod usages;
mod version;

pub use crate::consts::FOLDER_NAMESPACE_LANGUAGES;
pub use analysis::{
    AnalyzeRequest, FileAnalyzer, FileFacts, MetricsCalculator, PendingReference, Receiver,
    ReferenceResolver, Resolution, SnippetFingerprinter, SymbolFingerprints, SymbolHash,
};
pub use annotations::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnnotationCreation, AnnotationFilter,
    AnnotationStore, AnnotationUpdate,
};
pub use catalog::{DatabaseInfo, DatabaseState, LanguageShare, ProjectCatalog, ProjectOverview};
pub use clock::Clock;
pub use compare::ComparisonSource;
pub use confirmation::{ConfirmationWindow, Level2Switch, WindowOutcome};
pub use consent::{ConsentGrant, ConsentRecord, ConsentStore, RunPermit};
pub use credentials::{AuthError, Credential, CredentialStore, FIELD_SEPARATOR, LockedSecret};
pub use digest::ContentDigest;
pub use highlight::{SyntaxHighlighter, TokenClass, TokenSpan};
pub use indexer::{
    IndexArtifact, IndexJob, IndexLimits, IndexRunner, IndexerSpec, IndexerStatus,
    PLACEHOLDER_OUTPUT, PLACEHOLDER_PROJECT, ToolIssue, ToolProblem, ToolSeen, ToolSpec,
    matches_project_file,
};
pub use keys::{KeyEntry, KeyKind, KeySearch, ProjectKeys};
pub use level2::{ImportRequest, Imported, ReferenceImporter};
pub use memory::{MemoryGauge, MemoryLimit, MemoryWatch};
pub use mentions::{Mention, MentionLimits, Mentions, NameMentions};
pub use outline::{CodeOutline, MemberRow, OutlineNode};
pub use query::{
    CodeIndex, CodeSpan, FileHit, MemberRef, ScanEntry, SearchHit, StepTime, SymbolDetail,
    SymbolKey, TypeFocus, TypeNeighbour,
};
pub use references::{ReferencePlace, ReferencePlaces, ReferenceSite, ReferenceSites};
pub use report::{
    LanguageLine, LanguageOutcome, LastScan, Level2Report, ReportWrite, ScanReport, ScanReportStore,
};
pub use rescan::{LeftOut, PathsReport, ProjectScanner, Rescanned, SkippedFile};
pub use scope::{FolderLister, ListedFile, ScopeStore};
pub use similarity::{SimilarHit, SimilarQuery, SimilarityIndex};
pub use source::{
    DiscoveredFile, Excerpt, FetchedSource, FileDiscovery, SourceProvider, SourceText,
};
pub use states::{SameName, SymbolState, SymbolStates};
pub use statistics::{
    CodeStatistics, RankedRows, RankingRequest, StatisticsSetup, StatisticsSummary, UnusedCounts,
};
pub use storage::{
    Confirmation, Confirmed, Level2Replacement, PreviousIndex, PreviousState, ProjectRemoval,
    ProjectRepository, ScanWarning, SnapshotData, SnapshotStore, StoredNames, StoredReferences,
};
pub use tools::{
    ProjectToolCheck, RequirementReader, RequirementValue, ToolCatalog, ToolPlan, ToolReportStore,
    ToolRequirement,
};
pub use usages::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, ImpactedSymbol, SymbolLabel,
    UsageEntry, UsageIndex, UsagePage,
};
pub use version::ToolVersion;
