//! The grammars compiled into the binary (blueprint 8.2: full tier).

use tree_sitter::Language;

use crate::consts::{GRAMMAR_CSHARP, GRAMMAR_RUST};

/// The built-in grammar called `name`, if there is one.
#[must_use]
pub fn builtin(name: &str) -> Option<Language> {
    match name {
        GRAMMAR_RUST => Some(tree_sitter_rust::LANGUAGE.into()),
        GRAMMAR_CSHARP => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        _ => None,
    }
}
