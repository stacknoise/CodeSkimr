//! The rename preview: what a new name for one symbol would hit, without changing anything
//! (ADR 0104). The places that use the symbol come from the reverse index, the name conflicts
//! from the symbols of the snapshot, and the places the index cannot see (strings, configuration,
//! documentation) from a text search of the project folder. The tool `preview_rename`, the command
//! `codeskimr preview-rename` and the dialog of the explorer all ask [`preview`], so they answer
//! the same.

use std::collections::HashSet;

use codeskimr_domain::{
    Confidence, Direction, FolderPath, Project, SnapshotId, SymbolId, UsageQuery, Visibility,
};

use crate::consts::{
    RENAME_COMMENT_PREFIXES, RENAME_CONFIG_EXTENSIONS, RENAME_CONFLICT_SEARCH_LIMIT,
    RENAME_DOC_EXTENSIONS, RENAME_EXTENSION_SEPARATOR, RENAME_MAX_FILE_BYTES, RENAME_MAX_SITES,
    RENAME_NAMEOF, RENAME_NO_EXTENSION, RENAME_QUOTE_PAIR, RENAME_QUOTES,
};
use crate::error::CodedError;
use crate::ports::{
    CodeIndex, MentionLimits, NameMentions, SearchHit, SymbolDetail, SymbolLabel, UsageIndex,
};

/// Whether `text` can be a new name: a letter or an underscore, then letters, digits and
/// underscores. A language with other rules for names gets the stricter check, which only costs
/// the rare name that the check refuses.
#[must_use]
pub fn is_valid_name(text: &str) -> bool {
    let mut letters = text.chars();
    letters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && letters.all(|letter| letter.is_alphanumeric() || letter == '_')
}

/// The start of the first place in `text` where `word` stands as a whole word: the characters
/// next to it are not letters, digits or underscores.
#[must_use]
pub fn find_word(text: &str, word: &str) -> Option<usize> {
    if word.is_empty() {
        return None;
    }
    text.match_indices(word)
        .map(|(start, _)| start)
        .find(|start| {
            let before = text[..*start].chars().next_back();
            let after = text[start + word.len()..].chars().next();
            !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
        })
}

fn is_word_char(letter: char) -> bool {
    letter.is_alphanumeric() || letter == '_'
}

/// What a place of the rename is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteKind {
    /// The declaration of the symbol, or of a member of the same name (a constructor).
    Declaration,
    /// A place that uses the symbol.
    Usage,
}

/// One place of the code that a rename has to change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// Declaration or usage.
    pub kind: SiteKind,
    /// File of the place.
    pub path: String,
    /// Line, zero-based.
    pub line: u32,
    /// Column, zero-based.
    pub column: u32,
    /// How sure the resolver was; `None` for a declaration.
    pub confidence: Option<Confidence>,
    /// Whether level 2 produced the usage; a declaration is a fact of the syntax and counts as
    /// verified.
    pub verified: bool,
    /// The symbol that holds the place, `None` for a declaration.
    pub container: Option<SymbolLabel>,
    /// The layer that owns the container, if layers are defined.
    pub layer: Option<String>,
}

/// A symbol that already has the new name in the scope of the renamed one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The symbol that has the name.
    pub symbol: SymbolLabel,
    /// File that declares it.
    pub path: String,
    /// Its signature as written.
    pub signature: Option<String>,
    /// Whether the signature is the same but for the name: two such members clash. A different
    /// one may be an overload, which depends on the language.
    pub same_signature: bool,
}

/// What a line that holds the old name as text is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    /// A documentation file (Markdown and the like).
    Documentation,
    /// A configuration file.
    Configuration,
    /// Inside quotes in a code file: a string, perhaps a name that is looked up at run time.
    Text,
    /// The argument of `nameof`, which the compiler changes with the name.
    Nameof,
    /// A comment of a code file.
    Comment,
    /// Code that no usage of the symbol explains: another symbol of the same name, a place the
    /// resolver could not follow, or a declaration of an interface or base class.
    Unresolved,
}

/// One line that holds the old name as text and that no site explains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextHit {
    /// File of the line.
    pub path: String,
    /// Line, zero-based.
    pub line: u32,
    /// The line as written.
    pub text: String,
    /// What the line is.
    pub kind: TextKind,
}

/// The lines with the old name that the index does not explain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextOutcome {
    /// Whether the project folder was searched; a Git source, a missing right to read source and
    /// a failed search leave it `false`.
    pub searched: bool,
    /// All such lines, also those that were cut.
    pub total: usize,
    /// Files the search could not read.
    pub skipped: usize,
    /// The first lines in the order of paths and lines.
    pub hits: Vec<TextHit>,
}

/// How many places lie in one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCount {
    /// Name of the layer.
    pub layer: String,
    /// Number of places.
    pub count: usize,
}

/// The answer of [`preview`].
#[derive(Debug, Clone, PartialEq)]
pub struct RenamePreview {
    /// The symbol that would be renamed.
    pub symbol: SymbolDetail,
    /// The name that is proposed.
    pub new_name: String,
    /// Whether the symbol is public: code outside the project may use it, and this tool does not
    /// see that code.
    pub public_interface: bool,
    /// All places that have to change.
    pub sites_total: usize,
    /// Of them, those that level 2 verified.
    pub verified: usize,
    /// Files that hold them.
    pub files: usize,
    /// The places per layer.
    pub layers: Vec<LayerCount>,
    /// The first places in the order of paths and lines, cut to the limit.
    pub sites: Vec<Site>,
    /// Symbols that already have the new name in the same scope.
    pub conflicts: Vec<Conflict>,
    /// The old name as text.
    pub text: TextOutcome,
}

/// What the preview reads.
#[derive(Clone, Copy)]
pub struct RenamePorts<'a> {
    /// Symbols and search.
    pub index: &'a dyn CodeIndex,
    /// The reverse index.
    pub usages: &'a dyn UsageIndex,
    /// The text search.
    pub mentions: &'a dyn NameMentions,
}

/// What the preview is asked.
#[derive(Clone, Copy)]
pub struct RenameAsk<'a> {
    /// The project, for the layers.
    pub project: &'a Project,
    /// The snapshot the answer is based on.
    pub snapshot: SnapshotId,
    /// The symbol to rename.
    pub symbol: &'a SymbolDetail,
    /// The proposed name, which the caller checked with [`is_valid_name`].
    pub new_name: &'a str,
    /// The folder to search for text, `None` if it cannot or may not be searched.
    pub text_root: Option<&'a FolderPath>,
    /// How many places and lines the answer lists; the counts stay whole.
    pub limit: usize,
}

/// The places that use one symbol (or a member of it that has its name) and its declarations.
/// The places, and how many usages lay beyond the cap of the reverse-index read and are not
/// among them.
fn sites_of(
    ports: &RenamePorts<'_>,
    ask: &RenameAsk<'_>,
) -> Result<(Vec<Site>, usize), CodedError> {
    let old = ask.symbol.symbol.name.as_str();
    let mut ids = vec![ask.symbol.symbol.id];
    ids.extend(
        ask.symbol
            .members
            .iter()
            .filter(|member| member.name.as_str() == old)
            .map(|member| member.id),
    );
    let mut sites = Vec::new();
    let mut beyond = 0;
    for id in ids {
        sites.extend(declaration(ports, ask, id)?);
        let (found, cut) = usages(ports, ask, id)?;
        sites.extend(found);
        beyond += cut;
    }
    // The same place twice is one place; the copy that level 2 verified stays.
    sites.sort_by_key(|site| (site.path.clone(), site.line, site.column, !site.verified));
    sites.dedup_by(|one, two| {
        (one.kind, &one.path, one.line, one.column) == (two.kind, &two.path, two.line, two.column)
    });
    Ok((sites, beyond))
}

fn declaration(
    ports: &RenamePorts<'_>,
    ask: &RenameAsk<'_>,
    id: SymbolId,
) -> Result<Option<Site>, CodedError> {
    let Some(found) = ports.index.symbol_card(ask.snapshot, id)? else {
        return Ok(None);
    };
    let start = found.symbol.span.start();
    Ok(Some(Site {
        kind: SiteKind::Declaration,
        path: found.path.as_str().to_owned(),
        line: start.line,
        column: start.column,
        confidence: None,
        verified: true,
        container: None,
        layer: ask
            .project
            .layer_name_of(found.symbol.qualified_name.as_str()),
    }))
}

fn usages(
    ports: &RenamePorts<'_>,
    ask: &RenameAsk<'_>,
    id: SymbolId,
) -> Result<(Vec<Site>, usize), CodedError> {
    let query = UsageQuery {
        symbol: id,
        direction: Direction::Incoming,
        depth: 1,
        include_tests: true,
    };
    let page = ports
        .usages
        .usages(ask.snapshot, &query, RENAME_MAX_SITES)?;
    let beyond = page.total.saturating_sub(page.entries.len());
    let found = page
        .entries
        .into_iter()
        .map(|entry| {
            let start = entry.reference.span.start();
            Site {
                kind: SiteKind::Usage,
                path: entry.path.as_str().to_owned(),
                line: start.line,
                column: start.column,
                confidence: Some(entry.reference.confidence),
                verified: entry.reference.verified,
                layer: ask
                    .project
                    .layer_name_of(entry.from.qualified_name.as_str()),
                container: Some(entry.from),
            }
        })
        .collect();
    Ok((found, beyond))
}

/// The signature without the name, to tell a clash from an overload.
fn without_name(signature: Option<&str>, name: &str) -> Option<String> {
    let signature = signature?;
    Some(find_word(signature, name).map_or_else(
        || signature.to_owned(),
        |start| [&signature[..start], &signature[start + name.len()..]].concat(),
    ))
}

/// The symbols that have the new name where the renamed symbol stands: the qualified name of the
/// symbol with its last segment replaced.
fn conflicts_of(ports: &RenamePorts<'_>, ask: &RenameAsk<'_>) -> Result<Vec<Conflict>, CodedError> {
    let symbol = &ask.symbol.symbol;
    let old = symbol.name.as_str();
    let Some(scope) = symbol.qualified_name.as_str().strip_suffix(old) else {
        return Ok(Vec::new());
    };
    let target = [scope, ask.new_name].concat();
    let own = without_name(symbol.signature.as_deref(), old);
    let hits = ports
        .index
        .search(ask.snapshot, ask.new_name, RENAME_CONFLICT_SEARCH_LIMIT)?;
    let mut found = Vec::new();
    for hit in hits
        .iter()
        .filter(|hit| is_conflict(hit, &target, symbol.id))
    {
        let other = ports.index.symbol_card(ask.snapshot, hit.symbol)?;
        let signature = other.and_then(|card| card.symbol.signature);
        found.push(Conflict {
            same_signature: without_name(signature.as_deref(), ask.new_name) == own,
            symbol: SymbolLabel {
                id: hit.symbol,
                qualified_name: hit.qualified_name.clone(),
                kind: hit.kind,
            },
            path: hit.path.as_str().to_owned(),
            signature,
        });
    }
    Ok(found)
}

fn is_conflict(hit: &SearchHit, target: &str, own: SymbolId) -> bool {
    hit.symbol != own && hit.qualified_name.as_str() == target
}

/// What a line with the old name is, from the file it stands in and the way it is written.
#[must_use]
pub fn classify(path: &str, line: &str, name: &str) -> TextKind {
    let extension = path
        .rsplit_once(RENAME_EXTENSION_SEPARATOR)
        .map_or(RENAME_NO_EXTENSION, |(_, extension)| extension);
    let is_one_of = |list: &[&str]| {
        list.iter()
            .any(|known| known.eq_ignore_ascii_case(extension))
    };
    if is_one_of(&RENAME_DOC_EXTENSIONS) {
        return TextKind::Documentation;
    }
    if is_one_of(&RENAME_CONFIG_EXTENSIONS) {
        return TextKind::Configuration;
    }
    let text = line.trim_start();
    let at = find_word(text, name).unwrap_or_default();
    if text.contains(RENAME_NAMEOF) {
        TextKind::Nameof
    } else if RENAME_COMMENT_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
    {
        TextKind::Comment
    } else if RENAME_QUOTES
        .iter()
        .any(|quote| text[..at].matches(*quote).count() % RENAME_QUOTE_PAIR == 1)
    {
        TextKind::Text
    } else {
        TextKind::Unresolved
    }
}

/// The old name as text in the project folder, without the lines that the sites explain.
fn text_of(
    ports: &RenamePorts<'_>,
    ask: &RenameAsk<'_>,
    sites: &[Site],
) -> Result<TextOutcome, CodedError> {
    let Some(root) = ask.text_root else {
        return Ok(TextOutcome::default());
    };
    let known: HashSet<(String, u32)> = sites
        .iter()
        .map(|site| (site.path.clone(), site.line))
        .collect();
    let name = ask.symbol.symbol.name.as_str();
    let limits = MentionLimits {
        max_items: ask.limit,
        max_file_bytes: RENAME_MAX_FILE_BYTES,
    };
    let found = ports.mentions.mentions(root, name, &limits, &known)?;
    Ok(TextOutcome {
        searched: true,
        total: found.total,
        skipped: found.skipped,
        hits: found
            .items
            .into_iter()
            .map(|mention| TextHit {
                kind: classify(mention.path.as_str(), &mention.text, name),
                path: mention.path.as_str().to_owned(),
                line: mention.line,
                text: mention.text,
            })
            .collect(),
    })
}

fn count_layers(sites: &[Site]) -> Vec<LayerCount> {
    let mut counts: Vec<LayerCount> = Vec::new();
    for layer in sites.iter().filter_map(|site| site.layer.as_ref()) {
        match counts.iter_mut().find(|found| &found.layer == layer) {
            Some(found) => found.count += 1,
            None => counts.push(LayerCount {
                layer: layer.clone(),
                count: 1,
            }),
        }
    }
    counts.sort_by(|one, two| one.layer.cmp(&two.layer));
    counts
}

/// What renaming the symbol to the new name would hit. Nothing is changed and nothing is stored.
/// Only direct usages count (the places that name the symbol), as everywhere in the product; the
/// answer says what it cannot see through the text hits and the flag `public_interface`.
///
/// # Errors
/// A coded failure of a port: the `STO` group of the index, `SRC-201` of the text search.
pub fn preview(ports: &RenamePorts<'_>, ask: &RenameAsk<'_>) -> Result<RenamePreview, CodedError> {
    let (all, beyond) = sites_of(ports, ask)?;
    let text = text_of(ports, ask, &all)?;
    let files: HashSet<&str> = all.iter().map(|site| site.path.as_str()).collect();
    Ok(RenamePreview {
        new_name: ask.new_name.to_owned(),
        public_interface: ask.symbol.symbol.visibility == Some(Visibility::Public),
        sites_total: all.len() + beyond,
        verified: all.iter().filter(|site| site.verified).count(),
        files: files.len(),
        layers: count_layers(&all),
        conflicts: conflicts_of(ports, ask)?,
        text,
        sites: all.into_iter().take(ask.limit).collect(),
        symbol: ask.symbol.clone(),
    })
}
