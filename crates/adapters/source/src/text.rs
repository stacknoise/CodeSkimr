//! Reading lines of a file of a folder source.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Excerpt, SourceText};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, RelativePath};

use crate::consts::{CARRIAGE_RETURN, FIRST_LINE, LINE_END, RELATIVE_PATH_SEPARATOR};

/// Reads lines of the files below a folder.
#[derive(Debug, Clone, Copy, Default)]
pub struct FolderText;

/// The real path of `path` below `root`, `None` if there is no such file or it leaves the root.
///
/// Both sides are resolved, so a link inside the project that points outside does not lead out.
fn inside(root: &Path, path: &RelativePath) -> Option<PathBuf> {
    let base = fs::canonicalize(root).ok()?;
    let joined: PathBuf = path
        .as_str()
        .split(RELATIVE_PATH_SEPARATOR)
        .collect::<PathBuf>();
    let real = fs::canonicalize(base.join(joined)).ok()?;
    (real.starts_with(&base) && real.is_file()).then_some(real)
}

fn not_found() -> CodedError {
    CodedError::new(ErrorCode::SourcePathNotFound)
}

/// The lines `range` of the opened file. Reading stops at `max_bytes`, so a file that grows after
/// the size check cannot make the answer larger than the limit.
fn lines_of(file: File, range: (u32, u32), limits: (u32, u64)) -> Result<Excerpt, CodedError> {
    let (first, last) = range;
    let (max_lines, max_bytes) = limits;
    let size = file.metadata().map_err(|_| not_found())?.len();
    if size > max_bytes {
        return Err(CodedError::new(ErrorCode::SourceTooLarge));
    }
    let reader = BufReader::new(file.take(max_bytes));
    let wanted = last.saturating_sub(first).saturating_add(FIRST_LINE);
    let take = wanted.min(max_lines);
    let skip = usize::try_from(first.saturating_sub(FIRST_LINE)).unwrap_or(usize::MAX);
    let lines: Vec<String> = reader
        .split(LINE_END)
        .skip(skip)
        .take(usize::try_from(take).unwrap_or(usize::MAX))
        .map(|line| {
            let bytes = line.map_err(|_| not_found())?;
            Ok(String::from_utf8_lossy(&bytes)
                .trim_end_matches(CARRIAGE_RETURN)
                .to_owned())
        })
        .collect::<Result<_, CodedError>>()?;
    Ok(Excerpt {
        lines,
        truncated: wanted > take,
    })
}

impl SourceText for FolderText {
    fn excerpt(
        &self,
        root: &FolderPath,
        path: &RelativePath,
        range: (u32, u32),
        limits: (u32, u64),
    ) -> Result<Excerpt, CodedError> {
        let real = inside(Path::new(root.as_str()), path).ok_or_else(not_found)?;
        let file = File::open(real).map_err(|_| not_found())?;
        lines_of(file, range, limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeskimr_codes::Coded;

    fn folder(files: &[(&str, &str)]) -> (tempfile::TempDir, FolderPath) {
        let dir = tempfile::tempdir().unwrap();
        for (name, text) in files {
            let target = dir.path().join(name);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, text).unwrap();
        }
        let root = FolderPath::new(dir.path().to_str().unwrap()).unwrap();
        (dir, root)
    }

    fn read(root: &FolderPath, path: &str, range: (u32, u32)) -> Result<Excerpt, CodedError> {
        FolderText.excerpt(root, &RelativePath::new(path).unwrap(), range, (3, 1000))
    }

    #[test]
    fn it_returns_the_lines_of_the_range_without_line_ends() {
        let (_dir, root) = folder(&[("src/a.cs", "one\r\ntwo\nthree\nfour\n")]);
        let excerpt = read(&root, "src/a.cs", (2, 3)).unwrap();
        assert_eq!(excerpt.lines, ["two", "three"]);
        assert!(!excerpt.truncated);
    }

    #[test]
    fn it_cuts_at_the_limit_and_says_so() {
        let (_dir, root) = folder(&[("a.cs", "1\n2\n3\n4\n5\n")]);
        let excerpt = read(&root, "a.cs", (1, 5)).unwrap();
        assert_eq!(excerpt.lines, ["1", "2", "3"]);
        assert!(excerpt.truncated);
    }

    #[test]
    fn a_range_behind_the_end_is_empty() {
        let (_dir, root) = folder(&[("a.cs", "1\n")]);
        assert!(read(&root, "a.cs", (9, 12)).unwrap().lines.is_empty());
    }

    #[test]
    fn a_missing_file_and_a_folder_are_not_found() {
        let (_dir, root) = folder(&[("src/a.cs", "x")]);
        for path in ["nope.cs", "src"] {
            let error = read(&root, path, (1, 1)).unwrap_err();
            assert_eq!(error.code().entry().id, "SRC-201", "{path}");
        }
    }

    #[test]
    fn a_path_that_leaves_the_root_is_not_found() {
        let (_dir, root) = folder(&[("project/a.cs", "x"), ("secret.txt", "key")]);
        let inner = FolderPath::new(format!("{}/project", root.as_str())).unwrap();
        let error = read(&inner, "../secret.txt", (1, 1)).unwrap_err();
        assert_eq!(error.code().entry().id, "SRC-201");
    }

    #[cfg(unix)]
    #[test]
    fn a_link_that_points_outside_is_not_found() {
        let (dir, root) = folder(&[("project/a.cs", "x"), ("secret.txt", "key")]);
        let inner = dir.path().join("project");
        std::os::unix::fs::symlink(dir.path().join("secret.txt"), inner.join("link.txt")).unwrap();
        let inner = FolderPath::new(inner.to_str().unwrap()).unwrap();
        let error = read(&inner, "link.txt", (1, 1)).unwrap_err();
        assert_eq!(error.code().entry().id, "SRC-201");
        drop(root);
    }

    #[test]
    fn an_absolute_path_is_not_found() {
        let (dir, root) = folder(&[("a.cs", "x"), ("secret.txt", "key")]);
        let outside = dir.path().join("secret.txt");
        let error = read(&root, outside.to_str().unwrap(), (1, 1)).unwrap_err();
        assert_eq!(error.code().entry().id, "SRC-201");
    }

    #[test]
    fn one_very_long_line_stays_inside_the_byte_limit() {
        let (_dir, root) = folder(&[("long.cs", &"x".repeat(900))]);
        let excerpt = FolderText
            .excerpt(
                &root,
                &RelativePath::new("long.cs").unwrap(),
                (1, 1),
                (3, 1000),
            )
            .unwrap();
        assert_eq!(excerpt.lines[0].len(), 900);
        let error = FolderText
            .excerpt(
                &root,
                &RelativePath::new("long.cs").unwrap(),
                (1, 1),
                (3, 899),
            )
            .unwrap_err();
        assert_eq!(error.code().entry().id, "SRC-501");
    }

    #[test]
    fn reading_stops_at_the_byte_limit_even_if_the_size_check_was_outdated() {
        let (dir, _root) = folder(&[("grown.cs", "0123456789\n0123456789\n")]);
        let file = File::open(dir.path().join("grown.cs")).unwrap();
        // The limit is 15 bytes, the file has 22: the size check refuses it...
        assert!(lines_of(file, (1, 2), (3, 15)).is_err());
        // ...and a file that passes the check is read at most up to the limit.
        let file = File::open(dir.path().join("grown.cs")).unwrap();
        let excerpt = lines_of(file, (1, 2), (3, 22)).unwrap();
        assert_eq!(excerpt.lines, ["0123456789", "0123456789"]);
    }

    #[test]
    fn a_file_over_the_byte_limit_is_refused() {
        let (_dir, root) = folder(&[("big.cs", "0123456789")]);
        let error = FolderText
            .excerpt(&root, &RelativePath::new("big.cs").unwrap(), (1, 1), (3, 4))
            .unwrap_err();
        assert_eq!(error.code().entry().id, "SRC-501");
    }
}
