//! Which of the project's files a request names.

use codeskimr_domain::RelativePath;

use crate::consts::{CURRENT_DIR, CURRENT_DIR_PREFIX, PATH_SEPARATOR, WINDOWS_SEPARATOR};
use crate::ports::DiscoveredFile;

/// The wanted paths in the form the project uses: forward slashes, no leading `./`, no trailing
/// separator.
pub(super) struct Selection(Vec<String>);

impl Selection {
    pub(super) fn new(paths: &[String]) -> Self {
        let tidy = |path: &String| {
            let slashed = path.replace(WINDOWS_SEPARATOR, &PATH_SEPARATOR.to_string());
            let trimmed = slashed
                .trim_start_matches(CURRENT_DIR_PREFIX)
                .trim_end_matches(PATH_SEPARATOR);
            if trimmed == CURRENT_DIR {
                String::new()
            } else {
                trimmed.to_owned()
            }
        };
        Self(paths.iter().map(tidy).collect())
    }

    /// Whether `path` is named, or lies below a named folder. No path named selects everything.
    pub(super) fn contains(&self, path: &RelativePath) -> bool {
        self.0.is_empty() || self.0.iter().any(|wanted| names(wanted, path))
    }

    /// The named paths that select none of `files`; a typo must not look like a clean check.
    pub(super) fn unmatched(&self, files: &[DiscoveredFile]) -> Vec<String> {
        self.0
            .iter()
            .filter(|wanted| !files.iter().any(|file| names(wanted, &file.path)))
            .cloned()
            .collect()
    }
}

fn names(wanted: &str, path: &RelativePath) -> bool {
    wanted.is_empty()
        || path.as_str() == wanted
        || path
            .as_str()
            .strip_prefix(wanted)
            .is_some_and(|rest| rest.starts_with(PATH_SEPARATOR))
}
