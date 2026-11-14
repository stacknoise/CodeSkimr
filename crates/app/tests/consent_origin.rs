//! Level 2 may be confirmed or revoked only by the interactive command of the composition root
//! (ADR 0033, decision 10): no other product file may call `level2::confirm`, `level2::revoke` or
//! `store_consent`, so neither MCP, nor the server, nor a scan, nor watch mode can grant it.

#[cfg(test)]
mod product;

use std::fs;
use std::path::{Path, PathBuf};

const CALLS: [&str; 3] = ["level2::confirm(", "level2::revoke(", "store_consent("];
/// The definitions (also the test fake), the storage adapter and the one interactive command.
const ALLOWED: [&str; 5] = [
    "app/src/level2_consent.rs",
    "application/src/level2/mod.rs",
    "application/src/level2/testing.rs",
    "application/src/ports/consent.rs",
    "storage/src/consent.rs",
];

/// Code lines only: a doc comment may name the functions.
#[cfg(test)]
fn calls_consent(file: &Path) -> bool {
    fs::read_to_string(file).unwrap().lines().any(|line| {
        !line.trim_start().starts_with("//") && CALLS.iter().any(|call| line.contains(call))
    })
}

#[test]
fn only_the_interactive_command_confirms_level_two() {
    let files = product::product_sources();
    assert!(files.len() > 50, "found too few files: {}", files.len());
    let offenders: Vec<_> = files
        .iter()
        .filter(|file| !ALLOWED.iter().any(|allowed| file.ends_with(allowed)))
        .filter(|file| calls_consent(file))
        .collect();
    assert_eq!(offenders, Vec::<&PathBuf>::new());
}
