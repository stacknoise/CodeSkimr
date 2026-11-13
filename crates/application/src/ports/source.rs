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
    /// Fetches or opens `source`; a folder source needs no work. If `cancel` is raised the adapter
    /// stops and may return any error; the pipeline then reports a cancellation, not a failure.
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
    /// Lists the files below `root` and marks those that differ from `previous`. Cancellation
    /// works as for `SourceProvider::fetch`.
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

/// A run of lines of a file, as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Excerpt {
    /// The lines without their line ends.
    pub lines: Vec<String>,
    /// Whether the run was cut at the limit and the declaration goes on.
    pub truncated: bool,
}

/// Reads lines of a file of a project as it is now. Whoever calls this hands out source text and
/// needs `Permission::ReadSource` (blueprint 5.2).
pub trait SourceText: Send + Sync {
    /// The lines `range` (first and last, counted from 1, both included) of `path` below `root`.
    /// `limits` are the most lines to return and the most bytes the file may have.
    ///
    /// The path must stay inside `root`: a path that leaves it, also through a link, is treated
    /// as not found.
    ///
    /// # Errors
    /// `SRC-201` if the file is missing, not a regular file or outside the root, `SRC-501` if it
    /// is larger than the byte limit.
    fn excerpt(
        &self,
        root: &FolderPath,
        path: &RelativePath,
        range: (u32, u32),
        limits: (u32, u64),
    ) -> Result<Excerpt, CodedError>;
}
