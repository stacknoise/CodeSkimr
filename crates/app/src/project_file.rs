//! Reading the project file `.codeskimr.toml` as text, for the layer editor and the exception
//! list of the unused symbols.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Read};
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectSource;

use codeskimr_application::config::PROJECT_CONFIG_FILE;

use crate::consts::PROJECT_FILE_MAX_BYTES;

/// The text of the project file; empty if there is none. A link or anything but a file is
/// refused, and no more than the limit is read, however the file grows while it is read.
pub(crate) fn read_project_file(path: &Path) -> Result<String, CodedError> {
    let unreadable = || CodedError::new(ErrorCode::ConfigParseFailed);
    let meta = match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(String::new()),
        Err(_) => return Err(unreadable()),
        Ok(meta) => meta,
    };
    if !meta.is_file() {
        return Err(unreadable());
    }
    let mut text = String::new();
    let read = fs::File::open(path).and_then(|file| {
        file.take(PROJECT_FILE_MAX_BYTES + 1)
            .read_to_string(&mut text)
    });
    match read {
        Ok(_) if text.len() as u64 <= PROJECT_FILE_MAX_BYTES => Ok(text),
        _ => Err(unreadable()),
    }
}

/// The version of a project file text: a number that changes when the text does. Only a server
/// compares it with one it made before, so it needs to be the same for the same text within a run.
pub(crate) fn version_of(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish().to_string()
}

/// The version of the project file of `source`; empty for a Git project, which has none.
pub(crate) fn version_of_source(source: &ProjectSource) -> Result<String, CodedError> {
    let ProjectSource::Folder(folder) = source else {
        return Ok(String::new());
    };
    let path = Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE);
    read_project_file(&path).map(|text| version_of(&text))
}

/// Refuses a write that was made from an older state of the file: `expected` is the version the
/// client read, `text` the file as it is now. A client that sends none writes without the check.
///
/// # Errors
/// `CFG-106` if the file changed since the client read it.
pub(crate) fn require_version(text: &str, expected: Option<&str>) -> Result<(), CodedError> {
    match expected {
        Some(version) if version != version_of(text) => {
            Err(CodedError::new(ErrorCode::ProjectFileChanged))
        }
        _ => Ok(()),
    }
}
