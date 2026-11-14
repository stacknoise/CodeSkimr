//! Listing the configuration and translation keys of a project folder (`find_constants`).
//!
//! Only the names are read, never the values. The folders come from the configuration; the files
//! come from walking them, so no path of a caller is ever opened (blueprint S-05). Links are not
//! followed, and a folder that resolves outside the root is skipped.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{KeyEntry, KeyKind, KeySearch, ProjectKeys};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, RelativePath};

use crate::consts::{
    ASSIGNMENT, BYTES_PER_KB, COMMENT_START, CONFIG_EXTENSION, FLUENT_EXTENSION, KEY_SEPARATOR,
    LINE_END_CHAR, NAME_EXTRA, QUOTE, TABLE_CLOSE, TABLE_OPEN,
};
use crate::paths::relative_path;

/// Lists the keys of the files below the configured folders of a project.
#[derive(Debug, Clone, Copy, Default)]
pub struct FolderKeys;

impl ProjectKeys for FolderKeys {
    fn keys(&self, root: &FolderPath, sources: &KeySearch) -> Result<Vec<KeyEntry>, CodedError> {
        let base = fs::canonicalize(root.as_str())
            .map_err(|_| CodedError::new(ErrorCode::SourcePathNotFound))?;
        let limit = usize::try_from(sources.max_files).unwrap_or(usize::MAX);
        let max_bytes = u64::from(sources.max_file_kb).saturating_mul(BYTES_PER_KB);
        let mut files: Vec<(PathBuf, KeyKind)> = Vec::new();
        for (folders, kind, extension) in [
            (&sources.config_folders, KeyKind::Config, CONFIG_EXTENSION),
            (&sources.i18n_folders, KeyKind::I18n, FLUENT_EXTENSION),
        ] {
            for folder in folders {
                collect(&base, &base.join(folder), (kind, extension), &mut files);
            }
        }
        files.sort_by(|one, two| one.0.cmp(&two.0));
        files.truncate(limit);
        Ok(files
            .iter()
            .flat_map(|(file, kind)| keys_of(&base, file, *kind, max_bytes))
            .collect())
    }
}

/// Adds the files with `extension` below `folder`, if `folder` lies inside `base`.
fn collect(
    base: &Path,
    folder: &Path,
    (kind, extension): (KeyKind, &str),
    out: &mut Vec<(PathBuf, KeyKind)>,
) {
    let Ok(real) = fs::canonicalize(folder) else {
        return;
    };
    if !real.starts_with(base) {
        return;
    }
    let Ok(entries) = fs::read_dir(&real) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind_of) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind_of.is_dir() {
            collect(base, &path, (kind, extension), out);
        } else if kind_of.is_file() && path.extension().is_some_and(|found| found == extension) {
            out.push((path, kind));
        }
    }
}

/// The keys of one file; nothing if it cannot be read or is larger than `max_bytes`.
fn keys_of(base: &Path, file: &Path, kind: KeyKind, max_bytes: u64) -> Vec<KeyEntry> {
    let Some(path) = relative_path(base, file) else {
        return Vec::new();
    };
    let mut text = String::new();
    let read = File::open(file).and_then(|opened| {
        opened
            .take(max_bytes.saturating_add(1))
            .read_to_string(&mut text)
    });
    if read.is_err() || u64::try_from(text.len()).unwrap_or(u64::MAX) > max_bytes {
        return Vec::new();
    }
    let names = match kind {
        KeyKind::Config => toml_keys(&text),
        KeyKind::I18n => fluent_ids(&text),
    };
    names
        .into_iter()
        .map(|(name, line)| entry(name, kind, &path, line))
        .collect()
}

fn entry(name: String, kind: KeyKind, path: &RelativePath, line: usize) -> KeyEntry {
    KeyEntry {
        name,
        kind,
        path: path.clone(),
        line: u32::try_from(line).unwrap_or(u32::MAX),
    }
}

/// The dotted key paths of a TOML text with their zero-based lines. Section headers set the
/// prefix; a line `key = value` outside a multi-line string is a key. Values are never read.
fn toml_keys(text: &str) -> Vec<(String, usize)> {
    let mut prefix = String::new();
    let mut found = Vec::new();
    for (line, raw) in text.split(LINE_END_CHAR).enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with(COMMENT_START) || trimmed.is_empty() {
            continue;
        }
        if let Some(header) = trimmed.strip_prefix(TABLE_OPEN) {
            let name = header
                .trim_start_matches(TABLE_OPEN)
                .split(TABLE_CLOSE)
                .next()
                .unwrap_or_default();
            name.trim().clone_into(&mut prefix);
        } else if let Some((key, _)) = trimmed.split_once(ASSIGNMENT) {
            let key = key.trim().trim_matches(QUOTE);
            if !key.is_empty() && !key.contains(char::is_whitespace) {
                let full = if prefix.is_empty() {
                    key.to_owned()
                } else {
                    [prefix.as_str(), key].join(KEY_SEPARATOR)
                };
                found.push((full, line));
            }
        }
    }
    found
}

/// The ids of the messages of a Fluent text with their zero-based lines: a line that starts with
/// a letter, then name characters, then `=`. Terms (`-brand`), attributes and comments are not.
fn fluent_ids(text: &str) -> Vec<(String, usize)> {
    text.split(LINE_END_CHAR)
        .enumerate()
        .filter_map(|(line, raw)| {
            let (id, _) = raw.split_once(ASSIGNMENT)?;
            let id = id.trim_end();
            let mut chars = id.chars();
            let valid = chars
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || NAME_EXTRA.contains(c));
            valid.then(|| (id.to_owned(), line))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_keys_get_their_section_as_prefix() {
        let text = "top = 1\n# comment = 2\n[mcp]\ndefault_limit = 20\n[[rows]]\nname = \"x\"\n";
        assert_eq!(
            toml_keys(text),
            [
                ("top".to_owned(), 0),
                ("mcp.default_limit".to_owned(), 3),
                ("rows.name".to_owned(), 5)
            ]
        );
    }

    #[test]
    fn fluent_ids_skip_terms_attributes_and_comments() {
        let text = "# note = x\nerror-MCP-501 = text\n-brand = Name\nhello =\n    .title = t\n";
        assert_eq!(
            fluent_ids(text),
            [("error-MCP-501".to_owned(), 1), ("hello".to_owned(), 3)]
        );
    }

    #[test]
    fn only_configured_folders_below_the_root_are_read() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, text: &str| {
            let target = dir.path().join(name);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, text).unwrap();
        };
        write("config/default.toml", "[a]\nb = 1\n");
        write("locales/de/x.ftl", "hello = Hallo\n");
        write("other/secret.toml", "leak = 1\n");
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("out.toml"), "escaped = 1\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), dir.path().join("config/link")).unwrap();
        let sources = KeySearch {
            config_folders: vec!["config".to_owned(), "../".to_owned()],
            i18n_folders: vec!["locales".to_owned(), "missing".to_owned()],
            max_files: 10,
            max_file_kb: 1,
        };
        let root = FolderPath::new(dir.path().to_str().unwrap()).unwrap();
        let found = FolderKeys.keys(&root, &sources).unwrap();
        let names: Vec<&str> = found.iter().map(|key| key.name.as_str()).collect();
        assert_eq!(names, ["a.b", "hello"]);
        assert_eq!(found[1].path.as_str(), "locales/de/x.ftl");
    }
}
