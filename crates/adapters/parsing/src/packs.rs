//! The set of installed language packs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use codeskimr_domain::LanguageId;

use crate::consts::{ID_MISMATCH, MANIFEST_FILE};
use crate::error::ParsingError;
use crate::manifest::{PackManifest, Tier};
use crate::pack::LanguagePack;

/// What is known about a pack without loading its grammar and queries: enough to route files to
/// languages and to show the tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSummary {
    id: LanguageId,
    tier: Tier,
    extensions: Vec<String>,
}

impl PackSummary {
    /// Id of the language.
    #[must_use]
    pub const fn id(&self) -> &LanguageId {
        &self.id
    }

    /// Tier of the pack.
    #[must_use]
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// File extensions the pack claims, without the dot.
    #[must_use]
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

/// One installed pack. A full-tier pack is loaded with the set; a basic-tier pack is loaded when a
/// file of its language is first analyzed (blueprint 8.2).
struct Slot {
    summary: PackSummary,
    dir: PathBuf,
    pack: OnceLock<LanguagePack>,
}

impl Slot {
    fn read(dir: PathBuf) -> Result<Self, ParsingError> {
        let manifest = PackManifest::read(&dir)?;
        if manifest.id != folder(&dir) {
            let reason = [ID_MISMATCH, manifest.id.as_str()].concat();
            return Err(ParsingError::invalid(folder(&dir), reason));
        }
        let summary = PackSummary {
            id: LanguageId::new(manifest.id)
                .map_err(|e| ParsingError::invalid(folder(&dir), e.to_string()))?,
            tier: manifest.tier,
            extensions: manifest.extensions,
        };
        let slot = Self {
            summary,
            dir,
            pack: OnceLock::new(),
        };
        if slot.summary.tier == Tier::Full {
            slot.pack().map(drop)?;
        }
        Ok(slot)
    }

    fn pack(&self) -> Result<&LanguagePack, ParsingError> {
        if let Some(pack) = self.pack.get() {
            return Ok(pack);
        }
        let loaded = LanguagePack::load(&self.dir)?;
        Ok(self.pack.get_or_init(|| loaded))
    }
}

fn folder(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// All language packs found below one folder, by language id.
pub struct LanguagePacks {
    slots: HashMap<LanguageId, Slot>,
}

impl LanguagePacks {
    /// Reads every sub folder of `root` that holds a `language.toml`. Full-tier packs are loaded
    /// completely; of a basic-tier pack only the manifest is read until its language is needed.
    ///
    /// # Errors
    /// [`ParsingError::Unreadable`] if `root` cannot be listed, or the error of the first pack that
    /// is invalid.
    pub fn load(root: &Path) -> Result<Self, ParsingError> {
        let mut slots = HashMap::new();
        for entry in fs::read_dir(root).map_err(ParsingError::Unreadable)? {
            let dir = entry.map_err(ParsingError::Unreadable)?.path();
            if dir.join(MANIFEST_FILE).is_file() {
                let slot = Slot::read(dir)?;
                slots.insert(slot.summary.id.clone(), slot);
            }
        }
        Ok(Self { slots })
    }

    /// All installed packs, in no particular order. Does not load any grammar.
    pub fn iter(&self) -> impl Iterator<Item = &PackSummary> {
        self.slots.values().map(|slot| &slot.summary)
    }

    /// The pack for `language`; a basic-tier pack is loaded by the first call.
    ///
    /// # Errors
    /// [`ParsingError::PackMissing`] if none is installed, [`ParsingError::PackInvalid`] if a
    /// basic-tier pack turns out to be invalid when it is loaded.
    pub fn get(&self, language: &LanguageId) -> Result<&LanguagePack, ParsingError> {
        self.slots
            .get(language)
            .ok_or_else(|| ParsingError::PackMissing(language.to_string()))?
            .pack()
    }
}
