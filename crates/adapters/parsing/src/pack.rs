//! A loaded language pack: grammar, rules and compiled queries.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::str::FromStr;

use codeskimr_domain::{LanguageId, Visibility};
use tree_sitter::{Language, Query};

use crate::captures::Vocabulary;
use crate::consts::{MANIFEST_FILE, REFERENCES_QUERY_FILE, SYMBOLS_QUERY_FILE};
use crate::error::ParsingError;
use crate::grammar;
use crate::manifest::{Manifest, Tier};

/// Everything the analyzer needs to understand one language.
pub struct LanguagePack {
    pub(crate) id: LanguageId,
    pub(crate) tier: Tier,
    pub(crate) extensions: Vec<String>,
    project_files: Vec<String>,
    test_patterns: Vec<String>,
    pub(crate) language: Language,
    pub(crate) separator: String,
    pub(crate) comment_nodes: HashSet<String>,
    pub(crate) doc_prefixes: Vec<String>,
    pub(crate) default_visibility: Option<Visibility>,
    pub(crate) visibility: HashMap<String, Visibility>,
    pub(crate) symbols: Vocabulary,
    pub(crate) references: Option<Vocabulary>,
}

impl LanguagePack {
    /// Loads the pack in `dir`: `language.toml`, `symbols.scm` and the optional `references.scm`.
    ///
    /// # Errors
    /// [`ParsingError::PackInvalid`] if a file is missing or malformed, the grammar is unknown, a
    /// query does not compile or uses a capture outside the vocabulary.
    pub fn load(dir: &Path) -> Result<Self, ParsingError> {
        let folder = folder_name(dir);
        let invalid = |reason: String| ParsingError::invalid(folder.clone(), reason);
        let text =
            fs::read_to_string(dir.join(MANIFEST_FILE)).map_err(|e| invalid(e.to_string()))?;
        let manifest: Manifest = toml::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        let language =
            grammar::builtin(&manifest.grammar).ok_or_else(|| invalid(manifest.grammar.clone()))?;
        let symbols = compile(&language, &dir.join(SYMBOLS_QUERY_FILE), &folder)?;
        let references_path = dir.join(REFERENCES_QUERY_FILE);
        let references = if references_path.is_file() {
            Some(compile(&language, &references_path, &folder)?)
        } else {
            None
        };
        Self::assemble(manifest, language, symbols, references, &folder)
    }

    fn assemble(
        manifest: Manifest,
        language: Language,
        symbols: Vocabulary,
        references: Option<Vocabulary>,
        folder: &str,
    ) -> Result<Self, ParsingError> {
        let invalid = |reason: String| ParsingError::invalid(folder, reason);
        if manifest.id != folder {
            return Err(invalid(manifest.id));
        }
        let id = LanguageId::new(manifest.id).map_err(|e| invalid(e.to_string()))?;
        let mut visibility = HashMap::new();
        for (keyword, label) in manifest.visibility.keywords {
            visibility.insert(
                keyword,
                Visibility::from_str(&label).map_err(|e| invalid(e.to_string()))?,
            );
        }
        let default_visibility = manifest
            .visibility
            .default
            .map(|label| Visibility::from_str(&label).map_err(|e| invalid(e.to_string())))
            .transpose()?;
        Ok(Self {
            id,
            tier: manifest.tier,
            extensions: manifest.extensions,
            project_files: manifest.project_files,
            test_patterns: manifest.test_patterns,
            language,
            separator: manifest.separator,
            comment_nodes: manifest.comment_nodes.into_iter().collect(),
            doc_prefixes: manifest.doc_prefixes,
            default_visibility,
            visibility,
            symbols,
            references,
        })
    }

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

    /// Project manifest files of the language, such as `Cargo.toml`.
    #[must_use]
    pub fn project_files(&self) -> &[String] {
        &self.project_files
    }

    /// Glob patterns that match test files.
    #[must_use]
    pub fn test_patterns(&self) -> &[String] {
        &self.test_patterns
    }

    /// File extensions the pack claims, without the dot.
    #[must_use]
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn compile(language: &Language, path: &Path, folder: &str) -> Result<Vocabulary, ParsingError> {
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let source = fs::read_to_string(path).map_err(|e| invalid(e.to_string()))?;
    let query = Query::new(language, &source).map_err(|e| invalid(e.to_string()))?;
    Vocabulary::new(query).map_err(invalid)
}
