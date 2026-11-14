//! The product source files of every crate, for the tests that watch who may call what.

use std::fs;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
fn walk(crates: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(crates).unwrap() {
        let path = entry.unwrap().path();
        if path.join("src").is_dir() {
            sources(&path.join("src"), found);
        } else if path.is_dir() {
            walk(&path, found);
        }
    }
}

/// Every `.rs` file below a `src` folder of the workspace's crates.
#[cfg(test)]
pub fn product_sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join(".."),
        &mut found,
    );
    found
}
