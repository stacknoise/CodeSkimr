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
    APPLICATION_ENV_PREFIX, BASE_ENV_ALLOW, FIXED_ENV, HOME_ENV, HOME_SETTINGS_ENV, PATH_ENV,
    SECRET_NAME_PARTS, TEMP_ENV,
};

/// Name/value pairs for `Command::env` after `env_clear`.
pub type Environment = Vec<(OsString, OsString)>;

/// The folders of one run: `scratch` becomes the temporary folder, `home` the empty home, and
/// `project_root` is kept out of `PATH`.
#[derive(Debug, Clone, Copy)]
pub struct Folders<'a> {
    /// The temporary folder of the run.
    pub scratch: &'a Path,
    /// The empty home folder of the run.
    pub home: &'a Path,
    /// The project folder.
    pub project_root: &'a Path,
}

/// Builds the environment of one run from `source`, the variables of the application.
///
/// The home of the user never reaches the indexer: `HOME` and `USERPROFILE` name an empty folder.
/// A folder that a tool needs from the real home is named by the variable of the pack
/// (`home_dirs`), unless the application passed the variable itself.
#[must_use]
pub fn build(
    source: impl IntoIterator<Item = (OsString, OsString)>,
    spec: &IndexerSpec,
    folders: &Folders<'_>,
) -> Environment {
    let source: Environment = source.into_iter().collect();
    let mut result: Environment = source
        .iter()
        .filter(|(name, _)| is_allowed(name, spec))
        .map(|(name, value)| keep_safe_path(name.clone(), value.clone(), folders.project_root))
        .collect();
    for name in TEMP_ENV {
        result.push((OsString::from(name), folders.scratch.as_os_str().to_owned()));
    }
    for name in HOME_ENV {
        result.push((OsString::from(name), folders.home.as_os_str().to_owned()));
    }
    for (name, below) in HOME_SETTINGS_ENV {
        let folder = below
            .iter()
            .fold(folders.home.to_path_buf(), |path, part| path.join(part));
        result.push((OsString::from(name), folder.into_os_string()));
    }
    for (name, value) in FIXED_ENV {
        result.push((OsString::from(name), OsString::from(value)));
    }
    add_home_dirs(&mut result, &source, spec);
    for (name, value) in &spec.env {
        result.retain(|(existing, _)| !same_name(existing, name));
        result.push((OsString::from(name), OsString::from(value)));
    }
    result
}

/// The real home of the user, as the application sees it.
fn real_home(source: &Environment) -> Option<PathBuf> {
    HOME_ENV.iter().find_map(|home| {
        source
            .iter()
            .find(|(name, _)| same_name(name, home))
            .map(|(_, value)| PathBuf::from(value))
            .filter(|path| path.is_absolute())
    })
}

/// Names the folders of the real home that the pack asks for, if they exist and the variable is
/// not set yet.
fn add_home_dirs(result: &mut Environment, source: &Environment, spec: &IndexerSpec) {
    let Some(home) = real_home(source) else {
        return;
    };
    for (variable, below) in &spec.home_dirs {
        let folder = home.join(below);
        if result.iter().all(|(name, _)| !same_name(name, variable)) && folder.is_dir() {
            result.push((OsString::from(variable), folder.into_os_string()));
        }
    }
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

/// The entries of `path` that an indexer may use: absolute, not empty and not inside the project.
/// A relative or empty entry would be looked up relative to a working folder, which is the project
/// for the indexer and wherever the user started `CodeSkimr`.
#[must_use]
pub fn safe_dirs(path: &OsStr, project_root: &Path) -> Vec<PathBuf> {
    let root = canonical(project_root);
    env::split_paths(path)
        .filter(|entry| entry.is_absolute() && !canonical(entry).starts_with(&root))
        .collect()
}

/// `PATH` as text, restricted to [`safe_dirs`].
#[must_use]
pub fn safe_path(path: &OsStr, project_root: &Path) -> OsString {
    env::join_paths(safe_dirs(path, project_root)).unwrap_or_default()
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::safe_path;

    #[test]
    fn relative_entries_and_entries_inside_the_project_leave_the_path() {
        let root = std::env::temp_dir().join("codeskimr-env-test-project");
        let inside = root.join("bin");
        let outside = std::env::temp_dir().join("codeskimr-env-test-other");
        let relative = Path::new("tools").to_path_buf();
        let joined = std::env::join_paths([relative, inside, outside.clone()]).unwrap();
        let safe = safe_path(&joined, &root);
        let kept: Vec<_> = std::env::split_paths(&safe).collect();
        assert_eq!(kept, [outside]);
    }
}

#[cfg(test)]
mod search_tests {
    use std::path::Path;

    use super::safe_dirs;

    #[test]
    fn an_empty_or_only_relative_path_leaves_no_directory() {
        let root = std::env::temp_dir();
        assert_eq!(
            safe_dirs(std::ffi::OsStr::new(""), &root),
            Vec::<std::path::PathBuf>::new()
        );
        let relative = std::env::join_paths([Path::new("."), Path::new("bin")]).unwrap();
        assert_eq!(
            safe_dirs(&relative, &root),
            Vec::<std::path::PathBuf>::new()
        );
    }
}
