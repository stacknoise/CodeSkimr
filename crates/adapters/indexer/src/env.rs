//! The environment of an indexer: built from an allow list, never inherited (ADR 0033,
//! decision 11).
//!
//! The application's environment may hold tokens, passwords and its own settings. An indexer
//! runs project code, so it gets only what the tools need to find their installation.

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use codeskimr_application::ports::IndexerSpec;

use crate::consts::{
    APPLICATION_ENV_PREFIX, BASE_ENV_ALLOW, PATH_ENV, SECRET_NAME_PARTS, TEMP_ENV,
};

/// Name/value pairs for `Command::env` after `env_clear`.
pub type Environment = Vec<(OsString, OsString)>;

/// Builds the environment of one run from `source`, the variables of the application.
///
/// `scratch` becomes the temporary folder of the run; `project_root` is kept out of `PATH`.
#[must_use]
pub fn build(
    source: impl IntoIterator<Item = (OsString, OsString)>,
    spec: &IndexerSpec,
    scratch: &Path,
    project_root: &Path,
) -> Environment {
    let mut result: Environment = source
        .into_iter()
        .filter(|(name, _)| is_allowed(name, spec))
        .map(|(name, value)| keep_safe_path(name, value, project_root))
        .collect();
    for name in TEMP_ENV {
        result.push((OsString::from(name), scratch.as_os_str().to_owned()));
    }
    for (name, value) in &spec.env {
        result.retain(|(existing, _)| !same_name(existing, name));
        result.push((OsString::from(name), OsString::from(value)));
    }
    result
}

/// Whether the variable is on the base list or the list of the pack, and does not look like a
/// secret or belong to the application.
fn is_allowed(name: &OsStr, spec: &IndexerSpec) -> bool {
    let upper = name.to_string_lossy().to_uppercase();
    if upper.starts_with(APPLICATION_ENV_PREFIX)
        || SECRET_NAME_PARTS.iter().any(|part| upper.contains(part))
        || TEMP_ENV.iter().any(|temp| *temp == upper)
    {
        return false;
    }
    BASE_ENV_ALLOW.iter().any(|allowed| *allowed == upper)
        || spec
            .env_allow
            .iter()
            .any(|allowed| allowed.to_uppercase() == upper)
}

fn same_name(left: &OsStr, right: &str) -> bool {
    left.to_string_lossy().eq_ignore_ascii_case(right)
}

fn keep_safe_path(name: OsString, value: OsString, project_root: &Path) -> (OsString, OsString) {
    if same_name(&name, PATH_ENV) {
        let safe = safe_path(&value, project_root);
        (name, safe)
    } else {
        (name, value)
    }
}

/// `PATH` without relative entries (they would resolve inside the project, which is the working
/// folder of the indexer) and without entries inside the project.
#[must_use]
pub fn safe_path(path: &OsStr, project_root: &Path) -> OsString {
    let root = canonical(project_root);
    let kept: Vec<PathBuf> = env::split_paths(path)
        .filter(|entry| entry.is_absolute() && !canonical(entry).starts_with(&root))
        .collect();
    env::join_paths(kept).unwrap_or_default()
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
