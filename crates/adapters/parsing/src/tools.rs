//! The tools that the language packs ask for, read without loading any grammar.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use codeskimr_application::ports::ToolCatalog;
use codeskimr_domain::{LanguageId, ToolSpec};

use crate::consts::{MANIFEST_FILE, TOOL_INVALID};
use crate::error::ParsingError;
use crate::manifest::PackManifest;
use crate::pack::pack_id;

/// The `[[tools]]` of all language packs below one folder, by language.
#[derive(Debug, Default)]
pub struct PackTools {
    by_language: HashMap<LanguageId, Vec<ToolSpec>>,
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
            let (language, specs) = specs_of(&manifest, &folder)?;
            by_language.insert(language, specs);
        }
        Ok(Self { by_language })
    }
}

fn specs_of(
    manifest: &PackManifest,
    folder: &str,
) -> Result<(LanguageId, Vec<ToolSpec>), ParsingError> {
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let language = pack_id(&manifest.id, folder)?;
    let specs = manifest
        .tools
        .iter()
        .map(|tool| tool.to_spec(&language))
        .collect::<Result<Vec<_>, String>>()
        .map_err(|detail| invalid([TOOL_INVALID, detail.as_str()].concat()))?;
    Ok((language, specs))
}

impl ToolCatalog for PackTools {
    fn tools_of(&self, language: &LanguageId) -> Vec<ToolSpec> {
        self.by_language.get(language).cloned().unwrap_or_default()
    }
}
