//! Paths relative to a project root.

use std::path::Path;

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
