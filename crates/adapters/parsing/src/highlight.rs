//! Colouring code with the `highlights.scm` of a language pack (blueprint 8.1, ADR 0064).
//!
//! The highlighter reads only the grammar and `highlights.scm` of a pack, never the symbol,
//! reference or metric queries, so asking for colours does not pay the load time that ADR 0063
//! budgets. A language is prepared when a file of it is first coloured.

use std::collections::HashMap;
use std::fs;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;

use codeskimr_application::ports::{SyntaxHighlighter, TokenClass, TokenSpan};
use codeskimr_domain::RelativePath;
use tree_sitter::{ParseOptions, ParseState, Parser, Query, QueryCursor, StreamingIterator};

use crate::consts::{
    CAPTURE_PART_SEPARATOR, HIGHLIGHT_ATTRIBUTE, HIGHLIGHT_COMMENT, HIGHLIGHT_CONSTANT,
    HIGHLIGHT_FUNCTION, HIGHLIGHT_KEYWORD, HIGHLIGHT_LABEL, HIGHLIGHT_NUMBER, HIGHLIGHT_QUERY_FILE,
    HIGHLIGHT_STRING, HIGHLIGHT_TAG, HIGHLIGHT_TIMEOUT, HIGHLIGHT_TYPE, HIGHLIGHT_VARIABLE_BUILTIN,
    LINE_BREAK, PATH_DOT,
};
use crate::grammar;
use crate::manifest::PackManifest;
use crate::packs::pack_dirs;

/// The class a capture of `highlights.scm` stands for; `None` for a capture the UI does not colour.
fn class_of(capture: &str) -> Option<TokenClass> {
    if capture == HIGHLIGHT_VARIABLE_BUILTIN {
        return Some(TokenClass::Keyword);
    }
    let head = capture
        .split(CAPTURE_PART_SEPARATOR)
        .next()
        .unwrap_or(capture);
    match head {
        HIGHLIGHT_COMMENT => Some(TokenClass::Comment),
        HIGHLIGHT_STRING => Some(TokenClass::String),
        HIGHLIGHT_NUMBER | HIGHLIGHT_CONSTANT => Some(TokenClass::Number),
        HIGHLIGHT_KEYWORD => Some(TokenClass::Keyword),
        HIGHLIGHT_TYPE | HIGHLIGHT_TAG => Some(TokenClass::Type),
        HIGHLIGHT_FUNCTION => Some(TokenClass::Function),
        HIGHLIGHT_ATTRIBUTE | HIGHLIGHT_LABEL => Some(TokenClass::Attribute),
        _ => None,
    }
}

/// Grammar and compiled `highlights.scm` of one language.
struct Colours {
    language: tree_sitter::Language,
    query: Query,
    /// The class of each capture of the query, by index; `None` means plain.
    classes: Vec<Option<TokenClass>>,
}

impl Colours {
    fn load(grammar_name: &str, dir: &Path) -> Option<Self> {
        let language = grammar::builtin(grammar_name)?;
        let source = fs::read_to_string(dir.join(HIGHLIGHT_QUERY_FILE)).ok()?;
        let query = Query::new(&language, &source).ok()?;
        let classes = query
            .capture_names()
            .iter()
            .map(|name| class_of(name))
            .collect();
        Some(Self {
            language,
            query,
            classes,
        })
    }

    /// The class of every byte of `text`; where captures overlap, the later one (the more
    /// specific, as it lies inside the earlier) wins.
    fn paint(&self, text: &str) -> Option<Vec<Option<TokenClass>>> {
        let mut parser = Parser::new();
        parser.set_language(&self.language).ok()?;
        let started = Instant::now();
        let mut late = |_: &ParseState| {
            if started.elapsed() > HIGHLIGHT_TIMEOUT {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        };
        let mut input =
            |at: usize, _: tree_sitter::Point| text.as_bytes().get(at..).unwrap_or_default();
        let options = ParseOptions::new().progress_callback(&mut late);
        let tree = parser.parse_with_options(&mut input, None, Some(options))?;
        let mut bytes = vec![None; text.len()];
        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&self.query, tree.root_node(), text.as_bytes());
        while let Some((found, which)) = captures.next() {
            let Some(capture) = found.captures().get(*which) else {
                continue;
            };
            let class = self.classes.get(capture.index as usize).copied().flatten();
            if let Some(slice) = bytes.get_mut(capture.node.byte_range()) {
                slice.fill(class);
            }
        }
        Some(bytes)
    }
}

/// One installed language: what the highlighter needs to find and prepare its colours.
struct Language {
    grammar: String,
    dir: PathBuf,
    colours: OnceLock<Option<Colours>>,
}

/// Colours code with the `highlights.scm` of the packs below one folder.
pub struct TreeSitterHighlighter {
    languages: HashMap<String, Language>,
    /// File extension to the id of the language.
    by_extension: HashMap<String, String>,
}

impl TreeSitterHighlighter {
    /// Reads the manifest of every pack below `root`. A pack whose manifest cannot be read, or
    /// that has no `highlights.scm`, is left out: its code stays plain.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        let mut languages = HashMap::new();
        let mut by_extension = HashMap::new();
        for dir in pack_dirs(root).unwrap_or_default() {
            let Ok(manifest) = PackManifest::read(&dir) else {
                continue;
            };
            if !dir.join(HIGHLIGHT_QUERY_FILE).is_file() {
                continue;
            }
            for extension in &manifest.extensions {
                by_extension.insert(extension.to_lowercase(), manifest.id.clone());
            }
            languages.insert(
                manifest.id,
                Language {
                    grammar: manifest.grammar,
                    dir,
                    colours: OnceLock::new(),
                },
            );
        }
        Self {
            languages,
            by_extension,
        }
    }

    fn colours_of(&self, path: &RelativePath) -> Option<&Colours> {
        let (_, extension) = path.as_str().rsplit_once(PATH_DOT)?;
        let id = self.by_extension.get(&extension.to_lowercase())?;
        let language = self.languages.get(id)?;
        language
            .colours
            .get_or_init(|| Colours::load(&language.grammar, &language.dir))
            .as_ref()
    }
}

impl SyntaxHighlighter for TreeSitterHighlighter {
    fn highlight(&self, path: &RelativePath, lines: &[String]) -> Vec<TokenSpan> {
        let Some(colours) = self.colours_of(path) else {
            return Vec::new();
        };
        let text = lines.join(LINE_BREAK);
        colours
            .paint(&text)
            .map(|bytes| runs(lines, &bytes))
            .unwrap_or_default()
    }
}

/// The runs of equal class inside the lines, given the class of each byte of the joined text.
fn runs(lines: &[String], bytes: &[Option<TokenClass>]) -> Vec<TokenSpan> {
    let mut spans = Vec::new();
    let mut offset = 0;
    for (index, line) in lines.iter().enumerate() {
        let classes = bytes.get(offset..offset + line.len()).unwrap_or_default();
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        spans.extend(
            runs_of(classes)
                .into_iter()
                .map(|(start, end, class)| TokenSpan {
                    line: index,
                    start,
                    end,
                    class,
                }),
        );
        offset += line.len() + LINE_BREAK.len();
    }
    spans
}

/// The runs of one line: first byte, position after the last byte and class of every stretch of
/// equal, coloured bytes.
fn runs_of(classes: &[Option<TokenClass>]) -> Vec<(u32, u32, TokenClass)> {
    let mut found: Vec<(u32, u32, TokenClass)> = Vec::new();
    for (at, class) in classes.iter().enumerate() {
        let (Some(class), Ok(at)) = (class, u32::try_from(at)) else {
            continue;
        };
        match found.last_mut() {
            Some((_, end, last)) if *end == at && last == class => *end = at + 1,
            _ => found.push((at, at + 1, *class)),
        }
    }
    found
}
