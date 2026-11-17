//! Content hashes of symbols (ADR 0068): the token stream of a symbol's span, hashed with the
//! function that also hashes whole files (BLAKE3).
//!
//! The stream is the text of every leaf of the syntax tree in order, comments left out. White
//! space is no leaf, so it never reaches the hash: a reformatted symbol keeps its hash, and a symbol
//! whose identifier, literal or operator changed gets another. Only the text of a token goes in,
//! not the name of its node kind, so a new version of a grammar that names its nodes differently
//! does not turn every critical symbol into an alert. All language packs go through this one walk;
//! what a pack decides is only which node kinds are comments (`comment_nodes`).

use std::ops::Range;

use codeskimr_application::ports::SymbolHash;
use codeskimr_domain::ContentHash;
use tree_sitter::Node;

use crate::consts::TOKEN_SEPARATOR;
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

/// The byte ranges of the leaves of the tree that are no comments, in source order.
fn token_ranges(pack: &LanguagePack, root: Node<'_>) -> Vec<Range<usize>> {
    let mut tokens = Vec::new();
    let mut cursor = root.walk();
    loop {
        let current = cursor.node();
        let is_comment = pack.comment_nodes.contains(current.kind());
        let is_leaf = is_comment || current.child_count() == 0;
        if is_leaf && !is_comment && current.end_byte() > current.start_byte() {
            tokens.push(current.start_byte()..current.end_byte());
        }
        if !is_leaf && cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return tokens;
            }
        }
    }
}

/// The tokens that lie inside `range`; `tokens` is sorted by start.
fn within<'t>(tokens: &'t [Range<usize>], range: &Range<usize>) -> &'t [Range<usize>] {
    let first = tokens.partition_point(|token| token.start < range.start);
    let rest = tokens.get(first..).unwrap_or_default();
    let count = rest.partition_point(|token| token.end <= range.end);
    rest.get(..count).unwrap_or_default()
}

fn hash_tokens(source: &str, tokens: &[Range<usize>]) -> Result<ContentHash, ParsingError> {
    let mut hasher = blake3::Hasher::new();
    for token in tokens {
        hasher.update(source.as_bytes().get(token.clone()).unwrap_or_default());
        hasher.update(&[TOKEN_SEPARATOR]);
    }
    Ok(ContentHash::new(hasher.finalize().to_hex().to_string())?)
}
