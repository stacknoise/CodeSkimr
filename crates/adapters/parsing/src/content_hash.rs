//! Content hashes of symbols (ADR 0068): the token stream of a symbol's span, hashed with the
//! function that also hashes whole files (BLAKE3).
//!
//! The stream is the text of every leaf of the syntax tree in order, comments left out. White
//! space is no leaf, so it never reaches the hash: a reformatted symbol keeps its hash, and a symbol
//! whose identifier, literal or operator changed gets another. Only the text of a token goes in,
//! not the name of its node kind, so a new version of a grammar that names its nodes differently
//! does not turn every critical symbol into an alert. All language packs go through this one walk;
//! what a pack decides is which node kinds are comments (`comment_nodes`) and which are blocks that
//! the grammar does not delimit with tokens (`block_nodes`).
//!
//! # Limits
//!
//! - Where the grammar has no token for a block (Python: the block is the indentation), the walk
//!   puts a marker for the start and the end of every node kind that the pack lists in
//!   `block_nodes` into the stream, so moving a statement out of an `if` changes the hash. Only
//!   Python needs it of the packs that exist; every other pack delimits blocks with braces, `end`
//!   or similar tokens, which are in the stream. A pack with an indentation grammar that lists no
//!   `block_nodes` hashes without the structure of its blocks.
//! - Only the tokens count, not the kinds of nodes. Two token sequences that a grammar reads as
//!   different trees but that spell the same text have the same hash.
//! - A symbol hashes its own span only. In Rust and Go the methods of a type stand in `impl`
//!   blocks and with receivers outside the span of the type, so the type keeps its hash when one of
//!   them changes; the method has a hash of its own.

use std::ops::Range;

use codeskimr_application::ports::SymbolHash;
use codeskimr_domain::ContentHash;
use tree_sitter::Node;

use crate::consts::{BLOCK_CLOSE, BLOCK_OPEN, HASH_CHUNK_BYTES, TOKEN_SEPARATOR};
use crate::error::ParsingError;
use crate::pack::LanguagePack;
use crate::symbols::SymbolTable;

/// The hash of every type and member of the file that has one (`SymbolKind::has_content_hash`).
///
/// # Errors
/// [`ParsingError::Model`] if a hash is not a valid value, which is a bug.
pub fn of_symbols(
    pack: &LanguagePack,
    root: Node<'_>,
    source: &str,
    table: &SymbolTable,
) -> Result<Vec<SymbolHash>, ParsingError> {
    let tokens = token_ranges(pack, root);
    table
        .spans()
        .filter(|(symbol, _)| symbol.kind.has_content_hash())
        .map(|(symbol, range)| {
            Ok(SymbolHash {
                symbol: symbol.id,
                hash: hash_tokens(source, within(&tokens, range))?,
            })
        })
        .collect()
}

/// One element of the stream that is hashed.
#[derive(Clone)]
enum Token {
    /// The text of a leaf of the tree.
    Text(Range<usize>),
    /// The start of a block that no token delimits, at a byte.
    Open(usize),
    /// The end of such a block, at a byte.
    Close(usize),
}

impl Token {
    fn start(&self) -> usize {
        match self {
            Self::Text(range) => range.start,
            Self::Open(at) | Self::Close(at) => *at,
        }
    }

    fn end(&self) -> usize {
        match self {
            Self::Text(range) => range.end,
            Self::Open(at) | Self::Close(at) => *at,
        }
    }
}

/// What the walk needs to know about a node kind.
#[derive(Clone, Copy)]
struct KindFlags {
    comment: bool,
    block: bool,
}

/// The flags of the node kinds seen so far, looked up once per kind and not once per node.
struct KindCache<'p> {
    pack: &'p LanguagePack,
    flags: Vec<Option<KindFlags>>,
}

impl<'p> KindCache<'p> {
    fn new(pack: &'p LanguagePack) -> Self {
        Self {
            pack,
            flags: Vec::new(),
        }
    }

    fn of(&mut self, node: Node<'_>) -> KindFlags {
        let id = usize::from(node.kind_id());
        if self.flags.len() <= id {
            self.flags.resize(id.saturating_add(1), None);
        }
        let pack = self.pack;
        match self.flags.get_mut(id) {
            Some(Some(flags)) => *flags,
            Some(slot) => {
                let flags = KindFlags {
                    comment: pack.comment_nodes.contains(node.kind()),
                    block: pack.block_nodes.contains(node.kind()),
                };
                *slot = Some(flags);
                flags
            }
            None => KindFlags {
                comment: false,
                block: false,
            },
        }
    }
}

/// The leaves of the tree that are no comments, in source order, with the markers of the blocks
/// that the pack names.
fn token_ranges(pack: &LanguagePack, root: Node<'_>) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut kinds = KindCache::new(pack);
    let mut cursor = root.walk();
    loop {
        let current = cursor.node();
        let flags = kinds.of(current);
        let is_leaf = flags.comment || current.child_count() == 0;
        if flags.block {
            tokens.push(Token::Open(current.start_byte()));
        }
        if is_leaf && !flags.comment && current.end_byte() > current.start_byte() {
            tokens.push(Token::Text(current.start_byte()..current.end_byte()));
        }
        if !is_leaf && cursor.goto_first_child() {
            continue;
        }
        if flags.block {
            tokens.push(Token::Close(current.end_byte()));
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return tokens;
            }
            if kinds.of(cursor.node()).block {
                tokens.push(Token::Close(cursor.node().end_byte()));
            }
        }
    }
}

/// The tokens that lie inside `range`; `tokens` is sorted by start.
fn within<'t>(tokens: &'t [Token], range: &Range<usize>) -> &'t [Token] {
    let first = tokens.partition_point(|token| token.start() < range.start);
    let rest = tokens.get(first..).unwrap_or_default();
    let count = rest.partition_point(|token| token.end() <= range.end);
    rest.get(..count).unwrap_or_default()
}

fn hash_tokens(source: &str, tokens: &[Token]) -> Result<ContentHash, ParsingError> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(HASH_CHUNK_BYTES.saturating_mul(2));
    for token in tokens {
        match token {
            Token::Text(range) => {
                buffer.extend_from_slice(source.as_bytes().get(range.clone()).unwrap_or_default());
            }
            Token::Open(_) => buffer.push(BLOCK_OPEN),
            Token::Close(_) => buffer.push(BLOCK_CLOSE),
        }
        buffer.push(TOKEN_SEPARATOR);
        if buffer.len() >= HASH_CHUNK_BYTES {
            hasher.update(&buffer);
            buffer.clear();
        }
    }
    hasher.update(&buffer);
    Ok(ContentHash::new(hasher.finalize().to_hex().to_string())?)
}
