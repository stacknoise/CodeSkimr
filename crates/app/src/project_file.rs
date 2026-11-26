//! Reading the project file `.codeskimr.toml` as text, for the layer editor and the exception
//! list of the unused symbols.

use std::fs;
use std::io::{ErrorKind, Read};
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

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
