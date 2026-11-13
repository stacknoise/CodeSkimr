//! Getting the code to scan (adapter: `codeskimr_source`).

use codeskimr_domain::{
    CommitHash, ContentHash, FolderPath, LanguageId, ProjectSource, RelativePath, SourceFile,
};

use crate::cancel::CancelToken;
use crate::error::CodedError;

/// The source after it was made available on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedSource {
    /// Local folder that holds the files.
    pub root: FolderPath,
    /// Commit that was checked out, if the source is a Git repository.
    pub commit: Option<CommitHash>,
}

/// A file found below the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFile {
    /// Path inside the project.
    pub path: RelativePath,
    /// Language of the file.
    pub language: LanguageId,
    /// Hash of the current content.
    pub content_hash: ContentHash,
    /// Whether the content differs from the previous snapshot, always true without one.
    pub changed: bool,
}

/// Makes a project source available as a local folder.
pub trait SourceProvider: Send + Sync {
    /// Fetches or opens `source`; a folder source needs no work.
    ///
    /// # Errors
    /// A coded failure such as `SRC-201`, `SRC-401` or `SRC-601`.
    fn fetch(
        &self,
        source: &ProjectSource,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, CodedError>;
}

/// Walks a folder, honours ignore rules, detects languages and hashes contents.
pub trait FileDiscovery: Send + Sync {
    /// Lists the files below `root` and marks those that differ from `previous`.
    ///
    /// # Errors
    /// A coded failure such as `SRC-201` or `SRC-501`.
    fn discover(
        &self,
        root: &FolderPath,
        previous: &[SourceFile],
        cancel: &CancelToken,
    ) -> Result<Vec<DiscoveredFile>, CodedError>;
}
