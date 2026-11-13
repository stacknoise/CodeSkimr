//! The ports: traits that the driven adapters implement (blueprint section 4.1).
//!
//! Every port is synchronous and object safe. Adapters that do I/O or CPU work decide themselves
//! how to run it (Tokio, Rayon); the application layer only sees finished results.

mod analysis;
mod clock;
mod source;
mod storage;

pub use analysis::{
    AnalyzeRequest, FileAnalyzer, FileFacts, MetricsCalculator, PendingReference, ReferenceResolver,
};
pub use clock::Clock;
pub use source::{DiscoveredFile, FetchedSource, FileDiscovery, SourceProvider};
pub use storage::{ProjectRepository, SnapshotData, SnapshotStore};
