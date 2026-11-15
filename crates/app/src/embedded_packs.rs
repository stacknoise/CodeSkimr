//! The language packs inside the program (see `build.rs`). When no folder with packs exists, the
//! program writes its own packs once to the data folder and loads them from there, so that one
//! executable is enough. A folder next to the program, in the current folder, the flag
//! `--languages` and the variable `CODESKIMR_LANGUAGES_DIR` still win.

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use codeskimr_application::error::CodedError;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;

use crate::consts::{
    ENV_LANGUAGES_DIR, LANGUAGES_DIR, PACKS_CACHE_DIR, PACKS_COMPLETE_MARKER,
    PACKS_REPLACED_MAX_AGE, PACKS_STAGING_MAX_AGE, PACKS_STAGING_SEPARATOR,
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
    files: Files<'_>,
    fingerprint: &str,
) -> Result<Locations, CodedError> {
    let named = global.languages_dir.is_some() || env(ENV_LANGUAGES_DIR).is_some();
    if named || beside_program(&locations) || files.is_empty() {
        return Ok(locations);
    }
    let languages_dir = materialize(files, fingerprint, &locations.data_dir)?;
    remove_stale(&locations.data_dir.join(PACKS_CACHE_DIR), &languages_dir);
    Ok(Locations {
        languages_dir,
        ..locations
    })
}

/// True if a folder with packs lies next to the program: the layout of a deliberate installation.
/// A `languages/` folder in the current folder does not count, because it may belong to the
/// scanned repository and must not replace the packs of the program without the user asking.
fn beside_program(locations: &Locations) -> bool {
    locations
        .bases
        .first()
        .is_some_and(|base| base.join(LANGUAGES_DIR).is_dir())
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
        // Touches the marker: the folder is in use, so it is not old (see `remove_stale`).
        let _ = fs::write(target.join(PACKS_COMPLETE_MARKER), []);
        return Ok(target);
    }
    let mut staging_name = OsString::from(fingerprint);
    staging_name.push(PACKS_STAGING_SEPARATOR);
    staging_name.push(std::process::id().to_string());
    let staging = cache.join(staging_name);
    write_all(files, &staging).map_err(unavailable)?;
    if is_complete(&target) {
        // Another start finished meanwhile and may be reading its folder: leave it alone.
        let _ = fs::remove_dir_all(&staging);
        return Ok(target);
    }
    // A folder without the marker is the rest of an older, interrupted start: it is replaced.
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

/// Removes what earlier starts left in `cache` and the current program does not use: unfinished
/// folders of crashed starts (younger ones may belong to a start that is writing right now) and
/// folders of programs that were replaced long ago. `keep` is never removed. Failures do not
/// matter: the next start tries again.
fn remove_stale(cache: &Path, keep: &Path) {
    let Ok(entries) = fs::read_dir(cache) else {
        return;
    };
    for entry in entries.flatten() {
        let unfinished = entry
            .file_name()
            .to_string_lossy()
            .contains(PACKS_STAGING_SEPARATOR);
        let limit = if unfinished {
            PACKS_STAGING_MAX_AGE
        } else {
            PACKS_REPLACED_MAX_AGE
        };
        // The marker is rewritten at every start that uses the folder; unfinished folders have none.
        let old = fs::metadata(entry.path().join(PACKS_COMPLETE_MARKER))
            .or_else(|_| entry.metadata())
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|time| time.elapsed().ok())
            .is_some_and(|age| age >= limit);
        if old && entry.path() != keep {
            let _ = fs::remove_dir_all(entry.path());
        }
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
        write_synced(&path, content)?;
    }
    fs::create_dir_all(folder)?;
    // The marker comes last and only after the files are on disk, so that it never vouches for
    // empty files after a power loss.
    write_synced(&folder.join(PACKS_COMPLETE_MARKER), &[])
}

fn write_synced(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content)?;
    file.sync_all()
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

    /// Locations whose program folder is `program` and whose current folder is `cwd`.
    fn with_bases(program: &Path, cwd: &Path, data: &Path) -> Locations {
        Locations {
            bases: vec![program.to_path_buf(), cwd.to_path_buf()],
            ..locations(program.join(LANGUAGES_DIR), data.to_path_buf())
        }
    }

    fn prefer(global: &GlobalOptions, env: Lookup<'_>, existing: &Path, data: &Path) -> Locations {
        let original = locations(existing.to_path_buf(), data.to_path_buf());
        prefer_embedded(original, global, env, FILES_OF_TWO_PACKS, "fp").unwrap()
    }

    #[test]
    fn without_embedded_packs_the_locations_stay_as_they_are() {
        let data = tempfile::tempdir().unwrap();
        let original = locations(
            PathBuf::from("/nowhere/languages"),
            data.path().to_path_buf(),
        );
        let kept = prefer_embedded(
            original.clone(),
            &GlobalOptions::default(),
            &|_| None,
            &[],
            "fp",
        )
        .unwrap();
        assert_eq!(kept, original);
    }

    #[test]
    fn a_folder_the_user_named_is_kept() {
        let data = tempfile::tempdir().unwrap();
        let missing = Path::new("/nowhere/languages");
        let flag = GlobalOptions {
            languages_dir: Some(PathBuf::from("/flag")),
            ..GlobalOptions::default()
        };
        let by_env = |name: &str| (name == ENV_LANGUAGES_DIR).then(|| "/env".to_owned());
        assert_eq!(
            prefer(&flag, &|_| None, missing, data.path()).languages_dir,
            missing
        );
        assert_eq!(
            prefer(&GlobalOptions::default(), &by_env, missing, data.path()).languages_dir,
            missing
        );
    }

    fn prefer_at(locations: Locations) -> Locations {
        prefer_embedded(
            locations,
            &GlobalOptions::default(),
            &|_| None,
            FILES_OF_TWO_PACKS,
            "fp",
        )
        .unwrap()
    }

    #[test]
    fn a_folder_next_to_the_program_is_kept() {
        let (program, cwd, data) = (
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
        );
        fs::create_dir(program.path().join(LANGUAGES_DIR)).unwrap();
        let found = prefer_at(with_bases(program.path(), cwd.path(), data.path()));
        assert_eq!(found.languages_dir, program.path().join(LANGUAGES_DIR));
        assert!(!data.path().join(PACKS_CACHE_DIR).exists());
    }

    #[test]
    fn a_folder_in_the_current_folder_does_not_replace_the_packs_of_the_program() {
        let (program, cwd, data) = (
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
        );
        // What `Locations::resolve` finds when only the current folder has `languages/`.
        fs::create_dir(cwd.path().join(LANGUAGES_DIR)).unwrap();
        let mut found = with_bases(program.path(), cwd.path(), data.path());
        found.languages_dir = cwd.path().join(LANGUAGES_DIR);
        let found = prefer_at(found);
        assert_eq!(
            found.languages_dir,
            data.path().join(PACKS_CACHE_DIR).join("fp")
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_folder_of_a_long_replaced_program_is_removed_but_not_the_current_one() {
        let data = tempfile::tempdir().unwrap();
        let old = materialize(FILES_OF_TWO_PACKS, "old", data.path()).unwrap();
        let current = materialize(FILES_OF_TWO_PACKS, "cur", data.path()).unwrap();
        let long_ago = std::time::SystemTime::now() - PACKS_REPLACED_MAX_AGE * 2;
        for folder in [&old, &current] {
            fs::File::open(folder.join(PACKS_COMPLETE_MARKER))
                .unwrap()
                .set_modified(long_ago)
                .unwrap();
        }
        remove_stale(&data.path().join(PACKS_CACHE_DIR), &current);
        assert!(!old.exists());
        assert!(current.is_dir());
    }

    #[test]
    fn without_any_folder_the_embedded_packs_are_written_and_used() {
        let data = tempfile::tempdir().unwrap();
        let found = prefer(
            &GlobalOptions::default(),
            &|_| None,
            Path::new("/nowhere/languages"),
            data.path(),
        );
        assert_eq!(
            found.languages_dir,
            data.path().join(PACKS_CACHE_DIR).join("fp")
        );
        assert!(is_complete(&found.languages_dir));
    }

    #[test]
    fn a_folder_that_became_complete_meanwhile_is_not_deleted() {
        let data = tempfile::tempdir().unwrap();
        let first = materialize(FILES_OF_TWO_PACKS, "abc", data.path()).unwrap();
        fs::write(first.join("go/language.toml"), b"in use").unwrap();
        // A second start that found nothing before now sees the finished folder of the first.
        let second = materialize(FILES_OF_TWO_PACKS, "abc", data.path()).unwrap();
        assert_eq!(
            fs::read(second.join("go/language.toml")).unwrap(),
            b"in use"
        );
    }

    #[test]
    fn a_fresh_unfinished_folder_of_another_start_is_kept() {
        let data = tempfile::tempdir().unwrap();
        let busy = data.path().join(PACKS_CACHE_DIR).join("old.writing-1");
        fs::create_dir_all(&busy).unwrap();
        materialize(FILES_OF_TWO_PACKS, "new", data.path()).unwrap();
        remove_stale(&data.path().join(PACKS_CACHE_DIR), Path::new("-"));
        assert!(busy.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn an_old_unfinished_folder_is_removed() {
        let data = tempfile::tempdir().unwrap();
        let left = data.path().join(PACKS_CACHE_DIR).join("old.writing-1");
        fs::create_dir_all(&left).unwrap();
        let long_ago = std::time::SystemTime::now() - PACKS_STAGING_MAX_AGE * 2;
        fs::File::open(&left)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
        remove_stale(&data.path().join(PACKS_CACHE_DIR), Path::new("-"));
        assert!(!left.exists());
    }
}
