//! Sources: file system, Git and file watching.
//!
//! - [`SourceFetcher`] implements `SourceProvider`: folders are opened, Git repositories are
//!   cloned shallowly into a cache folder.
//! - [`WalkDiscovery`] implements `FileDiscovery`: `.gitignore`-aware walk, language detection by
//!   extension, BLAKE3 content hashes, change detection against the previous snapshot.
//! - [`FolderText`] implements `SourceText`: lines of a file of a folder source, never outside it.
//! - [`ProcessProbe`] implements `ToolProbe` and [`ProjectRequirements`] implements
//!   `RequirementReader`: the tool check of ADR 0033 (version queries only, no project code).
//! - [`watch`] reports debounced file changes; [`publish_changes`] turns them into `FilesChanged`
//!   events on the event bus.

mod consts;
mod discovery;
mod error;
mod fetcher;
mod folder;
mod git;
mod hash;
mod languages;
mod paths;
mod probe;
mod redact;
mod requirement;
mod text;
mod watcher;

pub use discovery::{DiscoveryConfig, WalkDiscovery};
pub use error::SourceError;
pub use fetcher::SourceFetcher;
pub use git::{GitCache, Login};
pub use languages::LanguageMap;
pub use probe::{ProbeSettings, ProcessProbe};
pub use redact::redact_credentials;
pub use requirement::ProjectRequirements;
pub use text::FolderText;
pub use watcher::{WatchBatch, WatchConfig, WatchHandle, publish_changes, watch};
