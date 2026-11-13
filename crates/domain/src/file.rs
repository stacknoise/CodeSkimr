//! Source files of a snapshot.

use crate::metric::Loc;
use crate::snapshot::SnapshotId;
use crate::vocabulary::{id_type, text_type};

id_type!(
    /// Identifies a file within a snapshot.
    FileId
);
text_type!(
    /// Path of a file relative to the project root, with forward slashes.
    RelativePath
);
text_type!(
    /// Id of a language pack, such as the folder name under `languages/`.
    LanguageId
);
text_type!(
    /// Hash of the file content, used to skip unchanged files.
    ContentHash
);

/// A source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Identity.
    pub id: FileId,
    /// Owning snapshot.
    pub snapshot: SnapshotId,
    /// Path inside the project.
    pub path: RelativePath,
    /// Language of the file.
    pub language: LanguageId,
    /// Hash of the content.
    pub content_hash: ContentHash,
    /// Lines of code.
    pub loc: Loc,
}
