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
