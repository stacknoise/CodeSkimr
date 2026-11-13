//! Extraction of symbols from a syntax tree with the declaration query of a language pack.

use std::cmp::Reverse;
use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;

use codeskimr_application::ids::IdSequence;
use codeskimr_domain::{
    FileId, QualifiedName, RelativePath, Span, Symbol, SymbolId, SymbolKind, SymbolName, Visibility,
};
use tree_sitter::{Node, QueryCapture, QueryCursor, StreamingIterator};

use crate::captures::Capture;
use crate::consts::{
    BODY_FIELD, CALLABLE_KINDS, LINE_SEPARATOR, NO_EXTENSION, PATH_SEPARATOR, WORD_SEPARATOR,
};
use crate::error::ParsingError;
use crate::location::{span_between, span_of, text_of};
use crate::manifest::{DocBlock, DocSummary};
use crate::pack::LanguagePack;

/// What the extraction needs to know about the file.
pub struct SymbolContext<'a> {
    /// Language pack of the file.
    pub pack: &'a LanguagePack,
    /// Decoded text of the file.
    pub source: &'a str,
    /// Id of the file.
    pub file: FileId,
    /// Path of the file inside the project.
    pub path: &'a RelativePath,
    /// Source of ids.
    pub ids: &'a IdSequence,
}

/// The symbols of a file and where they sit, so occurrences can be assigned to a symbol.
pub struct SymbolTable {
    /// Symbols in source order; the first one is the module of the file.
    pub symbols: Vec<Symbol>,
    ranges: Vec<Range<usize>>,
    parents: Vec<Option<usize>>,
    with_body: HashSet<SymbolId>,
}

impl SymbolTable {
    /// Whether the declaration has a body (the grammar's `body` field); abstract and interface
    /// members have none and are not measured.
    #[must_use]
    pub fn has_body(&self, symbol: SymbolId) -> bool {
        self.with_body.contains(&symbol)
    }

    /// The innermost symbol that contains the byte offset; the module if no other does.
    ///
    /// Symbols are sorted by start, so the last one that starts at or before the offset is the
    /// candidate; its enclosing chain is as deep as the nesting, not as long as the file.
    #[must_use]
    pub fn innermost(&self, offset: usize) -> Option<&Symbol> {
        let started = self.ranges.partition_point(|range| range.start <= offset);
        let mut index = started.checked_sub(1);
        while let Some(current) = index {
            if self
                .ranges
                .get(current)
                .is_some_and(|range| range.contains(&offset))
            {
                return self.symbols.get(current);
            }
            index = self.parents.get(current).copied().flatten();
        }
        self.symbols.first()
    }

    /// The callable symbols (functions, methods, constructors) with the byte range of their
    /// declaration.
    pub fn callables(&self) -> impl Iterator<Item = (&Symbol, &Range<usize>)> {
        self.symbols
            .iter()
            .zip(&self.ranges)
            .filter(|(symbol, _)| CALLABLE_KINDS.contains(&symbol.kind))
    }
}

struct Declaration<'t> {
    node: Node<'t>,
    kind: SymbolKind,
    name: Node<'t>,
    modifiers: Vec<Node<'t>>,
    is_static: bool,
    docs: Vec<Node<'t>>,
    signature: Option<Node<'t>>,
    scope_end: Option<Node<'t>>,
}

#[derive(Default)]
struct Parts<'t> {
    declaration: Option<(Node<'t>, SymbolKind)>,
    name: Option<Node<'t>>,
    modifiers: Vec<Node<'t>>,
    is_static: bool,
    docs: Vec<Node<'t>>,
    signature: Option<Node<'t>>,
    scope_end: Option<Node<'t>>,
}

impl<'t> Parts<'t> {
    fn add(&mut self, capture: Capture, node: Node<'t>) {
        match capture {
            Capture::Symbol(kind) => self.declaration = Some((node, kind)),
            Capture::Name => self.name = Some(node),
            Capture::Visibility | Capture::Modifier => self.modifiers.push(node),
            Capture::Static => self.is_static = true,
            Capture::Doc => self.docs.push(node),
            Capture::Signature => self.signature = Some(node),
            Capture::ScopeEnd => self.scope_end = Some(node),
            Capture::Reference(_)
            | Capture::Receiver(_)
            | Capture::Local
            | Capture::Field
            | Capture::DeclaredType
            | Capture::Scope => {}
        }
    }

    fn finish(self) -> Option<Declaration<'t>> {
        let (node, kind) = self.declaration?;
        Some(Declaration {
            node,
            kind,
            name: self.name?,
            modifiers: self.modifiers,
            is_static: self.is_static,
            docs: self.docs,
            signature: self.signature,
            scope_end: self.scope_end,
        })
    }
}

fn collect<'t>(ctx: &SymbolContext<'_>, root: Node<'t>) -> Vec<Declaration<'t>> {
    let vocabulary = &ctx.pack.symbols;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&vocabulary.query, root, ctx.source.as_bytes());
    let mut found = Vec::new();
    while let Some(found_match) = matches.next() {
        let mut parts = Parts::default();
        for QueryCapture { node, index } in found_match.captures() {
            if let Some(capture) = vocabulary.role(*index) {
                parts.add(capture, *node);
            }
        }
        found.extend(parts.finish());
    }
    found.sort_by_key(|d| (d.node.start_byte(), Reverse(d.node.end_byte()), d.node.id()));
    // Several matches may describe the same declaration, for example one per subset of a
    // repeated capture; they are merged into the first one.
    found.dedup_by(|later, first| {
        let same = (later.node.id(), later.kind) == (first.node.id(), first.kind);
        if same {
            first.absorb(later);
        }
        same
    });
    found
}

impl<'t> Declaration<'t> {
    /// Adds the repeated captures of another match of the same declaration.
    fn absorb(&mut self, other: &mut Declaration<'t>) {
        self.is_static |= other.is_static;
        self.scope_end = self.scope_end.or(other.scope_end);
        self.signature = self.signature.or(other.signature);
        merge_nodes(&mut self.modifiers, &mut other.modifiers);
        merge_nodes(&mut self.docs, &mut other.docs);
    }
}

/// Appends the nodes of `more` that `nodes` lacks and restores source order.
fn merge_nodes<'t>(nodes: &mut Vec<Node<'t>>, more: &mut Vec<Node<'t>>) {
    nodes.append(more);
    nodes.sort_by_key(Node::id);
    nodes.dedup_by_key(|node| node.id());
    nodes.sort_by_key(Node::start_byte);
}

struct Frame {
    kind: SymbolKind,
    end: usize,
    index: usize,
    id: SymbolId,
    qualified: String,
}

/// Everything a new symbol needs except its place in the hierarchy.
struct NewSymbol {
    kind: SymbolKind,
    name: String,
    span: Span,
    range: Range<usize>,
    visibility: Option<Visibility>,
    is_static: bool,
    signature: Option<String>,
    doc: Option<String>,
    has_body: bool,
}

struct Builder<'a, 'c> {
    ctx: &'c SymbolContext<'a>,
    symbols: Vec<Symbol>,
    ranges: Vec<Range<usize>>,
    parents: Vec<Option<usize>>,
    with_body: HashSet<SymbolId>,
    stack: Vec<Frame>,
}

/// Extracts the symbols of the file: one module symbol and every declaration the pack finds.
///
/// # Errors
/// A model error if a name or span violates the domain rules, which is a bug of the pack or adapter.
pub fn extract(ctx: &SymbolContext<'_>, root: Node<'_>) -> Result<SymbolTable, ParsingError> {
    let mut builder = Builder {
        ctx,
        symbols: Vec::new(),
        ranges: Vec::new(),
        parents: Vec::new(),
        with_body: HashSet::new(),
        stack: Vec::new(),
    };
    builder.add_module(root)?;
    for declaration in collect(ctx, root) {
        builder.add_declaration(&declaration)?;
    }
    Ok(SymbolTable {
        symbols: builder.symbols,
        ranges: builder.ranges,
        parents: builder.parents,
        with_body: builder.with_body,
    })
}

impl Builder<'_, '_> {
    fn add_module(&mut self, root: Node<'_>) -> Result<(), ParsingError> {
        let path = Path::new(self.ctx.path.as_str());
        let qualified = path
            .with_extension(NO_EXTENSION)
            .to_string_lossy()
            .replace(PATH_SEPARATOR, &self.ctx.pack.separator);
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.push(
            qualified,
            NewSymbol {
                kind: SymbolKind::Module,
                name,
                span: span_of(root)?,
                range: 0..root.end_byte(),
                visibility: None,
                is_static: false,
                signature: None,
                doc: None,
                has_body: false,
            },
        )
    }

    fn add_declaration(&mut self, declaration: &Declaration<'_>) -> Result<(), ParsingError> {
        let ctx = self.ctx;
        let start = declaration.node.start_byte();
        // The module frame at the bottom is never popped.
        while self.stack.len() > 1 && self.stack.last().is_some_and(|top| top.end <= start) {
            self.stack.pop();
        }
        let name = text_of(declaration.name, ctx.source).to_owned();
        let prefix = self
            .stack
            .last()
            .map_or_else(<&str>::default, |frame| frame.qualified.as_str());
        let qualified = [prefix, ctx.pack.separator.as_str(), name.as_str()].concat();
        let parent = self.stack.last().map(|frame| frame.kind);
        let new = new_symbol(ctx, declaration, name, parent)?;
        self.push(qualified, new)
    }

    fn push(&mut self, qualified: String, new: NewSymbol) -> Result<(), ParsingError> {
        let id = self.ctx.ids.symbol();
        let index = self.symbols.len();
        self.parents
            .push(self.stack.last().map(|frame| frame.index));
        self.symbols.push(Symbol {
            id,
            file: self.ctx.file,
            kind: new.kind,
            name: SymbolName::new(new.name)?,
            qualified_name: QualifiedName::new(qualified.clone())?,
            parent: self.stack.last().map(|frame| frame.id),
            visibility: new.visibility,
            is_static: new.is_static,
            span: new.span,
            signature: new.signature,
            doc: new.doc,
        });
        if new.has_body {
            self.with_body.insert(id);
        }
        self.stack.push(Frame {
            kind: new.kind,
            end: new.range.end,
            index,
            id,
            qualified,
        });
        self.ranges.push(new.range);
        Ok(())
    }
}

/// The symbol for `declaration`; a scope end node stretches span and range beyond the declaration.
fn new_symbol(
    ctx: &SymbolContext<'_>,
    declaration: &Declaration<'_>,
    name: String,
    parent: Option<SymbolKind>,
) -> Result<NewSymbol, ParsingError> {
    let node = declaration.node;
    let (span, end) = match declaration.scope_end {
        Some(last) => (span_between(node, last)?, last.end_byte()),
        None => (span_of(node)?, node.end_byte()),
    };
    let words: Vec<&str> = declaration
        .modifiers
        .iter()
        .map(|modifier| text_of(*modifier, ctx.source).trim())
        .collect();
    let pack = ctx.pack;
    let kind = kind_of(pack, declaration, &words, parent);
    let visibility = pack.visibility.of(kind, &name, &words, parent);
    Ok(NewSymbol {
        kind,
        name,
        span,
        range: node.start_byte()..end,
        visibility,
        is_static: declaration.is_static
            || words
                .iter()
                .any(|word| pack.static_keywords.contains(*word)),
        signature: declaration
            .signature
            .map(|node| text_of(node, ctx.source).trim().to_owned()),
        doc: doc(ctx, &declaration.docs),
        has_body: has_body(pack, declaration.node),
    })
}

/// The kind of the declaration: a modifier can change it, and the parent can give it another one.
fn kind_of(
    pack: &LanguagePack,
    declaration: &Declaration<'_>,
    words: &[&str],
    parent: Option<SymbolKind>,
) -> SymbolKind {
    let kind = words
        .iter()
        .find_map(|word| pack.modifier_kinds.get(*word))
        .copied()
        .unwrap_or(declaration.kind);
    pack.kind_in(parent, kind)
}

/// Whether the declaration has a body: the grammar's `body` field or a child the pack calls a body.
fn has_body(pack: &LanguagePack, node: Node<'_>) -> bool {
    node.child_by_field_name(BODY_FIELD).is_some()
        || (0..node.child_count())
            .filter_map(|index| node.child(index))
            .any(|child| pack.body_nodes.contains(child.kind()))
}

fn doc(ctx: &SymbolContext<'_>, nodes: &[Node<'_>]) -> Option<String> {
    let lines: Vec<String> = nodes
        .iter()
        .filter_map(|node| doc_text(ctx.pack, text_of(*node, ctx.source)))
        .flat_map(|text| {
            text.lines()
                .map(|line| line.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    let text = lines.join(LINE_SEPARATOR).trim().to_owned();
    let text = summary(ctx.pack.doc_summary.as_ref(), &text).unwrap_or(text);
    (!text.is_empty()).then_some(text)
}

/// The documentation in one comment: the description of a block comment (`JSDoc`), or the text of a
/// line comment after its doc prefix. A pack with blocks and without prefixes ignores line comments.
fn doc_text(pack: &LanguagePack, comment: &str) -> Option<String> {
    let text = comment.trim_start();
    match &pack.doc_block {
        Some(block) if is_block(block, text) => Some(block_text(block, text)),
        Some(_) if pack.doc_prefixes.is_empty() => None,
        _ => strip_prefix(&pack.doc_prefixes, text).map(str::to_owned),
    }
}

/// Whether `text` is a complete block comment; `/**/` is an empty comment, not a block.
fn is_block(block: &DocBlock, text: &str) -> bool {
    text.starts_with(block.open.as_str())
        && text.ends_with(block.close.as_str())
        && text.len() >= block.open.len() + block.close.len()
}

/// The line without its marker; a marker that is part of the text (`**bold**`) stays.
fn without_marker<'t>(line: &'t str, marker: &str) -> &'t str {
    match line.strip_prefix(marker) {
        Some(rest) if rest.is_empty() || rest.starts_with(char::is_whitespace) => rest.trim(),
        _ => line,
    }
}

/// The description inside a block comment: delimiters and line markers removed, ending at the
/// first line that starts with `stop_at`.
fn block_text(block: &DocBlock, text: &str) -> String {
    let inner = text.strip_prefix(block.open.as_str()).unwrap_or(text);
    let inner = inner.strip_suffix(block.close.as_str()).unwrap_or(inner);
    let lines = inner
        .lines()
        .map(|line| without_marker(line.trim(), block.line_prefix.as_str()));
    let kept: Vec<&str> = match block.stop_at.as_deref() {
        Some(stop) => lines.take_while(|line| !line.starts_with(stop)).collect(),
        None => lines.collect(),
    };
    kept.join(LINE_SEPARATOR).trim().to_owned()
}

/// The text between the summary tags with its whitespace collapsed (empty for an empty summary);
/// `None` if the pack has no tags or the comment lacks them.
fn summary(tags: Option<&DocSummary>, text: &str) -> Option<String> {
    let tags = tags?;
    let (_, rest) = text.split_once(tags.open.as_str())?;
    let (inner, _) = rest.split_once(tags.close.as_str())?;
    let words: Vec<&str> = inner.split_whitespace().collect();
    Some(words.join(WORD_SEPARATOR))
}

/// Removes the doc prefix; a comment without one is no documentation. A pack without prefixes
/// marks documentation through its query alone and keeps the text as written.
fn strip_prefix<'t>(prefixes: &[String], text: &'t str) -> Option<&'t str> {
    let text = text.trim_start();
    if prefixes.is_empty() {
        return Some(text);
    }
    prefixes
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags() -> DocSummary {
        DocSummary {
            open: "<s>".to_owned(),
            close: "</s>".to_owned(),
        }
    }

    #[test]
    fn summary_collapses_whitespace_between_the_tags() {
        assert_eq!(
            summary(Some(&tags()), "<s> a\n  b </s><r>x</r>").as_deref(),
            Some("a b")
        );
    }

    fn jsdoc() -> DocBlock {
        DocBlock {
            open: "/**".to_owned(),
            close: "*/".to_owned(),
            line_prefix: "*".to_owned(),
            stop_at: Some("@".to_owned()),
        }
    }

    #[test]
    fn block_text_drops_delimiters_markers_and_tags() {
        let text = "/**\n * First line.\n *\n * Second line.\n * @param x the value\n */";
        assert_eq!(block_text(&jsdoc(), text), "First line.\n\nSecond line.");
        assert_eq!(block_text(&jsdoc(), "/** One line. */"), "One line.");
        assert_eq!(block_text(&jsdoc(), "/** @internal */"), "");
        assert_eq!(
            block_text(&jsdoc(), "/**\n * **bold** text\n */"),
            "**bold** text"
        );
    }

    #[test]
    fn an_empty_block_comment_is_no_documentation() {
        assert!(!is_block(&jsdoc(), "/**/"));
        assert!(is_block(&jsdoc(), "/** */"));
    }

    #[test]
    fn empty_summary_is_empty_and_missing_tags_fall_back() {
        assert_eq!(summary(Some(&tags()), "<s> </s>").as_deref(), Some(""));
        assert_eq!(summary(Some(&tags()), "plain"), None);
        assert_eq!(summary(None, "<s>x</s>"), None);
    }
}
