//! Mapping from file extensions to language packs.

use std::collections::HashMap;
use std::path::Path;

use codeskimr_domain::LanguageId;

/// Which language a file belongs to, decided by its extension.
///
/// The map is filled by the composition root from the installed language packs, so this crate
/// knows no language by name.
#[derive(Debug, Clone, Default)]
pub struct LanguageMap {
    by_extension: HashMap<String, LanguageId>,
}

impl LanguageMap {
    /// An empty map: no file is recognised.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `extension` (without the dot, case-insensitive) for `language`. A later call for
    /// the same extension replaces the earlier one.
    pub fn insert(&mut self, extension: &str, language: LanguageId) {
        self.by_extension.insert(extension.to_lowercase(), language);
    }

    /// The language of the file at `path`, if its extension is registered.
    #[must_use]
    pub fn detect(&self, path: &Path) -> Option<&LanguageId> {
        let extension = path.extension()?.to_str()?;
        self.by_extension.get(&extension.to_lowercase())
    }
}

impl<'a> FromIterator<(&'a str, LanguageId)> for LanguageMap {
    fn from_iter<I: IntoIterator<Item = (&'a str, LanguageId)>>(pairs: I) -> Self {
        let mut map = Self::new();
        for (extension, language) in pairs {
            map.insert(extension, language);
        }
        map
    }
}
