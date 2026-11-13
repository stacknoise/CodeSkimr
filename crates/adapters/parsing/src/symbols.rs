//! Extraction of symbols from a syntax tree with the declaration query of a language pack.

use std::cmp::Reverse;
use std::ops::Range;
use std::path::Path;

use codeskimr_application::ids::IdSequence;
use codeskimr_domain::{
    FileId, QualifiedName, RelativePath, Span, Symbol, SymbolId, SymbolKind, SymbolName, Visibility,
};
use tree_sitter::{Node, QueryCapture, QueryCursor, StreamingIterator};

use crate::captures::Capture;
use crate::consts::{LINE_SEPARATOR, PATH_SEPARATOR};
use crate::error::ParsingError;
use crate::location::{span_of, text_of};
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
}

impl SymbolTable {
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
}

struct Declaration<'t> {
    node: Node<'t>,
    kind: SymbolKind,
    name: Node<'t>,
    visibility: Option<Node<'t>>,
    is_static: bool,
    docs: Vec<Node<'t>>,
    signature: Option<Node<'t>>,
}

#[derive(Default)]
struct Parts<'t> {
    declaration: Option<(Node<'t>, SymbolKind)>,
    name: Option<Node<'t>>,
    visibility: Option<Node<'t>>,
    is_static: bool,
    docs: Vec<Node<'t>>,
    signature: Option<Node<'t>>,
}

impl<'t> Parts<'t> {
    fn add(&mut self, capture: Capture, node: Node<'t>) {
        match capture {
            Capture::Symbol(kind) => self.declaration = Some((node, kind)),
            Capture::Name => self.name = Some(node),
            Capture::Visibility => self.visibility = Some(node),
            Capture::Static => self.is_static = true,
            Capture::Doc => self.docs.push(node),
            Capture::Signature => self.signature = Some(node),
            Capture::Reference(_) => {}
        }
    }

    fn finish(self) -> Option<Declaration<'t>> {
        let (node, kind) = self.declaration?;
        Some(Declaration {
            node,
            kind,
            name: self.name?,
            visibility: self.visibility,
            is_static: self.is_static,
            docs: self.docs,
            signature: self.signature,
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
    // Several patterns may describe the same declaration; the first one wins.
    found.dedup_by_key(|d| (d.node.id(), d.kind));
    found
}

struct Frame {
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
}

struct Builder<'a, 'c> {
    ctx: &'c SymbolContext<'a>,
    symbols: Vec<Symbol>,
    ranges: Vec<Range<usize>>,
    parents: Vec<Option<usize>>,
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
    })
}

impl Builder<'_, '_> {
    fn add_module(&mut self, root: Node<'_>) -> Result<(), ParsingError> {
        let path = Path::new(self.ctx.path.as_str());
        let qualified = path
            .with_extension("")
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
        self.push(
            qualified,
            NewSymbol {
                kind: declaration.kind,
                name,
                span: span_of(declaration.node)?,
                range: declaration.node.byte_range(),
                visibility: visibility(ctx, declaration),
                is_static: declaration.is_static,
                signature: declaration
                    .signature
                    .map(|node| text_of(node, ctx.source).trim().to_owned()),
                doc: doc(ctx, &declaration.docs),
            },
        )
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
        self.stack.push(Frame {
            end: new.range.end,
            index,
            id,
            qualified,
        });
        self.ranges.push(new.range);
        Ok(())
    }
}

fn visibility(ctx: &SymbolContext<'_>, declaration: &Declaration<'_>) -> Option<Visibility> {
    declaration
        .visibility
        .and_then(|node| ctx.pack.visibility.get(text_of(node, ctx.source).trim()))
        .copied()
        .or(ctx.pack.default_visibility)
}

fn doc(ctx: &SymbolContext<'_>, nodes: &[Node<'_>]) -> Option<String> {
    let lines: Vec<&str> = nodes
        .iter()
        .filter_map(|node| strip_prefix(&ctx.pack.doc_prefixes, text_of(*node, ctx.source)))
        .map(str::trim)
        .collect();
    (!lines.is_empty()).then(|| lines.join(LINE_SEPARATOR))
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
