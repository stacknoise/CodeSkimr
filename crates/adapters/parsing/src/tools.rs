//! The tools that the language packs ask for, read without loading any grammar.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use codeskimr_application::ports::{ToolCatalog, ToolPlan};
use codeskimr_domain::LanguageId;

use crate::consts::{MANIFEST_FILE, TOOL_INVALID};
use crate::error::ParsingError;
use crate::manifest::PackManifest;
use crate::pack::pack_id;

/// The `[indexer]` of all language packs below one folder, by language.
#[derive(Debug, Default)]
pub struct PackTools {
    by_language: HashMap<LanguageId, ToolPlan>,
}

impl PackTools {
    /// Reads the manifest of every sub folder of `root` that has one. Grammars and queries are
    /// not touched, so this is cheap.
    ///
    /// # Errors
    /// [`ParsingError::Unreadable`] if `root` cannot be listed, [`ParsingError::PackInvalid`] for
    /// the first manifest that is malformed or names a tool wrongly.
    pub fn load(root: &Path) -> Result<Self, ParsingError> {
        let mut by_language = HashMap::new();
        for entry in fs::read_dir(root).map_err(ParsingError::Unreadable)? {
            let dir = entry.map_err(ParsingError::Unreadable)?.path();
            if !dir.join(MANIFEST_FILE).is_file() {
                continue;
            }
            let manifest = PackManifest::read(&dir)?;
            let folder = dir
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if let Some((language, plan)) = plan_of(&manifest, &folder)? {
                by_language.insert(language, plan);
            }
        }
        Ok(Self { by_language })
    }
}

fn plan_of(
    manifest: &PackManifest,
    folder: &str,
) -> Result<Option<(LanguageId, ToolPlan)>, ParsingError> {
    let invalid =
        |reason: String| ParsingError::invalid(folder, [TOOL_INVALID, reason.as_str()].concat());
    let Some(indexer) = &manifest.indexer else {
        return Ok(None);
    };
    let language = pack_id(&manifest.id, folder)?;
    let plan = ToolPlan {
        indexer: indexer.spec().map_err(invalid)?,
        requirements: indexer.requirements().map_err(invalid)?,
    };
    Ok(Some((language, plan)))
}

impl ToolCatalog for PackTools {
    fn plan_of(&self, language: &LanguageId) -> Option<ToolPlan> {
        self.by_language.get(language).cloned()
    }
}
