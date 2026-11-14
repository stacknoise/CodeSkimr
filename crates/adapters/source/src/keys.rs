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

use crate::consts::{BYTES_PER_KB, CONFIG_EXTENSION, FLUENT_EXTENSION};
use crate::key_text::{fluent_ids, toml_keys};
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
        let walk = Walk {
            base: &base,
            max_files: limit,
            max_depth: sources.max_depth,
        };
        for (folders, kind, extension) in [
            (&sources.config_folders, KeyKind::Config, CONFIG_EXTENSION),
            (&sources.i18n_folders, KeyKind::I18n, FLUENT_EXTENSION),
        ] {
            for folder in folders {
                walk.collect(&base.join(folder), (kind, extension), (0, &mut files));
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

/// What the walk of the key folders keeps to.
struct Walk<'a> {
    base: &'a Path,
    /// The walk stops once this many files are found.
    max_files: usize,
    /// Folders below a configured folder that are looked into.
    max_depth: u32,
}

impl Walk<'_> {
    /// Adds the files with `extension` below `folder`, if `folder` lies inside the root. Entries
    /// are visited in name order, so the files that are kept do not depend on the file system.
    fn collect(
        &self,
        folder: &Path,
        (kind, extension): (KeyKind, &str),
        (depth, out): (u32, &mut Vec<(PathBuf, KeyKind)>),
    ) {
        let Ok(real) = fs::canonicalize(folder) else {
            return;
        };
        if !real.starts_with(self.base) {
            return;
        }
        let Ok(entries) = fs::read_dir(&real) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            if out.len() >= self.max_files {
                return;
            }
            let Ok(kind_of) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind_of.is_dir() && depth < self.max_depth {
                self.collect(&path, (kind, extension), (depth + 1, out));
            } else if kind_of.is_file() && path.extension().is_some_and(|found| found == extension)
            {
                out.push((path, kind));
            }
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
        // A file that was swapped for a link since the walk must not lead out of the root.
        if !fs::canonicalize(file).is_ok_and(|real| real.starts_with(base)) {
            return Err(std::io::ErrorKind::PermissionDenied.into());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const CANARY: &str = "CanaryValue9f3a";

    fn search() -> KeySearch {
        KeySearch {
            config_folders: vec!["config".to_owned(), "../".to_owned()],
            i18n_folders: vec!["locales".to_owned(), "missing".to_owned()],
            max_files: 10,
            max_file_kb: 1,
            max_depth: 2,
        }
    }

    fn write(dir: &Path, name: &str, text: &str) {
        let target = dir.join(name);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, text).unwrap();
    }

    fn keys_in(dir: &Path, search: &KeySearch) -> Vec<KeyEntry> {
        let root = FolderPath::new(dir.to_str().unwrap()).unwrap();
        FolderKeys.keys(&root, search).unwrap()
    }

    #[test]
    fn only_configured_folders_below_the_root_are_read() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "config/default.toml", "[a]\nb = 1\n");
        write(dir.path(), "locales/de/x.ftl", "hello = Hallo\n");
        write(dir.path(), "other/secret.toml", "leak = 1\n");
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "out.toml", "escaped = 1\n");
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), dir.path().join("config/link")).unwrap();
        let found = keys_in(dir.path(), &search());
        let names: Vec<&str> = found.iter().map(|key| key.name.as_str()).collect();
        assert_eq!(names, ["a.b", "hello"]);
        assert_eq!(found[1].path.as_str(), "locales/de/x.ftl");
    }

    #[test]
    fn values_never_reach_the_answer() {
        let dir = tempfile::tempdir().unwrap();
        let toml = format!(
            "[auth]\ntoken = \"{CANARY}\"\nnote = \"\"\"\npassword = {CANARY}\n\"\"\"\nlist = [\n  \"key = {CANARY}\",\n]\n"
        );
        write(dir.path(), "config/app.toml", &toml);
        write(
            dir.path(),
            "locales/en/m.ftl",
            &format!("greeting = {CANARY}\n"),
        );
        let found = keys_in(dir.path(), &search());
        let shown = format!("{found:?}");
        assert!(!shown.contains(CANARY), "{shown}");
        let names: Vec<&str> = found.iter().map(|key| key.name.as_str()).collect();
        assert_eq!(names, ["auth.token", "auth.note", "auth.list", "greeting"]);
    }

    #[test]
    fn the_walk_stops_at_the_file_limit_and_the_depth_limit() {
        let dir = tempfile::tempdir().unwrap();
        for n in 0..5 {
            write(dir.path(), &format!("config/f{n}.toml"), "k = 1\n");
        }
        write(dir.path(), "config/a/b/c/deep.toml", "deep = 1\n");
        let few = KeySearch {
            max_files: 3,
            ..search()
        };
        assert_eq!(keys_in(dir.path(), &few).len(), 3);
        let all = keys_in(dir.path(), &search());
        assert!(all.iter().all(|key| key.name != "deep"), "{all:?}");
    }
}
