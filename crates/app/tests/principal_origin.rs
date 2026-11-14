//! Full-permission principals may be created only by the single-user composition root
//! (ADR 0019): no other crate may build `local_owner`, `limited` or `PermissionSet::all`.

#[cfg(test)]
mod product;

use std::fs;

const CREATORS: [&str; 3] = ["local_owner", "limited", "PermissionSet::all"];
const ALLOWED: [&str; 2] = ["app/src/principal.rs", "domain/src/access.rs"];

#[test]
fn only_the_composition_root_creates_principals() {
    let files = product::product_sources();
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

const FEATURE: &str = "issue-principals";
const FEATURE_OWNER: &str = "/app/Cargo.toml";
const DEV_SECTION: &str = "[dev-dependencies]";

/// The manifests below `crates/`.
#[cfg(test)]
fn manifests() -> Vec<std::path::PathBuf> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut found = Vec::new();
    let mut open = vec![root];
    while let Some(dir) = open.pop() {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                open.push(path);
            } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
                found.push(path);
            }
        }
    }
    found
}

/// The feature that makes principal constructors public is switched on in the normal dependencies
/// of the composition root only; everywhere else a dev-dependency may enable it for tests, so a
/// product build of any other crate cannot create a principal (issue #70).
#[test]
fn only_the_composition_root_enables_the_principal_feature_for_products() {
    let mut offenders = Vec::new();
    for manifest in manifests() {
        let text = fs::read_to_string(&manifest).unwrap();
        let mut section = "";
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                section = line;
            }
            let enables = line.contains(FEATURE) && line.contains("codeskimr_domain");
            let allowed = section == DEV_SECTION
                || manifest
                    .to_string_lossy()
                    .replace('\\', "/")
                    .ends_with(FEATURE_OWNER);
            if enables && !allowed {
                offenders.push(manifest.clone());
            }
        }
    }
    assert!(offenders.is_empty(), "{offenders:?}");
}
