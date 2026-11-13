//! The set of installed language packs.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use codeskimr_domain::LanguageId;

use crate::consts::MANIFEST_FILE;
use crate::error::ParsingError;
use crate::pack::LanguagePack;

/// All language packs found below one folder, by language id.
pub struct LanguagePacks {
    packs: HashMap<LanguageId, LanguagePack>,
}

impl LanguagePacks {
    /// Loads every sub folder of `root` that holds a `language.toml`.
    ///
    /// # Errors
    /// [`ParsingError::Unreadable`] if `root` cannot be listed, or the error of the first pack that
    /// is invalid.
    pub fn load(root: &Path) -> Result<Self, ParsingError> {
        let mut packs = HashMap::new();
        for entry in fs::read_dir(root).map_err(ParsingError::Unreadable)? {
            let dir = entry.map_err(ParsingError::Unreadable)?.path();
            if dir.join(MANIFEST_FILE).is_file() {
                let pack = LanguagePack::load(&dir)?;
                packs.insert(pack.id().clone(), pack);
            }
        }
        Ok(Self { packs })
    }

    /// The pack for `language`.
    ///
    /// # Errors
    /// [`ParsingError::PackMissing`] if none is installed.
    pub fn get(&self, language: &LanguageId) -> Result<&LanguagePack, ParsingError> {
        self.packs
            .get(language)
            .ok_or_else(|| ParsingError::PackMissing(language.to_string()))
    }
}
