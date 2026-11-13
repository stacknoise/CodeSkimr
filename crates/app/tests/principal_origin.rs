//! Full-permission principals may be created only by the single-user composition root
//! (ADR 0019): no other crate may build `local_owner`, `limited` or `PermissionSet::all`.

use std::fs;
use std::path::{Path, PathBuf};

const CREATORS: [&str; 3] = ["local_owner", "limited", "PermissionSet::all"];
const ALLOWED: [&str; 2] = ["app/src/principal.rs", "domain/src/access.rs"];

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
fn product_sources(crates: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(crates).unwrap() {
        let path = entry.unwrap().path();
        if path.join("src").is_dir() {
            sources(&path.join("src"), found);
        } else if path.is_dir() {
            product_sources(&path, found);
        }
    }
}

#[test]
fn only_the_composition_root_creates_principals() {
    let mut files = Vec::new();
    product_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join(".."),
        &mut files,
    );
    assert!(
        files.len() > 50,
        "the scan found too few files: {}",
        files.len()
    );
    let offenders: Vec<_> = files
        .iter()
        .filter(|file| !ALLOWED.iter().any(|allowed| file.ends_with(allowed)))
        .filter(|file| {
            let text = fs::read_to_string(file).unwrap();
            CREATORS.iter().any(|creator| text.contains(creator))
        })
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
}
