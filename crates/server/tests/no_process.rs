//! The server starts no process (ADR 0071, decision 7): the window of the level 2 confirmation is
//! started by the adapter in the composition root, behind a port. Nothing in the source of this
//! crate may name the ways to start a program.
#![cfg(test)]

use std::fs;
use std::path::Path;

/// Ways to start a program, by the path or the type that does it.
const PROCESS_WORDS: [&str; 5] = [
    "std::process",
    "tokio::process",
    "process::Command",
    "Command::new",
    "CommandExt",
];

/// The line that opens the test module at the end of a source file.
const TEST_MODULE_MARK: &str = "#[cfg(test)]";

fn sources(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_server_crate_never_names_the_ways_to_start_a_process() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 10, "found too few files: {}", files.len());
    let offenders: Vec<_> = files
        .iter()
        .filter(|file| {
            let text = fs::read_to_string(file).unwrap();
            // Comment lines may talk about it, and the test modules at the end of a file may start
            // helper programs of their own; the product code before them may not.
            let code: String = text
                .lines()
                .take_while(|line| line.trim() != TEST_MODULE_MARK)
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect();
            PROCESS_WORDS.iter().any(|word| code.contains(word))
        })
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
}
