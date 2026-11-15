//! A command request is built only where the permission is checked (issue #344): inside the
//! application crate, `Request::new` may be called by the registry code and nowhere else.

use std::fs;
use std::path::{Path, PathBuf};

const BUILDER: &str = "Request::new(";
const ALLOWED: [&str; 2] = ["registry/command.rs", "registry/mod.rs"];

#[cfg(test)]
fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn only_the_registry_builds_requests() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
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
