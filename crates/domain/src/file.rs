//! Source files of a snapshot.

use crate::consts::PATH_SEPARATOR;
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

impl RelativePath {
    /// This path below `folder` (`a.rs` in `cli` is `cli/a.rs`); `None` if the joined text would
    /// be empty, which two non-empty paths never give.
    #[must_use]
    pub fn in_folder(&self, folder: &Self) -> Option<Self> {
        let mut joined = folder.as_str().to_owned();
        joined.push(PATH_SEPARATOR);
        joined.push_str(self.as_str());
        Self::new(joined).ok()
    }

    /// Whether this path lies below `folder` (`cli/a.rs` is below `cli`, `client/a.rs` is not).
    #[must_use]
    pub fn is_below(&self, folder: &Self) -> bool {
        self.as_str()
            .strip_prefix(folder.as_str())
            .is_some_and(|rest| rest.starts_with(PATH_SEPARATOR))
    }
}

/// Lines of code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Loc(pub u32);

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
