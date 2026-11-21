//! Finds registered error codes that no product code uses (blueprint rule E-02).
//!
//! A code counts as used when production Rust names its enum variant (`ErrorCode::Variant`) or
//! production TypeScript names its id (`"UI-901"`). Tests, generated files and this generator do
//! not count: a code that only a test mentions is still dead. Retired codes are skipped.

use std::path::{Path, PathBuf};

use crate::consts::{
    GENERATED_MARKER, SCANNED_DIRS, SKIPPED_DIRS, SKIPPED_FILE_PARTS, SOURCE_EXTENSIONS,
    VARIANT_PATH,
};
use crate::registry::{Code, Registry};

/// One source file: its path relative to the repository root and its text.
pub type Source = (String, String);

/// Reads every production source file under the repository `root`.
///
/// # Errors
/// Returns one message per folder or file that cannot be read.
pub fn collect_sources(root: &Path) -> Result<Vec<Source>, Vec<String>> {
    let mut sources = Vec::new();
    let mut problems = Vec::new();
    for dir in SCANNED_DIRS {
        walk(root, &root.join(dir), &mut sources, &mut problems);
    }
    if problems.is_empty() {
        Ok(sources)
    } else {
        Err(problems)
    }
}

/// Adds the production sources below `dir` to `sources`.
fn walk(root: &Path, dir: &Path, sources: &mut Vec<Source>, problems: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            problems.push(format!("{}: {e}", dir.display()));
            return;
        }
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            if !is_skipped_dir(&path) {
                walk(root, &path, sources, problems);
            }
        } else if is_source(&path) {
            read_source(root, &path, sources, problems);
        }
    }
}

/// `true` for folders that hold no production code (build output, dependencies, tests).
fn is_skipped_dir(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    SKIPPED_DIRS.contains(&name)
}

/// `true` for a source file that can use a code and is neither a test nor generated.
fn is_source(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    SOURCE_EXTENSIONS.contains(&extension)
        && !SKIPPED_FILE_PARTS.iter().any(|part| name.contains(part))
}

/// Reads one file; a generated file is left out.
fn read_source(root: &Path, path: &Path, sources: &mut Vec<Source>, problems: &mut Vec<String>) {
    match std::fs::read_to_string(path) {
        Ok(text) if text.contains(GENERATED_MARKER) => {}
        Ok(text) => {
            let rel = path.strip_prefix(root).unwrap_or(path);
            sources.push((rel.display().to_string(), text));
        }
        Err(e) => problems.push(format!("{}: {e}", path.display())),
    }
}

/// `true` when `text` contains `needle` and the next character does not continue an identifier.
fn names_whole(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(at, _)| {
        text[at + needle.len()..]
            .chars()
            .next()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
    })
}

/// `true` when one of the sources uses the code.
fn is_used(code: &Code, sources: &[Source]) -> bool {
    let variant = format!("{VARIANT_PATH}{}", code.variant());
    let quoted = format!("\"{}\"", code.id);
    sources
        .iter()
        .any(|(_, text)| names_whole(text, &variant) || text.contains(&quoted))
}

/// One message per code that is registered, not retired and used nowhere in `sources`.
#[must_use]
pub fn unused_codes(registry: &Registry, sources: &[Source]) -> Vec<String> {
    registry
        .codes
        .iter()
        .filter(|code| !code.deprecated && !is_used(code, sources))
        .map(|code| {
            format!(
                "{}: code is not used anywhere, use it or retire it with `deprecated = true`",
                code.id
            )
        })
        .collect()
}
