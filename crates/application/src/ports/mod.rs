//! The ports: traits that the driven adapters implement (blueprint section 4.1).
//!
//! Every port is synchronous and object safe, except `CredentialStore`: its loan method takes a
//! closure (blueprint 12.3), so adapters and the composition root use it through generics.
//! Adapters that do I/O or CPU work decide themselves how to run it (Tokio, Rayon); the
//! application layer only sees finished results.

mod analysis;
mod catalog;
mod clock;
mod credentials;
mod query;
mod rescan;
mod similarity;
mod source;
mod storage;
mod usages;

pub use analysis::{
    AnalyzeRequest, FileAnalyzer, FileFacts, MetricsCalculator, PendingReference, Receiver,
    ReferenceResolver, SnippetFingerprinter, SymbolFingerprints,
};
pub use catalog::{LanguageShare, ProjectCatalog, ProjectOverview};
pub use clock::Clock;
pub use credentials::{AuthError, Credential, CredentialStore};
pub use query::{CodeIndex, MemberRef, SearchHit, SymbolDetail, SymbolKey};
pub use rescan::ProjectScanner;
pub use similarity::{SimilarHit, SimilarQuery, SimilarityIndex};
pub use source::{DiscoveredFile, FetchedSource, FileDiscovery, SourceProvider};
pub use storage::{PreviousIndex, ProjectRepository, SnapshotData, SnapshotStore};
pub use usages::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, ImpactedSymbol, SymbolLabel,
    UsageEntry, UsageIndex, UsagePage,
};
