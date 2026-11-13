//! Dylint lint `no_literals`: blueprint rule R-06 (section 3.2).
//!
//! Production code must not contain literals. Allowed are the integers `0` and `1`, booleans,
//! test code, the value side of `const`/`static` items in a `consts.rs` module and build scripts.
#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_span;

mod allowed;

use rustc_ast::{Attribute, Expr, ExprKind, FormatArgsPiece, Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;

dylint_linting::impl_early_lint! {
    /// ### What it does
    ///
    /// Reports every literal (numbers other than `0` and `1`, strings, characters, byte strings,
    /// floats) and every format string with fixed text in production code. Rule R-06, error
    /// code `CHK-101`.
    ///
    /// ### Why is this bad?
    ///
    /// Values belong in `consts.rs`, `config/default.toml`, `locales/`, `errors/error-codes.toml`
    /// or `languages/`, so that they exist exactly once and can be found and changed.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let limit = 4096;
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// let limit = BUFFER_SIZE;
    /// ```
    pub NO_LITERALS,
    Deny,
    "literal in production code (R-06, CHK-101)",
    NoLiterals::default()
}

/// State of the pass: how deeply the walk is inside exempt items.
#[derive(Default)]
struct NoLiterals {
    /// Open test items (`#[cfg(test)]`, `#[test]`).
    test_depth: u32,
    /// Open `const`/`static` items inside a `consts.rs` module.
    const_depth: u32,
    /// The attribute being walked: `#[doc = ".."]`, `#[path = ".."]` and friends are not value literals.
    attribute: Option<Span>,
}

impl NoLiterals {
    fn exempt(&self) -> bool {
        self.test_depth > 0 || self.const_depth > 0
    }

    fn report(cx: &EarlyContext<'_>, span: Span) {
        if allowed::in_exempt_file(cx, span) || allowed::is_doc_comment(cx, span) {
            return;
        }
        let decorate = |diag: &mut rustc_errors::Diag<'_, ()>| {
            diag.primary_message("literal in production code (CHK-101)");
            diag.help(
                "move the value to `consts.rs`, `config/default.toml`, `locales/`, \
                 `errors/error-codes.toml` or `languages/` (blueprint 3.2)",
            );
        };
        cx.opt_span_lint(
            NO_LITERALS,
            Some(span),
            rustc_errors::DiagDecorator(decorate),
        );
    }
}

impl EarlyLintPass for NoLiterals {
    fn check_attribute(&mut self, _cx: &EarlyContext<'_>, attr: &Attribute) {
        self.attribute = Some(attr.span);
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if allowed::is_test_item(&item.attrs) {
            self.test_depth += 1;
        }
        let is_value = matches!(item.kind, ItemKind::Const(_) | ItemKind::Static(_));
        if is_value && allowed::in_consts_module(cx, item.span) {
            self.const_depth += 1;
        }
    }

    fn check_item_post(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if allowed::is_test_item(&item.attrs) {
            self.test_depth -= 1;
        }
        let is_value = matches!(item.kind, ItemKind::Const(_) | ItemKind::Static(_));
        if is_value && allowed::in_consts_module(cx, item.span) {
            self.const_depth -= 1;
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if self.exempt() || self.attribute.is_some_and(|attr| attr.contains(expr.span)) {
            return;
        }
        match &expr.kind {
            ExprKind::Lit(lit) if !allowed::is_free_literal(lit) => {
                if allowed::own_source(expr.span) && allowed::is_written_literal(cx, expr.span) {
                    Self::report(cx, expr.span);
                }
            }
            ExprKind::FormatArgs(args) => {
                let has_text = args
                    .template
                    .iter()
                    .any(|piece| matches!(piece, FormatArgsPiece::Literal(_)));
                if has_text {
                    if let Some(span) = allowed::macro_call_in_own_source(args.span) {
                        Self::report(cx, span);
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
