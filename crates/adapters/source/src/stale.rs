//! Work folders that a crashed process left in the Git cache.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::consts::{WORK_PREFIX, WORK_SEPARATOR};

/// Removes the clone folders of fetches in which nothing changed for `max_age`. A running clone
/// keeps writing files below its folder (the folder itself changes only when entries are added
/// to it), so the newest change anywhere in the tree tells a crash from a slow fetch. The parked
/// copy of a swap (`<id>-.work-`) is left alone: it keeps the age of the old clone and is only
/// there for the moment of the swap. Failures are ignored: a folder that cannot be removed now is
/// tried again by the next fetch.
pub fn remove_left_behind(cache_root: &Path, max_age: Duration) {
    let Ok(entries) = fs::read_dir(cache_root) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_fresh_folder = name.starts_with(WORK_PREFIX) && !is_parked(&name);
        if is_fresh_folder && is_idle(&entry.path(), now, max_age) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

/// The name of a parked copy: the work folder name plus separator and prefix.
fn is_parked(name: &str) -> bool {
    let suffix = format!("{WORK_SEPARATOR}{WORK_PREFIX}");
    name.ends_with(&suffix)
}

/// Nothing at or below `path` changed for `max_age`. A tree that cannot be read counts as busy.
fn is_idle(path: &Path, now: SystemTime, max_age: Duration) -> bool {
    newest_change(path)
        .and_then(|newest| now.duration_since(newest).ok())
        .is_some_and(|age| age >= max_age)
}

/// The most recent modification time of `path` and everything below it.
fn newest_change(path: &Path) -> Option<SystemTime> {
    let own = fs::symlink_metadata(path)
        .and_then(|meta| meta.modified())
        .ok()?;
    let Ok(entries) = fs::read_dir(path) else {
        return Some(own);
    };
    let newest_below = entries
        .flatten()
        .filter_map(|entry| newest_change(&entry.path()))
        .fold(own, SystemTime::max);
    Some(newest_below)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cache with the work folders of one fetch (`.work-1-x`, with a file deep below), its
    /// parked copy and a finished clone.
    fn cache_with_folders(test: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("codeskimr-stale-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".work-1-x").join(".git").join("objects")).unwrap();
        fs::write(
            root.join(".work-1-x")
                .join(".git")
                .join("objects")
                .join("pack"),
            [0],
        )
        .unwrap();
        fs::create_dir_all(root.join(".work-1-x-.work-")).unwrap();
        fs::create_dir_all(root.join("abc123")).unwrap();
        root
    }

    #[test]
    fn an_idle_work_folder_goes_and_parked_copies_and_clones_stay() {
        let root = cache_with_folders("idle");
        remove_left_behind(&root, Duration::ZERO);
        assert!(!root.join(".work-1-x").exists());
        assert!(root.join(".work-1-x-.work-").exists());
        assert!(root.join("abc123").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_work_folder_with_recent_changes_below_it_stays() {
        let root = cache_with_folders("busy");
        remove_left_behind(&root, Duration::from_secs(3600));
        assert!(root.join(".work-1-x").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_newest_change_deep_in_the_tree_counts() {
        let root = cache_with_folders("deep");
        let top = root.join(".work-1-x");
        let old = SystemTime::now() - Duration::from_secs(10_000);
        fs::File::open(&top).unwrap().set_modified(old).unwrap();
        let recent = newest_change(&top).unwrap();
        assert!(
            recent > old,
            "the file below the folder is newer than the folder"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_cache_root_is_no_error() {
        remove_left_behind(Path::new("/does/not/exist/anywhere"), Duration::ZERO);
    }
}
