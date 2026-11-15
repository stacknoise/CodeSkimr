//! The body of a `macro_rules!` is checked where the macro is defined: the crate that defines it
//! never expands it, and the crates that use it see it as foreign code.

use rustc_ast::token::{Delimiter, Token, TokenKind};
use rustc_ast::tokenstream::{TokenStream, TokenTree};
use rustc_span::{Span, kw, sym};

use crate::allowed::is_free_literal;

/// Built-in macros whose string arguments name a variable, a file or a message, not a value.
const SYNTAX_MACROS: [&str; 6] = [
    "env",
    "option_env",
    "compile_error",
    "include",
    "include_str",
    "include_bytes",
];

/// Calls `report` for every literal on the transcriber side of the rules (right of `=>`).
pub fn check(body: &TokenStream, report: &mut dyn FnMut(Span)) {
    let trees: Vec<&TokenTree> = body.iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        let after_arrow = index > 0 && is_token(trees[index - 1], &TokenKind::FatArrow);
        if let (true, TokenTree::Delimited(_, _, _, transcriber)) = (after_arrow, tree) {
            if !generates_tests(transcriber) {
                check_tokens(transcriber, report);
            }
        }
    }
}

fn check_tokens(tokens: &TokenStream, report: &mut dyn FnMut(Span)) {
    let trees: Vec<&TokenTree> = tokens.iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        match tree {
            TokenTree::Token(token, _) => {
                if is_value_literal(token) && !follows_extern(&trees, index) {
                    report(token.span);
                }
            }
            TokenTree::Delimited(_, _, _, inner) => {
                if !is_attribute(&trees, index) && !is_syntax_macro_call(&trees, index) {
                    check_tokens(inner, report);
                }
            }
        }
    }
}

fn is_token(tree: &TokenTree, kind: &TokenKind) -> bool {
    matches!(tree, TokenTree::Token(token, _) if token.kind == *kind)
}

fn is_value_literal(token: &Token) -> bool {
    matches!(&token.kind, TokenKind::Literal(lit) if !is_free_literal(lit))
}

/// `extern "C"`: the ABI string is syntax.
fn follows_extern(trees: &[&TokenTree], index: usize) -> bool {
    index > 0
        && matches!(trees[index - 1], TokenTree::Token(token, _)
            if token.is_ident_named(kw::Extern))
}

/// The bracket group of `#[..]` or `#![..]`.
fn is_attribute(trees: &[&TokenTree], index: usize) -> bool {
    let is_bracket = matches!(trees[index], TokenTree::Delimited(_, _, Delimiter::Bracket, _));
    let after_pound = index > 0 && is_token(trees[index - 1], &TokenKind::Pound);
    let after_inner = index > 1
        && is_token(trees[index - 1], &TokenKind::Bang)
        && is_token(trees[index - 2], &TokenKind::Pound);
    is_bracket && (after_pound || after_inner)
}

/// The argument group of `env!(..)`, `compile_error!(..)` and the like.
fn is_syntax_macro_call(trees: &[&TokenTree], index: usize) -> bool {
    index > 1
        && is_token(trees[index - 1], &TokenKind::Bang)
        && matches!(trees[index - 2], TokenTree::Token(token, _)
            if token.ident().is_some_and(|(ident, _)| {
                SYNTAX_MACROS.contains(&ident.name.as_str())
            }))
}

/// A macro that writes `#[test]` or `#[cfg(test)]` into its output makes test code.
fn generates_tests(tokens: &TokenStream) -> bool {
    let trees: Vec<&TokenTree> = tokens.iter().collect();
    trees.iter().enumerate().any(|(index, tree)| match tree {
        TokenTree::Delimited(_, _, _, inner) => {
            (is_attribute(&trees, index) && mentions_test(inner)) || generates_tests(inner)
        }
        TokenTree::Token(..) => false,
    })
}

fn mentions_test(tokens: &TokenStream) -> bool {
    tokens.iter().any(|tree| match tree {
        TokenTree::Token(token, _) => token.is_ident_named(sym::test),
        TokenTree::Delimited(_, _, _, inner) => mentions_test(inner),
    })
}
