//! Syntax fingerprints of callable symbols (blueprint 10.4): the tree is flattened into a
//! normalised token stream and handed to the domain's winnowing.

use std::ops::Range;

use codeskimr_application::ports::SymbolFingerprints;
use codeskimr_domain::{FingerprintParams, TokenId, winnow};
use tree_sitter::Node;

use crate::pack::LanguagePack;
use crate::symbols::SymbolTable;

/// How the analyzer fingerprints: the winnowing parameters and the shortest body that counts.
#[derive(Debug, Clone, Copy)]
pub struct FingerprintSettings {
    /// N-gram and window size.
    pub params: FingerprintParams,
    /// Bodies with fewer tokens get no fingerprints.
    pub min_tokens: usize,
}

/// Fingerprints every callable of the file that has at least `min_tokens` syntax tokens.
#[must_use]
pub fn of_callables(
    pack: &LanguagePack,
    root: Node<'_>,
    table: &SymbolTable,
    settings: FingerprintSettings,
) -> Vec<SymbolFingerprints> {
    table
        .callables()
        .filter_map(|(symbol, range)| {
            let tokens = tokens_in(pack, root, range);
            if tokens.len() < settings.min_tokens {
                return None;
            }
            let fingerprints = winnow(&tokens, settings.params);
            (!fingerprints.is_empty()).then_some(SymbolFingerprints {
                symbol: symbol.id,
                fingerprints,
            })
        })
        .collect()
}

/// The normalised tokens of the smallest node that covers `range`: comments are dropped,
/// identifiers and literals are abstracted, everything else keeps its node kind.
fn tokens_in(pack: &LanguagePack, root: Node<'_>, range: &Range<usize>) -> Vec<TokenId> {
    let Some(node) = root.descendant_for_byte_range(range.start, range.end) else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    let mut cursor = node.walk();
    loop {
        let current = cursor.node();
        let kind = current.kind();
        let is_comment = pack.comment_nodes.contains(kind);
        let is_literal = pack.literal_nodes.contains(kind);
        let is_leaf = is_comment || is_literal || current.child_count() == 0;
        if is_leaf && !is_comment && current.end_byte() > current.start_byte() {
            tokens.push(token_of(pack, kind, is_literal));
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

fn token_of(pack: &LanguagePack, kind: &str, is_literal: bool) -> TokenId {
    if is_literal {
        TokenId::literal()
    } else if pack.identifier_nodes.contains(kind) {
        TokenId::identifier()
    } else {
        TokenId::node_kind(kind)
    }
}
