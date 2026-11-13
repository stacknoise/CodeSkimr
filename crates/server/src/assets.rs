//! The built UI: static files from one folder (ADR 0028, ADR 0030).
//!
//! A path is only taken literally, never decoded: every segment may hold letters, digits, `.`,
//! `_` and `-` and must not start with a dot. That leaves nothing for `..`, an encoded `..`, a
//! drive letter or a hidden file to hide in. Unknown paths without a file extension get the start
//! page, so that the UI can route by itself; unknown files get `API-201`.

use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use codeskimr_codes::ErrorCode;

use crate::consts::{INDEX_FILE, MEDIA_TYPES, MEDIA_UNKNOWN};
use crate::error::Failure;

const SEGMENT_SEPARATOR: char = '/';
const EXTENSION_SEPARATOR: char = '.';
const SEGMENT_EXTRA: [char; 3] = ['.', '_', '-'];

/// The folder the UI is served from.
#[derive(Debug, Clone)]
pub struct Assets {
    dir: PathBuf,
}

impl Assets {
    /// Assets below `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The answer for a `GET` of `path`.
    pub async fn get(&self, path: &str) -> Response {
        let Some(relative) = relative_file(path) else {
            return Failure::code(ErrorCode::CommandNotFound).into_response();
        };
        match read(&self.dir.join(&relative)).await {
            Some(body) => answer(&relative, body),
            None => Failure::code(ErrorCode::CommandNotFound).into_response(),
        }
    }
}

/// The file below the folder that `path` names, or `None` for anything that is not a plain path.
/// The root and a path without a file extension name the start page.
fn relative_file(path: &str) -> Option<PathBuf> {
    let trimmed = path.trim_start_matches(SEGMENT_SEPARATOR);
    let segments: Vec<&str> = trimmed
        .split(SEGMENT_SEPARATOR)
        .filter(|segment| !segment.is_empty())
        .collect();
    if !segments.iter().copied().all(is_plain_segment) {
        return None;
    }
    let last = segments.last().copied().unwrap_or_default();
    if last.contains(EXTENSION_SEPARATOR) {
        Some(segments.iter().collect())
    } else {
        Some(PathBuf::from(INDEX_FILE))
    }
}

fn is_plain_segment(segment: &str) -> bool {
    !segment.starts_with(EXTENSION_SEPARATOR)
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || SEGMENT_EXTRA.contains(&c))
}

/// The content of a regular file; a folder, a missing file or a read error is `None`.
async fn read(path: &Path) -> Option<Vec<u8>> {
    let meta = tokio::fs::metadata(path).await.ok()?;
    if !meta.is_file() {
        return None;
    }
    tokio::fs::read(path).await.ok()
}

fn answer(file: &Path, body: Vec<u8>) -> Response {
    let media = file
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(|extension| {
            MEDIA_TYPES
                .iter()
                .find(|(known, _)| known.eq_ignore_ascii_case(extension))
        })
        .map_or(MEDIA_UNKNOWN, |(_, media)| media);
    (StatusCode::OK, [(CONTENT_TYPE, media)], body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_paths_name_files_and_the_start_page() {
        assert_eq!(relative_file("/"), Some(PathBuf::from("index.html")));
        assert_eq!(
            relative_file("/settings"),
            Some(PathBuf::from("index.html"))
        );
        assert_eq!(
            relative_file("/assets/app-3fA_9.js"),
            Some(PathBuf::from("assets").join("app-3fA_9.js"))
        );
    }

    #[test]
    fn nothing_leaves_the_folder() {
        for path in [
            "/../secret.txt",
            "/assets/../../x.js",
            "/%2e%2e/x.js",
            "/.env",
            "/assets/.hidden.js",
            "/a\\b.js",
            "/C:/x.js",
            "//..//x.js",
        ] {
            assert_eq!(relative_file(path), None, "{path}");
        }
    }
}
