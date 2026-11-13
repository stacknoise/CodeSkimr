//! The exceptions of blueprint rule R-06 and the span helpers that decide whether a literal
//! belongs to the code under check or to a foreign macro expansion.

use rustc_ast::Attribute;
use rustc_ast::token::{Lit, LitKind};
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::hygiene::{ExpnKind, MacroKind};
use rustc_span::{FileName, Span, sym};

/// File of the constants module, whose `const`/`static` values may hold literals.
const CONSTS_FILE: &str = "consts.rs";
/// Build scripts generate code and may hold literals.
const BUILD_SCRIPT: &str = "build.rs";
/// Characters that open a string or character literal.
const QUOTES: [char; 2] = ['"', '\''];
/// Prefixes of byte, raw and C string literals.
const LITERAL_PREFIXES: [char; 3] = ['b', 'r', 'c'];
/// Second character of a raw string prefix.
const RAW_MARKER: char = '#';
/// Start of a doc comment in the source text.
const DOC_MARKERS: [&str; 4] = ["///", "//!", "/**", "/*!"];
/// Cargo sets this for crates with a build script: the folder of the generated code.
const GENERATED_DIR_VAR: &str = "OUT_DIR";
/// Top-level directory of developer tooling (code generator, CI scripts, lints): not product code.
const TOOLS_DIR: &str = "tools";
/// Directories that only contain test or benchmark code.
const TEST_DIRS: [&str; 2] = ["tests", "benches"];
/// Marks the root folder of a crate.
const MANIFEST: &str = "Cargo.toml";

/// `0` and `1` (any integer suffix), booleans and error tokens are always allowed.
pub fn is_free_literal(lit: &Lit) -> bool {
    match lit.kind {
        LitKind::Bool | LitKind::Err(_) => true,
        LitKind::Integer => matches!(lit.symbol.as_str(), "0" | "1"),
        _ => false,
    }
}

/// Doc comments become `#[doc = ".."]` literals; their source text still starts with the comment marker.
pub fn is_doc_comment(cx: &EarlyContext<'_>, span: Span) -> bool {
    cx.sess()
        .source_map()
        .span_to_snippet(span)
        .is_ok_and(|text| DOC_MARKERS.iter().any(|marker| text.starts_with(marker)))
}

/// Derives such as serde reuse the span of a field name for the strings they generate. A real
/// literal in the source starts with a digit, a quote or a prefix (`b`, `r`, `c`) followed by a quote.
pub fn is_written_literal(cx: &EarlyContext<'_>, span: Span) -> bool {
    let Ok(text) = cx.sess().source_map().span_to_snippet(span) else {
        return false;
    };
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(first), _) if first.is_ascii_digit() || QUOTES.contains(&first) => true,
        (Some(prefix), Some(next)) if LITERAL_PREFIXES.contains(&prefix) => {
            QUOTES.contains(&next) || next == RAW_MARKER
        }
        _ => false,
    }
}

/// `#[test]` functions and `#[cfg(test)]` items are test code.
pub fn is_test_item(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.has_name(sym::test)
            || (attr.has_name(sym::cfg)
                && attr
                    .meta_item_list()
                    .is_some_and(|list| list.iter().any(|nested| nested.has_name(sym::test))))
    })
}

fn path_of(cx: &EarlyContext<'_>, span: Span) -> Option<std::path::PathBuf> {
    match cx.sess().source_map().span_to_filename(span) {
        FileName::Real(real) => real.local_path().map(std::path::Path::to_path_buf),
        _ => None,
    }
}

fn file_name_is(cx: &EarlyContext<'_>, span: Span, wanted: &str) -> bool {
    path_of(cx, span)
        .and_then(|path| path.file_name().map(|name| name == wanted))
        .unwrap_or(false)
}

/// The item lives in a `consts.rs` module.
pub fn in_consts_module(cx: &EarlyContext<'_>, span: Span) -> bool {
    file_name_is(cx, span, CONSTS_FILE)
}

/// Build scripts, developer tooling below `tools/` (relative path), generated code in `OUT_DIR` and files in `tests/` or `benches/` directly below a crate root are exempt as a whole.
pub fn in_exempt_file(cx: &EarlyContext<'_>, span: Span) -> bool {
    if file_name_is(cx, span, BUILD_SCRIPT) {
        return true;
    }
    path_of(cx, span).is_some_and(|path| {
        let generated = std::env::var_os(GENERATED_DIR_VAR)
            .is_some_and(|dir| path.starts_with(std::path::Path::new(&dir)));
        generated
            || path.starts_with(TOOLS_DIR)
            || is_test_dir_path(&path, |dir| dir.join(MANIFEST).is_file())
    })
}

/// A file is test code when it lies in `tests/` or `benches/` directly below a crate root.
/// A module that is merely named `tests` or `examples` inside `src/` does not count.
fn is_test_dir_path(
    path: &std::path::Path,
    is_crate_root: impl Fn(&std::path::Path) -> bool,
) -> bool {
    path.ancestors().any(|dir| {
        let is_test_dir = dir
            .file_name()
            .is_some_and(|name| TEST_DIRS.iter().any(|wanted| name == *wanted));
        is_test_dir && dir.parent().is_some_and(&is_crate_root)
    })
}

#[cfg(test)]
mod tests {
    use super::is_test_dir_path;
    use std::path::Path;

    fn root(dir: &Path) -> bool {
        dir == Path::new("crates/domain")
    }

    #[test]
    fn tests_directory_below_crate_root_is_exempt() {
        assert!(is_test_dir_path(
            Path::new("crates/domain/tests/model.rs"),
            root
        ));
        assert!(is_test_dir_path(
            Path::new("crates/domain/benches/scan/a.rs"),
            root
        ));
    }

    #[test]
    fn modules_with_test_like_names_are_not_exempt() {
        assert!(!is_test_dir_path(
            Path::new("crates/domain/src/tests/mod.rs"),
            root
        ));
        assert!(!is_test_dir_path(
            Path::new("crates/domain/src/examples/mod.rs"),
            root
        ));
    }
}

/// The literal comes out of `concat!`: the call site in the checked code, if it is there.
pub fn concat_call_in_own_source(span: Span) -> Option<Span> {
    if !span.from_expansion() {
        return None;
    }
    let data = span.ctxt().outer_expn_data();
    match data.kind {
        ExpnKind::Macro(MacroKind::Bang, name) if name == sym::concat => {
            macro_call_in_own_source(data.call_site)
        }
        _ => None,
    }
}

/// A literal belongs to the checked code when it was written there, or inside a macro defined
/// in the same crate. Literals produced by foreign macros (derives, `std`) do not count.
pub fn own_source(span: Span) -> bool {
    if span.is_empty() {
        return false;
    }
    if !span.from_expansion() {
        return true;
    }
    span.ctxt()
        .outer_expn_data()
        .macro_def_id
        .is_some_and(|def| def.is_local())
}

/// For a format string: the span of the call (`format!`, `println!`, ...) in the checked code.
/// Walks outwards through foreign function-like macros and gives up at derives and attributes.
pub fn macro_call_in_own_source(span: Span) -> Option<Span> {
    let mut current = span;
    while current.from_expansion() {
        let data = current.ctxt().outer_expn_data();
        if data.macro_def_id.is_some_and(|def| def.is_local()) {
            return Some(current);
        }
        match data.kind {
            ExpnKind::Macro(MacroKind::Bang, _) => current = data.call_site,
            _ => return None,
        }
    }
    Some(current)
}
