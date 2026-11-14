//! Sources: file system, Git and file watching.
//!
//! - [`SourceFetcher`] implements `SourceProvider`: folders are opened, Git repositories are
//!   cloned shallowly into a cache folder.
//! - [`WalkDiscovery`] implements `FileDiscovery`: `.gitignore`-aware walk, language detection by
//!   extension, BLAKE3 content hashes, change detection against the previous snapshot.
//! - [`FolderText`] implements `SourceText`: lines of a file of a folder source, never outside it.
//! - [`ProjectRequirements`] implements `RequirementReader`: the versions a project names for
//!   the tool check of ADR 0033 (reads a few files, runs no project code).
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
mod redact;
mod requirement;
mod text;
mod watcher;

pub use discovery::{DiscoveryConfig, WalkDiscovery};
pub use error::SourceError;
pub use fetcher::SourceFetcher;
pub use git::{GitCache, Login};
pub use languages::LanguageMap;
pub use redact::redact_credentials;
pub use requirement::ProjectRequirements;
pub use text::FolderText;
pub use watcher::{WatchBatch, WatchConfig, WatchHandle, publish_changes, watch};
