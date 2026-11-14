//! Level 2 may be confirmed or revoked only by the interactive command of the composition root
//! (ADR 0033, decision 10). This is the second layer: the port already refuses a write without a
//! `ConsentGrant`, which only `level2::confirm` and `level2::revoke` can make. Here no other
//! product file may name those functions, the grant or `store_consent`, in any spelling: a path,
//! a `use` (also in braces, over several lines, with an alias or a glob) or a re-export.

#[cfg(test)]
mod product;

use std::fs;
use std::path::PathBuf;

/// Names that only the allowed files may use, compared after all white space is removed.
const FORBIDDEN: [&str; 7] = [
    "level2::confirm",
    "level2::revoke",
    "level2::*",
    "level2as",
    "store_consent",
    "ConsentGrant",
    "ConsentWriter",
];
/// The words a `use level2::{...}` may not import.
const IMPORTS: [&str; 2] = ["confirm", "revoke"];
/// The definitions, the re-exports, the storage adapter, the one interactive command and its
/// tests (compiled only for tests).
const ALLOWED: [&str; 6] = [
    "app/src/level2_consent.rs",
    "app/src/level2_consent_tests.rs",
    "application/src/level2/mod.rs",
    "application/src/ports/consent.rs",
    "application/src/ports/mod.rs",
    "storage/src/consent.rs",
];

/// The code of a file without its comment lines and without white space.
fn squeezed(text: &str) -> String {
    let mut code = String::new();
    let mut in_block = false;
    for line in text.lines() {
        let line = line.trim();
        if in_block || line.starts_with("/*") {
            in_block = !line.contains("*/");
        } else if !line.starts_with("//") {
            code.extend(line.chars().filter(|c| !c.is_whitespace()));
        }
    }
    code
}

/// The text of every `level2::{ ... }` import in `code`.
fn braces(code: &str) -> Vec<&str> {
    code.match_indices("level2::{")
        .filter_map(|(at, open)| {
            let rest = &code[at + open.len()..];
            rest.find('}').map(|end| &rest[..end])
        })
        .collect()
}

/// Whether `text` names the way to a consent write.
fn reaches_consent(text: &str) -> bool {
    let code = squeezed(text);
    FORBIDDEN.iter().any(|name| code.contains(name))
        || braces(&code).iter().any(|list| {
            list.split(',').any(|item| {
                IMPORTS
                    .iter()
                    .any(|word| item.split("as").next() == Some(word))
            })
        })
}

#[test]
fn the_check_sees_every_spelling_of_the_way_in() {
    let spellings = [
        "level2::confirm(&p, s, id)",
        "use codeskimr_application::level2::revoke;",
        "use codeskimr_application::level2::{\n    authorize_run,\n    confirm as c,\n};",
        "use codeskimr_application::level2 as l;",
        "use codeskimr_application::level2::*;",
        "pub use crate::level2::{ConsentGrant, RunPermit};",
        "level2 :: \n  confirm(",
        "store.store_consent(&grant)",
    ];
    for spelling in spellings {
        assert!(reaches_consent(spelling), "not seen: {spelling}");
    }
    let harmless = [
        "// level2::confirm(&p, s, id)",
        "/* store_consent\n*/ let a = 1;",
        "use codeskimr_application::level2::{authorize_run, RunPermit};",
        "let confirmation = 1;",
    ];
    for text in harmless {
        assert!(!reaches_consent(text), "wrongly seen: {text}");
    }
}

#[test]
fn only_the_interactive_command_confirms_level_two() {
    let files = product::product_sources();
    assert!(files.len() > 50, "found too few files: {}", files.len());
    let offenders: Vec<_> = files
        .iter()
        .filter(|file| !ALLOWED.iter().any(|allowed| file.ends_with(allowed)))
        .filter(|file| reaches_consent(&fs::read_to_string(file).unwrap()))
        .collect();
    assert_eq!(offenders, Vec::<&PathBuf>::new());
}
