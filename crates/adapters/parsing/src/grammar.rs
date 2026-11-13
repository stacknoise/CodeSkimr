//! The grammars compiled into the binary: the full tier always, the basic tier behind the cargo
//! feature `basic-grammars` (blueprint 8.2, ADR 0020).

use tree_sitter::Language;

use crate::consts::{
    GRAMMAR_CSHARP, GRAMMAR_JAVA, GRAMMAR_JAVASCRIPT, GRAMMAR_KOTLIN, GRAMMAR_PYTHON, GRAMMAR_RUST,
    GRAMMAR_TSX, GRAMMAR_TYPESCRIPT,
};

/// The built-in grammar called `name`, if there is one.
#[must_use]
pub fn builtin(name: &str) -> Option<Language> {
    match name {
        GRAMMAR_RUST => Some(tree_sitter_rust::LANGUAGE.into()),
        GRAMMAR_CSHARP => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        GRAMMAR_TYPESCRIPT => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        GRAMMAR_TSX => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        GRAMMAR_JAVASCRIPT => Some(tree_sitter_javascript::LANGUAGE.into()),
        GRAMMAR_PYTHON => Some(tree_sitter_python::LANGUAGE.into()),
        GRAMMAR_JAVA => Some(tree_sitter_java::LANGUAGE.into()),
        GRAMMAR_KOTLIN => Some(tree_sitter_kotlin_ng::LANGUAGE.into()),
        _ => basic(name),
    }
}

#[cfg(feature = "basic-grammars")]
fn basic(name: &str) -> Option<Language> {
    use crate::consts::{GRAMMAR_C, GRAMMAR_CPP, GRAMMAR_GO, GRAMMAR_PHP, GRAMMAR_RUBY};
    match name {
        GRAMMAR_GO => Some(tree_sitter_go::LANGUAGE.into()),
        GRAMMAR_C => Some(tree_sitter_c::LANGUAGE.into()),
        GRAMMAR_CPP => Some(tree_sitter_cpp::LANGUAGE.into()),
        GRAMMAR_RUBY => Some(tree_sitter_ruby::LANGUAGE.into()),
        GRAMMAR_PHP => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        _ => None,
    }
}

#[cfg(not(feature = "basic-grammars"))]
const fn basic(_name: &str) -> Option<Language> {
    None
}
