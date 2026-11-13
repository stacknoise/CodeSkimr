//! A loaded language pack: grammar, rules and compiled queries.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::str::FromStr;

use codeskimr_domain::{LanguageId, ModelError, SymbolKind};
use tree_sitter::{Language, Query};

use crate::captures::{MetricsQuery, Vocabulary};
use crate::consts::{ID_MISMATCH, METRICS_QUERY_FILE, REFERENCES_QUERY_FILE, SYMBOLS_QUERY_FILE};
use crate::error::ParsingError;
use crate::grammar;
use crate::manifest::{DocSummary, PackManifest, Tier};
use crate::visibility::Visibilities;

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
    pub(crate) static_keywords: HashSet<String>,
    pub(crate) modifier_kinds: HashMap<String, SymbolKind>,
    pub(crate) doc_summary: Option<DocSummary>,
    pub(crate) identifier_nodes: HashSet<String>,
    pub(crate) literal_nodes: HashSet<String>,
    pub(crate) visibility: Visibilities,
    pub(crate) symbols: Vocabulary,
    pub(crate) references: Option<Vocabulary>,
    pub(crate) metrics: Option<MetricsQuery>,
}

impl LanguagePack {
    /// Loads the pack in `dir`: `language.toml`, `symbols.scm` and the optional `references.scm` and
    /// `metrics.scm`.
    ///
    /// # Errors
    /// [`ParsingError::PackInvalid`] if a file is missing or malformed, the grammar is unknown, a
    /// query does not compile or uses a capture outside the vocabulary.
    pub fn load(dir: &Path) -> Result<Self, ParsingError> {
        let folder = folder_name(dir);
        let invalid = |reason: String| ParsingError::invalid(folder.clone(), reason);
        let manifest = PackManifest::read(dir)?;
        let language =
            grammar::builtin(&manifest.grammar).ok_or_else(|| invalid(manifest.grammar.clone()))?;
        let symbols = compile(&language, &dir.join(SYMBOLS_QUERY_FILE), &folder)?;
        let references_path = dir.join(REFERENCES_QUERY_FILE);
        let references = if references_path.is_file() {
            Some(compile(&language, &references_path, &folder)?)
        } else {
            None
        };
        let metrics = load_metrics(&language, &dir.join(METRICS_QUERY_FILE), &folder)?;
        Self::assemble(manifest, language, (symbols, references, metrics), &folder)
    }

    fn assemble(
        manifest: PackManifest,
        language: Language,
        (symbols, references, metrics): (Vocabulary, Option<Vocabulary>, Option<MetricsQuery>),
        folder: &str,
    ) -> Result<Self, ParsingError> {
        let invalid = |reason: String| ParsingError::invalid(folder, reason);
        if manifest.id != folder {
            return Err(invalid([ID_MISMATCH, manifest.id.as_str()].concat()));
        }
        let id = LanguageId::new(manifest.id).map_err(|e| invalid(e.to_string()))?;
        let visibility = Visibilities::resolve(manifest.visibility).map_err(invalid)?;
        let modifier_kinds = manifest
            .modifier_kinds
            .into_iter()
            .map(|(keyword, label)| Ok((keyword, SymbolKind::from_str(&label)?)))
            .collect::<Result<_, ModelError>>()
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            id,
            tier: manifest.tier,
            extensions: manifest.extensions,
            project_files: manifest.manifest_files,
            test_patterns: manifest.test_patterns,
            language,
            separator: manifest.separator,
            comment_nodes: manifest.comment_nodes.into_iter().collect(),
            doc_prefixes: manifest.doc_prefixes,
            static_keywords: manifest.static_keywords.into_iter().collect(),
            modifier_kinds,
            doc_summary: manifest.doc_summary,
            identifier_nodes: manifest.identifier_nodes.into_iter().collect(),
            literal_nodes: manifest.literal_nodes.into_iter().collect(),
            visibility,
            symbols,
            references,
            metrics,
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

fn load_metrics(
    language: &Language,
    path: &Path,
    folder: &str,
) -> Result<Option<MetricsQuery>, ParsingError> {
    if !path.is_file() {
        return Ok(None);
    }
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let source = fs::read_to_string(path).map_err(|e| invalid(e.to_string()))?;
    let query = Query::new(language, &source).map_err(|e| invalid(e.to_string()))?;
    MetricsQuery::new(query).map(Some).map_err(invalid)
}

fn compile(language: &Language, path: &Path, folder: &str) -> Result<Vocabulary, ParsingError> {
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let source = fs::read_to_string(path).map_err(|e| invalid(e.to_string()))?;
    let query = Query::new(language, &source).map_err(|e| invalid(e.to_string()))?;
    Vocabulary::new(query).map_err(invalid)
}
