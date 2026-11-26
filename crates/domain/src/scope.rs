//! What a project covers: the languages and folders that its scans leave out (ADR 0101).
//!
//! The scope lists what is left out, not what is taken, so a language or folder that appears later
//! is covered without anybody changing the setting. An empty scope covers everything, which is what
//! every project had before the scope existed.

use std::collections::BTreeSet;

use crate::consts::PATH_SEPARATOR;
use crate::file::{LanguageId, RelativePath};

/// The languages and folders a project leaves out of its scans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    languages: BTreeSet<LanguageId>,
    folders: BTreeSet<RelativePath>,
}

impl Scope {
    /// A scope that leaves out `languages` and everything below `folders`. A folder below another
    /// listed folder is dropped, because the outer one already covers it.
    #[must_use]
    pub fn new(
        languages: impl IntoIterator<Item = LanguageId>,
        folders: impl IntoIterator<Item = RelativePath>,
    ) -> Self {
        let folders: BTreeSet<RelativePath> = folders.into_iter().collect();
        let outer: BTreeSet<RelativePath> = folders
            .iter()
            .filter(|folder| !folders.iter().any(|other| lies_below(folder, other)))
            .cloned()
            .collect();
        Self {
            languages: languages.into_iter().collect(),
            folders: outer,
        }
    }

    /// The languages that are left out.
    #[must_use]
    pub const fn languages(&self) -> &BTreeSet<LanguageId> {
        &self.languages
    }

    /// The folders that are left out, each with everything below it.
    #[must_use]
    pub const fn folders(&self) -> &BTreeSet<RelativePath> {
        &self.folders
    }

    /// Whether nothing is left out.
    #[must_use]
    pub fn is_everything(&self) -> bool {
        self.languages.is_empty() && self.folders.is_empty()
    }

    /// Whether the scope covers the language.
    #[must_use]
    pub fn covers_language(&self, language: &LanguageId) -> bool {
        !self.languages.contains(language)
    }

    /// Whether the scope covers the folder, which is not so if it or a folder above it is left out.
    /// The project folder itself (`None`) is always covered.
    #[must_use]
    pub fn covers_folder(&self, folder: &RelativePath) -> bool {
        !self
            .folders
            .iter()
            .any(|left_out| left_out == folder || lies_below(folder, left_out))
    }

    /// Whether a file of `language` at `path` is covered: its language is, and so is every folder
    /// above it.
    #[must_use]
    pub fn covers(&self, path: &RelativePath, language: &LanguageId) -> bool {
        self.covers_language(language)
            && !self
                .folders
                .iter()
                .any(|left_out| lies_below(path, left_out))
    }
}

/// Whether `path` lies strictly below `folder`: it starts with the folder and a separator.
#[must_use]
pub fn lies_below(path: &RelativePath, folder: &RelativePath) -> bool {
    path.as_str()
        .strip_prefix(folder.as_str())
        .is_some_and(|rest| rest.starts_with(PATH_SEPARATOR))
}
