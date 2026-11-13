//! `Principal::local_owner` may be created only by the single-user composition root (ADR 0019).

use std::fs;
use std::path::{Path, PathBuf};

const CREATOR: &str = "Principal::local_owner(";

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
fn only_the_composition_root_creates_the_local_owner() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut offenders = Vec::new();
    for entry in fs::read_dir(&crates).unwrap() {
        let mut files = Vec::new();
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
        for file in files {
            let allowed =
                file.ends_with("app/src/principal.rs") || file.ends_with("domain/src/access.rs");
            if !allowed && fs::read_to_string(&file).unwrap().contains(CREATOR) {
                offenders.push(file);
            }
        }
    }
    assert!(offenders.is_empty(), "{offenders:?}");
}
