//! The peer seam of `BoundServer` (the method that serves with a fixed peer) is for tests (issue
//! #531): it sets the peer address that the loopback check of the confirmation route looks at.
//! Nothing else in the repository may call it (program code, build scripts, tools), or a caller
//! could claim to be somebody else.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// Where the seam is defined and where it is used, relative to the workspace root.
const ALLOWED: [&str; 2] = [
    "crates/server/src/server.rs",
    "crates/server/tests/confirm_window.rs",
];
/// Folders that hold no source of the repository.
const SKIPPED: [&str; 4] = ["target", ".git", "node_modules", "web"];

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
        let skipped = path
            .file_name()
            .is_some_and(|name| SKIPPED.iter().any(|skip| name == *skip));
        if path.is_dir() && !skipped {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn only_the_definition_and_the_test_of_the_route_use_the_peer_seam() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(files.len() > ALLOWED.len(), "the scan found no sources");
    let name = seam();
    let mut users: Vec<String> = files
        .iter()
        .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(&name)))
        .filter_map(|path| path.strip_prefix(&root).ok())
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    users.sort();
    assert_eq!(users, ALLOWED, "{users:?}");
}
