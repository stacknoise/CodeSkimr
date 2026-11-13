//! Folder sources: nothing to fetch, only to check.

use std::fs;
use std::path::Path;

use codeskimr_application::ports::FetchedSource;
use codeskimr_domain::FolderPath;

use crate::error::SourceError;

/// Opens a folder source. The folder must exist and be listable.
///
/// # Errors
/// [`SourceError::PathNotFound`] if `root` is not a readable folder.
pub fn open_folder(root: &FolderPath) -> Result<FetchedSource, SourceError> {
    let path = Path::new(root.as_str());
    let readable = path.is_dir() && fs::read_dir(path).is_ok();
    if readable {
        Ok(FetchedSource {
            root: root.clone(),
            commit: None,
        })
    } else {
        Err(SourceError::PathNotFound { path: root.clone() })
    }
}
