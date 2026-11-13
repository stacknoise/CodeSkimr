//! Sources: file system, Git and file watching.
//!
//! - [`SourceFetcher`] implements `SourceProvider`: folders are opened, Git repositories are
//!   cloned shallowly into a cache folder.
//! - [`WalkDiscovery`] implements `FileDiscovery`: `.gitignore`-aware walk, language detection by
//!   extension, BLAKE3 content hashes, change detection against the previous snapshot.
//! - [`watch`] reports debounced file changes.

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
mod watcher;

pub use discovery::{DiscoveryConfig, WalkDiscovery};
pub use error::SourceError;
pub use fetcher::SourceFetcher;
pub use git::GitCache;
pub use languages::LanguageMap;
pub use redact::redact_credentials;
pub use watcher::{WatchConfig, WatchHandle, watch};
