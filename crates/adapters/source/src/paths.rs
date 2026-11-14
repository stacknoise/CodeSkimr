//! Paths relative to a project root.

use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_domain::RelativePath;

use crate::consts::RELATIVE_PATH_SEPARATOR;

/// The path below `root` with forward slashes, `None` if `full` is not below `root` or is not
/// valid UTF-8.
#[must_use]
pub fn relative_path(root: &Path, full: &Path) -> Option<RelativePath> {
    let parts: Option<Vec<&str>> = full
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_str())
        .collect();
    let separator = RELATIVE_PATH_SEPARATOR.to_string();
    RelativePath::new(parts?.join(&separator)).ok()
}

/// The real path of `path` below `root`, `None` if there is no such file or it leaves the root.
///
/// Both sides are resolved, so a link inside the project that points outside does not lead out.
pub(crate) fn inside(root: &Path, path: &RelativePath) -> Option<PathBuf> {
    let base = fs::canonicalize(root).ok()?;
    let joined: PathBuf = path
        .as_str()
        .split(RELATIVE_PATH_SEPARATOR)
        .collect::<PathBuf>();
    let real = fs::canonicalize(base.join(joined)).ok()?;
    (real.starts_with(&base) && real.is_file()).then_some(real)
}
