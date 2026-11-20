//! The built UI: static files from one folder or from the program itself (ADR 0028, ADR 0030).
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
use tokio::io::AsyncReadExt;

use crate::consts::{
    EXTENSION_SEPARATOR, INDEX_FILE, MEDIA_TYPES, MEDIA_UNKNOWN, SEGMENT_EXTRA, SEGMENT_SEPARATOR,
};
use crate::error::Failure;

/// One file of the UI inside the program: its path below the UI folder, with `/` between the
/// segments, and its content.
pub type EmbeddedFile = (&'static str, &'static [u8]);

/// Where the UI files come from.
#[derive(Debug, Clone)]
enum Source {
    /// A folder on disk, for development and for a UI folder the user puts next to the program.
    Dir(PathBuf),
    /// Files compiled into the program.
    Embedded(&'static [EmbeddedFile]),
}

/// The UI files the server serves.
#[derive(Debug, Clone)]
pub struct Assets {
    source: Source,
}

impl Assets {
    /// Assets below `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            source: Source::Dir(dir),
        }
    }

    /// Assets that are part of the program.
    #[must_use]
    pub const fn embedded(files: &'static [EmbeddedFile]) -> Self {
        Self {
            source: Source::Embedded(files),
        }
    }

    /// The answer for a `GET` of `path`.
    pub async fn get(&self, path: &str) -> Response {
        let Some(relative) = relative_file(path) else {
            return Failure::code(ErrorCode::CommandNotFound).into_response();
        };
        let body = match &self.source {
            Source::Dir(dir) => read(dir, &relative).await,
            Source::Embedded(files) => embedded(files, &relative),
        };
        match body {
            Some(body) => answer(&relative, body),
            None => Failure::code(ErrorCode::CommandNotFound).into_response(),
        }
    }
}

/// The content of the embedded file at `relative`.
fn embedded(files: &[EmbeddedFile], relative: &Path) -> Option<Vec<u8>> {
    let name: Vec<&str> = relative
        .components()
        .filter_map(|part| part.as_os_str().to_str())
        .collect();
    let name = name.join(&SEGMENT_SEPARATOR.to_string());
    files
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, body)| body.to_vec())
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

/// The content of a regular file below `dir`; a folder, a link (the file itself or any folder on
/// the way), a missing file or a read error is `None`. A link inside the UI folder could lead out
/// of it.
///
/// The path is checked part by part, the file is opened, and then the opened file must be the one
/// that the fully resolved path names, and that path must lie inside the resolved folder. A link
/// put in place after the walk (the file or a folder on the way) is seen there. What remains is a
/// swap in the instant between that last check and the read, which needs write access to the UI
/// folder, the trust boundary itself (issue #310).
async fn read(dir: &Path, relative: &Path) -> Option<Vec<u8>> {
    let mut path = dir.to_path_buf();
    let mut kind = None;
    for part in relative.components() {
        path.push(part);
        let found = tokio::fs::symlink_metadata(&path).await.ok()?.file_type();
        if found.is_symlink() {
            return None;
        }
        kind = Some(found);
    }
    // A pipe or a device would block the opening, so only a regular file is opened.
    if !kind?.is_file() {
        return None;
    }
    let mut file = tokio::fs::File::open(&path).await.ok()?;
    let opened = file.metadata().await.ok()?;
    if !opened.is_file() || !is_inside_and_same(dir, &path, &opened).await {
        return None;
    }
    let mut body = Vec::new();
    file.read_to_end(&mut body).await.ok()?;
    Some(body)
}

/// Whether the resolved `path` lies in the resolved `dir` and names the file that was opened.
async fn is_inside_and_same(dir: &Path, path: &Path, opened: &std::fs::Metadata) -> bool {
    let (Ok(root), Ok(resolved)) = (
        tokio::fs::canonicalize(dir).await,
        tokio::fs::canonicalize(path).await,
    ) else {
        return false;
    };
    let Ok(named) = tokio::fs::metadata(&resolved).await else {
        return false;
    };
    resolved.starts_with(root) && is_same_file(opened, &named)
}

/// Whether both describe the same file.
#[cfg(unix)]
fn is_same_file(opened: &std::fs::Metadata, named: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    opened.dev() == named.dev() && opened.ino() == named.ino()
}

/// Without file identities the resolved path inside the folder has to do.
#[cfg(not(unix))]
fn is_same_file(_opened: &std::fs::Metadata, _named: &std::fs::Metadata) -> bool {
    true
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

    /// None of the request paths names a file of the folder.
    fn assert_no_file(paths: &[&str]) {
        for path in paths {
            assert_eq!(relative_file(path), None, "{path}");
        }
    }

    #[test]
    fn nothing_leaves_the_folder() {
        assert_no_file(&[
            "/../secret.txt",
            "/assets/../../x.js",
            "/%2e%2e/x.js",
            "/.env",
            "/assets/.hidden.js",
            "/a\\b.js",
            "/C:/x.js",
            "//..//x.js",
        ]);
    }

    #[cfg(unix)]
    mod unix {
        use super::*;

        #[tokio::test]
        async fn a_regular_file_is_read_and_links_never() {
            let dir = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            std::fs::write(outside.path().join("secret.js"), "x").unwrap();
            std::fs::write(dir.path().join("app.js"), "ok").unwrap();
            std::os::unix::fs::symlink(outside.path().join("secret.js"), dir.path().join("l.js"))
                .unwrap();
            std::os::unix::fs::symlink(outside.path(), dir.path().join("sub")).unwrap();
            assert_eq!(
                read(dir.path(), Path::new("app.js")).await,
                Some(b"ok".to_vec())
            );
            assert_eq!(read(dir.path(), Path::new("l.js")).await, None);
            assert_eq!(read(dir.path(), Path::new("sub/secret.js")).await, None);
        }

        #[tokio::test]
        async fn a_file_that_is_not_the_opened_one_is_refused() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("a.js"), "a").unwrap();
            std::fs::write(dir.path().join("b.js"), "b").unwrap();
            let opened = std::fs::metadata(dir.path().join("a.js")).unwrap();
            let path = dir.path().join("b.js");
            assert!(!is_inside_and_same(dir.path(), &path, &opened).await);
            let own = dir.path().join("a.js");
            assert!(is_inside_and_same(dir.path(), &own, &opened).await);
        }

        #[tokio::test]
        async fn a_pipe_is_never_opened() {
            let dir = tempfile::tempdir().unwrap();
            let pipe = dir.path().join("p.js");
            let made = std::process::Command::new("mkfifo").arg(&pipe).status();
            if made.is_ok_and(|status| status.success()) {
                assert_eq!(read(dir.path(), Path::new("p.js")).await, None);
            }
        }
    }
}
