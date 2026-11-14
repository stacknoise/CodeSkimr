//! The set of installed language packs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;

use codeskimr_domain::LanguageId;

use crate::consts::{ID_MISMATCH, LOAD_PANICKED, MANIFEST_FILE};
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

/// The folders below `root` that hold a `language.toml`, in order. `LanguagePacks::load` and
/// `tiers` use this one search, so the tier shown is that of the pack the scan loads.
fn pack_dirs(root: &Path) -> Result<Vec<PathBuf>, ParsingError> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root).map_err(ParsingError::Unreadable)? {
        let dir = entry.map_err(ParsingError::Unreadable)?.path();
        if dir.join(MANIFEST_FILE).is_file() {
            dirs.push(dir);
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// The tier of every pack below `root`, read from the manifests alone: no grammar and no query is
/// compiled, so this is cheap enough for a screen that asks often. A pack whose manifest cannot
/// be read is left out; the scan reports it when it loads the packs.
///
/// # Errors
/// [`ParsingError::Unreadable`] if `root` cannot be listed.
pub fn tiers(root: &Path) -> Result<Vec<(String, Tier)>, ParsingError> {
    Ok(pack_dirs(root)?
        .iter()
        .filter_map(|dir| PackManifest::read(dir).ok())
        .map(|manifest| (manifest.id, manifest.tier))
        .collect())
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
        let dirs = pack_dirs(root)?;
        // Compiling the queries of a full-tier pack takes long, so the packs are read in parallel;
        // the error reported is the one of the first pack in the order of the folders.
        let read: Vec<Result<Slot, ParsingError>> = thread::scope(|scope| {
            let handles: Vec<_> = dirs
                .into_iter()
                .map(|dir| scope.spawn(|| Slot::read(dir)))
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle.join().unwrap_or_else(|_| {
                        Err(ParsingError::invalid(
                            String::new(),
                            LOAD_PANICKED.to_owned(),
                        ))
                    })
                })
                .collect()
        });
        let mut slots = HashMap::new();
        for slot in read {
            let slot = slot?;
            slots.insert(slot.summary.id.clone(), slot);
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
