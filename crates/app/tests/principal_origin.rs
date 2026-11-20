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
const FEATURE_OWNER: &str = "crates/app/Cargo.toml";
const FEATURE_DEFINITION: &str = "issue-principals = []";

/// Every `Cargo.toml` of the repository outside `target`, with its path relative to the root.
#[cfg(test)]
fn manifests() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    let mut open = vec![root.clone()];
    while let Some(dir) = open.pop() {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() && name != "target" && name != "node_modules" && !name.starts_with('.')
            {
                open.push(path);
            } else if name == "Cargo.toml" {
                let relative = path.strip_prefix(&root).unwrap().to_string_lossy();
                found.push((
                    relative.replace('\\', "/"),
                    fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    found
}

/// Whether the manifest at `path` mentions the feature anywhere it must not. The feature may be
/// defined in the domain crate, switched on in `[dev-dependencies]` or `[target.*.dev-dependencies]`
/// of any crate, and switched on in the normal dependencies of the composition root only.
fn mentions_feature_wrongly(path: &str, text: &str) -> bool {
    let mut section = String::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            line.clone_into(&mut section);
        }
        if !line.contains(FEATURE) || line.starts_with('#') {
            continue;
        }
        let allowed = line == FEATURE_DEFINITION
            || section.contains("dev-dependencies")
            || path == FEATURE_OWNER;
        if !allowed {
            return true;
        }
    }
    false
}

/// The feature that makes principal constructors public is switched on in the normal dependencies
/// of the composition root only; everywhere else a dev-dependency may enable it for tests. Cargo
/// unifies features inside one build, so this test (and not the compiler) is what keeps a driving
/// adapter from calling the constructors in production code (issue #70).
#[test]
fn only_the_composition_root_enables_the_principal_feature_for_products() {
    let manifests = manifests();
    assert!(manifests.len() > 10, "found {} manifests", manifests.len());
    let offenders: Vec<_> = manifests
        .iter()
        .filter(|(path, text)| mentions_feature_wrongly(path, text))
        .map(|(path, _)| path)
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
}

#[test]
fn the_manifest_check_sees_every_form_of_switching_the_feature_on() {
    let inline =
        "[dependencies]\ncodeskimr_domain = { path = \"x\", features = [\"issue-principals\"] }\n";
    let table =
        "[dependencies.codeskimr_domain]\npath = \"x\"\nfeatures = [\"issue-principals\"]\n";
    let target = "[target.'cfg(unix)'.dependencies]\ncodeskimr_domain = { path = \"x\", features = [\"issue-principals\"] }\n";
    let default = "[features]\ndefault = [\"issue-principals\"]\n";
    let workspace = "[workspace.dependencies]\ncodeskimr_domain = { path = \"x\", features = [\"issue-principals\"] }\n";
    for text in [inline, table, target, default, workspace] {
        assert!(
            mentions_feature_wrongly("crates/server/Cargo.toml", text),
            "{text}"
        );
        assert!(
            mentions_feature_wrongly("crates/adapters/app/Cargo.toml", text),
            "{text}"
        );
        assert!(mentions_feature_wrongly("Cargo.toml", text), "{text}");
    }
    assert!(!mentions_feature_wrongly(FEATURE_OWNER, inline));
    let dev = "[dev-dependencies]\ncodeskimr_domain = { path = \"x\", features = [\"issue-principals\"] }\n";
    let target_dev = "[target.'cfg(unix)'.dev-dependencies]\ncodeskimr_domain = { path = \"x\", features = [\"issue-principals\"] }\n";
    assert!(!mentions_feature_wrongly("crates/server/Cargo.toml", dev));
    assert!(!mentions_feature_wrongly(
        "crates/server/Cargo.toml",
        target_dev
    ));
    assert!(!mentions_feature_wrongly(
        "crates/domain/Cargo.toml",
        "[features]\nissue-principals = []\n"
    ));
}

/// The file of the MCP channels, which share process and token with the HTTP commands of `serve`.
const MCP_FILE: &str = "app/src/mcp.rs";

/// Every MCP channel runs as `remote_client`: the file that builds them never names the principal
/// of the browser, so it cannot hand it to a tool by accident (ADR 0072, decision 2).
#[test]
fn the_mcp_channels_never_name_the_browser_client() {
    let file = product::product_sources()
        .into_iter()
        .find(|file| file.ends_with(MCP_FILE))
        .unwrap();
    let code = fs::read_to_string(file).unwrap();
    assert!(!code.contains("browser_client"));
    assert!(code.contains("remote_client"));
}
