//! `BoundServer::serve_as_peer` is a seam for tests (issue #531): it sets the peer address that
//! the loopback check of the confirmation route looks at. No crate may call it in its program
//! code, or a caller could claim to be somebody else.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// The name of the seam, built so that this file does not contain it as one word.
fn seam() -> String {
    ["serve", "as", "peer"].join("_")
}

fn rust_files(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn only_the_server_defines_and_its_tests_call_the_peer_seam() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for group in [crates.clone(), crates.join("adapters")] {
        let Ok(entries) = fs::read_dir(group) else {
            continue;
        };
        for entry in entries.flatten() {
            rust_files(&entry.path().join("src"), &mut files);
        }
    }
    let name = seam();
    let users: Vec<_> = files
        .iter()
        .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(&name)))
        .map(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(users, [Some("server.rs".to_owned())], "{users:?}");
}
