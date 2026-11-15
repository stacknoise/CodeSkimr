//! A command request is built only where the permission is checked (issue #344): inside the
//! application crate, `Request::new` may be called by the registry code and nowhere else.

use std::fs;
use std::path::{Path, PathBuf};

const BUILDER: &str = "Request::new(";
const ALLOWED: [&str; 2] = ["registry/command.rs", "registry/mod.rs"];

/// All `.rs` files below `root`, found with a work list instead of recursion.
#[cfg(test)]
fn rust_files(root: PathBuf) -> Vec<PathBuf> {
    let mut pending = vec![root];
    let mut files = Vec::new();
    while let Some(dir) = pending.pop() {
        let entries = fs::read_dir(&dir).map(|read| read.flatten().collect::<Vec<_>>());
        for entry in entries.unwrap_or_default() {
            let path = entry.path();
            match path.extension().and_then(|ext| ext.to_str()) {
                _ if path.is_dir() => pending.push(path),
                Some("rs") => files.push(path),
                _ => {}
            }
        }
    }
    files
}

#[test]
fn only_the_registry_builds_requests() {
    let files = rust_files(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
    assert!(files.len() > 20, "the scan found {} files", files.len());
    let offenders: Vec<_> = files
        .iter()
        .filter(|file| {
            let path = file.to_string_lossy().replace('\\', "/");
            !ALLOWED.iter().any(|allowed| path.ends_with(allowed))
        })
        .filter(|file| fs::read_to_string(file).unwrap().contains(BUILDER))
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
}
