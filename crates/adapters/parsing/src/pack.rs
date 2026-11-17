//! A loaded language pack: grammar, rules and compiled queries.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use codeskimr_domain::{LanguageId, ModelError, SymbolKind};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use tree_sitter::{Language, Query};

use crate::captures::{MetricsQuery, Vocabulary};
use crate::consts::{
    ID_MISMATCH, METRICS_QUERY_FILE, PATH_DOT, PATH_SEPARATORS, QUERIES_FILE_UNKNOWN,
    QUERIES_FROM_CHAINED, QUERIES_FROM_INVALID, QUERIES_FROM_SELF, QUERY_JOIN,
    REFERENCES_QUERY_FILE, SHAREABLE_QUERY_FILES, SYMBOLS_QUERY_FILE, TEST_PATTERN_SEPARATOR,
};
use crate::error::ParsingError;
use crate::grammar;
use crate::manifest::{DocBlock, DocSummary, PackManifest, QueriesFrom, Tier};
use crate::visibility::Visibilities;

/// Everything the analyzer needs to understand one language.
pub struct LanguagePack {
    pub(crate) id: LanguageId,
    pub(crate) tier: Tier,
    pub(crate) extensions: Vec<String>,
    project_files: Vec<String>,
    test_patterns: Vec<String>,
    test_files: GlobSet,
    pub(crate) language: Language,
    pub(crate) separator: String,
    /// Whether the name of the file starts every qualified name (ADR 0036).
    pub(crate) file_in_names: bool,
    /// Whether a name without receiver inside a function can mean a member of its type (ADR 0053).
    pub(crate) bare_names_reach_members: bool,
    pub(crate) comment_nodes: HashSet<String>,
    /// Node kinds whose start and end the content hash marks (blocks without delimiting tokens).
    pub(crate) block_nodes: HashSet<String>,
    pub(crate) doc_prefixes: Vec<String>,
    pub(crate) doc_before_declaration: Option<HashSet<String>>,
    pub(crate) static_keywords: HashSet<String>,
    pub(crate) modifier_kinds: HashMap<String, SymbolKind>,
    /// Kind a declaration gets inside a parent: (parent kind, kind) to kind.
    kinds_by_parent: HashMap<(SymbolKind, SymbolKind), SymbolKind>,
    pub(crate) doc_summary: Option<DocSummary>,
    pub(crate) doc_block: Option<DocBlock>,
    pub(crate) body_nodes: HashSet<String>,
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
        let shared = shared_queries(dir, &manifest, &folder)?;
        let text = |file: &str| query_text(dir, shared.as_ref(), file, &folder);
        let symbols = text(SYMBOLS_QUERY_FILE)?
            .ok_or_else(|| invalid(SYMBOLS_QUERY_FILE.to_owned()))
            .and_then(|source| compile(&language, &source, &folder))?;
        let references = text(REFERENCES_QUERY_FILE)?
            .map(|source| compile(&language, &source, &folder))
            .transpose()?;
        let metrics = text(METRICS_QUERY_FILE)?
            .map(|source| load_metrics(&language, &source, &folder))
            .transpose()?;
        Self::assemble(manifest, language, (symbols, references, metrics), &folder)
    }

    fn assemble(
        manifest: PackManifest,
        language: Language,
        (symbols, references, metrics): (Vocabulary, Option<Vocabulary>, Option<MetricsQuery>),
        folder: &str,
    ) -> Result<Self, ParsingError> {
        let invalid = |reason: String| ParsingError::invalid(folder, reason);
        let id = pack_id(&manifest.id, folder)?;
        let visibility = Visibilities::resolve(manifest.visibility).map_err(invalid)?;
        let modifier_kinds =
            modifier_kinds(manifest.modifier_kinds).map_err(|e| invalid(e.to_string()))?;
        let kinds_by_parent = parent_kinds(manifest.kinds_by_parent).map_err(invalid)?;
        Ok(Self {
            kinds_by_parent,
            id,
            tier: manifest.tier,
            extensions: manifest.extensions,
            project_files: manifest.manifest_files,
            test_files: test_files(&manifest.test_patterns).map_err(invalid)?,
            test_patterns: manifest.test_patterns,
            language,
            separator: manifest.separator,
            file_in_names: manifest.file_in_names,
            bare_names_reach_members: manifest.bare_names_reach_members,
            comment_nodes: manifest.comment_nodes.into_iter().collect(),
            block_nodes: manifest.block_nodes.into_iter().collect(),
            doc_prefixes: manifest.doc_prefixes,
            doc_before_declaration: manifest.doc_before_declaration.map(into_set),
            static_keywords: manifest.static_keywords.into_iter().collect(),
            modifier_kinds,
            doc_summary: manifest.doc_summary,
            doc_block: manifest.doc_block,
            body_nodes: manifest.body_nodes.into_iter().collect(),
            identifier_nodes: manifest.identifier_nodes.into_iter().collect(),
            literal_nodes: manifest.literal_nodes.into_iter().collect(),
            visibility,
            symbols,
            references,
            metrics,
        })
    }

    /// The kind a declaration of `kind` gets inside a parent of kind `parent`.
    pub(crate) fn kind_in(&self, parent: Option<SymbolKind>, kind: SymbolKind) -> SymbolKind {
        parent
            .and_then(|parent| self.kinds_by_parent.get(&(parent, kind)))
            .copied()
            .unwrap_or(kind)
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

    /// Whether `path` (relative to the project, with `/`) is a test file of the language: one of
    /// the `test_patterns` matches.
    #[must_use]
    pub fn is_test_file(&self, path: &str) -> bool {
        self.test_files.is_match(path)
    }

    /// File extensions the pack claims, without the dot.
    #[must_use]
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

/// The kinds that a modifier keyword gives to a declaration.
fn modifier_kinds(
    labels: HashMap<String, String>,
) -> Result<HashMap<String, SymbolKind>, ModelError> {
    labels
        .into_iter()
        .map(|(keyword, label)| Ok((keyword, SymbolKind::from_str(&label)?)))
        .collect()
}

fn parent_kinds(
    rules: HashMap<String, HashMap<String, String>>,
) -> Result<HashMap<(SymbolKind, SymbolKind), SymbolKind>, String> {
    let parse = |label: &str| SymbolKind::from_str(label).map_err(|_| label.to_owned());
    let mut kinds = HashMap::new();
    for (parent, inner) in rules {
        for (from, to) in inner {
            kinds.insert((parse(&parent)?, parse(&from)?), parse(&to)?);
        }
    }
    Ok(kinds)
}

/// The items as a set.
fn into_set(items: Vec<String>) -> HashSet<String> {
    items.into_iter().collect()
}

/// The patterns of the manifest as one matcher; a broken pattern is a reason for the pack.
fn test_files(patterns: &[String]) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|e| {
                [pattern.as_str(), e.kind().to_string().as_str()].join(TEST_PATTERN_SEPARATOR)
            })?;
        builder.add(glob);
    }
    builder.build().map_err(|e| e.to_string())
}

fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn load_metrics(
    language: &Language,
    source: &str,
    folder: &str,
) -> Result<MetricsQuery, ParsingError> {
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let query = Query::new(language, source).map_err(|e| invalid(e.to_string()))?;
    MetricsQuery::new(query).map_err(invalid)
}

fn compile(language: &Language, source: &str, folder: &str) -> Result<Vocabulary, ParsingError> {
    let invalid = |reason: String| ParsingError::invalid(folder, reason);
    let query = Query::new(language, source).map_err(|e| invalid(e.to_string()))?;
    Vocabulary::new(query).map_err(invalid)
}

/// The text of the query file `file` of the pack in `dir`: the text of the pack that
/// `queries.files` takes it from (ADR 0056), then the file of the pack itself. `None` if neither
/// exists.
fn query_text(
    dir: &Path,
    shared: Option<&(&QueriesFrom, PathBuf)>,
    file: &str,
    folder: &str,
) -> Result<Option<String>, ParsingError> {
    let read = |path: PathBuf| {
        if path.is_file() {
            fs::read_to_string(&path)
                .map(Some)
                .map_err(|e| ParsingError::invalid(folder, e.to_string()))
        } else {
            Ok(None)
        }
    };
    let own = read(dir.join(file))?;
    let Some((_, base_dir)) = shared.filter(|(queries, _)| queries.files.iter().any(|f| f == file))
    else {
        return Ok(own);
    };
    Ok(match (read(base_dir.join(file))?, own) {
        (Some(base), Some(own)) => Some([base, own].join(QUERY_JOIN)),
        (base, own) => base.or(own),
    })
}

/// `queries` of the manifest with the folder it takes files from, after the checks of ADR 0056.
fn shared_queries<'m>(
    dir: &Path,
    manifest: &'m PackManifest,
    folder: &str,
) -> Result<Option<(&'m QueriesFrom, PathBuf)>, ParsingError> {
    manifest
        .queries
        .as_ref()
        .map(|queries| shared_dir(dir, queries, folder).map(|base| (queries, base)))
        .transpose()
}

/// The folder of the pack that `queries` takes files from, after the checks of ADR 0056.
fn shared_dir(dir: &Path, queries: &QueriesFrom, folder: &str) -> Result<PathBuf, ParsingError> {
    let invalid =
        |reason: &str| ParsingError::invalid(folder, [reason, queries.from.as_str()].concat());
    let plain = !queries.from.is_empty()
        && !queries.from.contains(PATH_SEPARATORS)
        && !queries.from.contains(PATH_DOT);
    if !plain {
        return Err(invalid(QUERIES_FROM_INVALID));
    }
    if queries.from == folder {
        return Err(invalid(QUERIES_FROM_SELF));
    }
    if let Some(unknown) = queries
        .files
        .iter()
        .find(|f| !SHAREABLE_QUERY_FILES.contains(&f.as_str()))
    {
        return Err(ParsingError::invalid(
            folder,
            [QUERIES_FILE_UNKNOWN, unknown].concat(),
        ));
    }
    let base = dir.parent().unwrap_or(dir).join(&queries.from);
    if PackManifest::read(&base)?.queries.is_some() {
        return Err(invalid(QUERIES_FROM_CHAINED));
    }
    Ok(base)
}

/// The language id of a manifest, which must equal the name of its folder.
pub(crate) fn pack_id(id: &str, folder: &str) -> Result<LanguageId, ParsingError> {
    if id != folder {
        return Err(ParsingError::invalid(folder, [ID_MISMATCH, id].concat()));
    }
    LanguageId::new(id).map_err(|e| ParsingError::invalid(folder, e.to_string()))
}
