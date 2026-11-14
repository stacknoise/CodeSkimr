//! The language packs inside the program (see `build.rs`). When no folder with packs exists, the
//! program writes its own packs once to the data folder and loads them from there, so that one
//! executable is enough. A folder next to the program, in the current folder, the flag
//! `--languages` and the variable `CODESKIMR_LANGUAGES_DIR` still win.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_application::error::CodedError;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;

use crate::consts::{
    ENV_LANGUAGES_DIR, PACKS_CACHE_DIR, PACKS_COMPLETE_MARKER, PACKS_STAGING_SEPARATOR,
};
use crate::locations::{Locations, Lookup};

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/embedded_packs.rs"));

/// Unit tests never use the packs of the build, so their results do not depend on it.
#[cfg(test)]
pub static FILES: &[(&str, &[u8])] = &[];
/// Fingerprint of the packs; part of the folder name, so that a new program never reads old packs.
#[cfg(test)]
pub const FINGERPRINT: &str = "";

/// The packs as a table: path below the pack folder (always with `/`) and content.
pub type Files<'a> = &'a [(&'a str, &'a [u8])];

/// Replaces the pack folder of `locations` with the embedded packs if the user did not name a
/// folder and none exists next to the program or in the current folder.
///
/// # Errors
/// `APP-203` if the packs cannot be written to the data folder.
pub fn prefer_embedded(
    locations: Locations,
    global: &GlobalOptions,
    env: Lookup<'_>,
) -> Result<Locations, CodedError> {
    let named = global.languages_dir.is_some() || env(ENV_LANGUAGES_DIR).is_some();
    if named || locations.languages_dir.is_dir() || FILES.is_empty() {
        return Ok(locations);
    }
    let languages_dir = materialize(FILES, FINGERPRINT, &locations.data_dir)?;
    Ok(Locations {
        languages_dir,
        ..locations
    })
}

/// Writes `files` to `<data_dir>/<cache>/<fingerprint>` unless that folder is complete already, and
/// returns it. The packs are written to a folder of their own first and moved into place at once, so
/// that a program started at the same time, or an interrupted start, never sees half of them.
///
/// # Errors
/// `APP-203` if a file or folder cannot be written.
pub fn materialize(
    files: Files<'_>,
    fingerprint: &str,
    data_dir: &Path,
) -> Result<PathBuf, CodedError> {
    let unavailable = |_| CodedError::new(ErrorCode::LanguagePacksUnavailable);
    let cache = data_dir.join(PACKS_CACHE_DIR);
    let target = cache.join(fingerprint);
    if is_complete(&target) {
        return Ok(target);
    }
    let mut staging_name = OsString::from(fingerprint);
    staging_name.push(PACKS_STAGING_SEPARATOR);
    staging_name.push(std::process::id().to_string());
    let staging = cache.join(staging_name);
    write_all(files, &staging).map_err(unavailable)?;
    // A folder without the marker is the rest of an interrupted start: it is replaced.
    if target.exists() {
        let _ = fs::remove_dir_all(&target);
    }
    match fs::rename(&staging, &target) {
        Ok(()) => Ok(target),
        // Another start was faster: its folder is as good as ours.
        Err(_) if is_complete(&target) => {
            let _ = fs::remove_dir_all(&staging);
            Ok(target)
        }
        Err(error) => Err(unavailable(error)),
    }
}

fn is_complete(folder: &Path) -> bool {
    folder.join(PACKS_COMPLETE_MARKER).is_file()
}

/// Writes every file below `folder`, a new empty folder, and the marker last.
fn write_all(files: Files<'_>, folder: &Path) -> std::io::Result<()> {
    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }
    for (name, content) in files {
        let path = folder.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    fs::create_dir_all(folder)?;
    fs::write(folder.join(PACKS_COMPLETE_MARKER), [])
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILES_OF_TWO_PACKS: Files<'static> = &[
        ("rust/language.toml", b"id = 1"),
        ("rust/symbols.scm", b"(a)"),
        ("go/language.toml", b"id = 2"),
    ];

    #[test]
    fn the_packs_are_written_once_and_reused() {
        let data = tempfile::tempdir().unwrap();
        let first = materialize(FILES_OF_TWO_PACKS, "abc", data.path()).unwrap();
        assert_eq!(fs::read(first.join("rust/symbols.scm")).unwrap(), b"(a)");
        assert_eq!(fs::read(first.join("go/language.toml")).unwrap(), b"id = 2");
        // A change that the program does not know about is not overwritten by the second start.
        fs::write(first.join("go/language.toml"), b"changed").unwrap();
        let second = materialize(FILES_OF_TWO_PACKS, "abc", data.path()).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            fs::read(second.join("go/language.toml")).unwrap(),
            b"changed"
        );
    }

    #[test]
    fn another_fingerprint_gets_a_folder_of_its_own() {
        let data = tempfile::tempdir().unwrap();
        let old = materialize(FILES_OF_TWO_PACKS, "old", data.path()).unwrap();
        let new = materialize(FILES_OF_TWO_PACKS, "new", data.path()).unwrap();
        assert_ne!(old, new);
        assert!(is_complete(&old) && is_complete(&new));
    }

    #[test]
    fn an_unfinished_folder_is_not_used() {
        let data = tempfile::tempdir().unwrap();
        let target = data.path().join(PACKS_CACHE_DIR).join("abc");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("rust"), b"half").unwrap();
        let found = materialize(FILES_OF_TWO_PACKS, "abc", data.path()).unwrap();
        assert!(is_complete(&found));
    }

    #[test]
    fn a_data_folder_that_is_a_file_gives_the_code() {
        let data = tempfile::tempdir().unwrap();
        let blocked = data.path().join("file");
        fs::write(&blocked, b"x").unwrap();
        let error = materialize(FILES_OF_TWO_PACKS, "abc", &blocked).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::LanguagePacksUnavailable
        );
    }

    fn locations(languages_dir: PathBuf, data_dir: PathBuf) -> Locations {
        Locations {
            data_dir,
            languages_dir,
            user_config: None,
            bases: Vec::new(),
        }
    }

    #[test]
    fn without_embedded_packs_the_locations_stay_as_they_are() {
        // In unit tests the table is empty (see above): nothing to prefer.
        let data = tempfile::tempdir().unwrap();
        let original = locations(
            PathBuf::from("/nowhere/languages"),
            data.path().to_path_buf(),
        );
        let kept = prefer_embedded(original.clone(), &GlobalOptions::default(), &|_| None).unwrap();
        assert_eq!(kept, original);
    }
}
