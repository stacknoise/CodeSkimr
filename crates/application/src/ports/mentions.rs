//! Finding the places where a name stands as text (adapter: `codeskimr_source`), for the
//! rename preview (ADR 0110).

use std::collections::HashSet;

use codeskimr_domain::{FolderPath, RelativePath};

use crate::error::CodedError;

/// How far the search for a name goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MentionLimits {
    /// Most mentions that are collected; the search goes on counting after that.
    pub max_items: usize,
    /// Largest file that is read, in bytes; a larger file is counted as skipped.
    pub max_file_bytes: u64,
}

/// One line of a file that holds the name as a whole word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    /// File that holds the line.
    pub path: RelativePath,
    /// The line, zero-based.
    pub line: u32,
    /// The line as written, without its line end and without the indentation.
    pub text: String,
}

/// The mentions of a name below a folder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mentions {
    /// The first mentions in the order of the paths and lines, at most `max_items`.
    pub items: Vec<Mention>,
    /// All lines that hold the name, also those that were cut.
    pub total: usize,
    /// Files that were not read: too large, not text, or not readable.
    pub skipped: usize,
}

/// Searches the text files of a project folder.
pub trait NameMentions: Send + Sync {
    /// The lines below `root` that hold `name` as a whole word (the characters next to it are not
    /// letters, digits or underscores), case-sensitive. The folder is walked as a scan walks it
    /// (ignore files and excluded folders count), but every text file is read, not only those of
    /// a language pack: configuration and documentation files are where a renamed name hides.
    /// Links are never followed, and a file that resolves outside `root` is skipped. A line whose
    /// path and zero-based number are in `except` is not a mention and is not counted: the caller
    /// knows it already.
    ///
    /// # Errors
    /// `SRC-201` if the folder cannot be opened.
    fn mentions(
        &self,
        root: &FolderPath,
        name: &str,
        limits: &MentionLimits,
        except: &HashSet<(String, u32)>,
    ) -> Result<Mentions, CodedError>;
}
