//! Work folders that a crashed process left in the Git cache.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::consts::{WORK_PREFIX, WORK_SEPARATOR};

/// Removes the clone folders of fetches in which nothing changed for `max_age`. A running clone
/// keeps writing files below its folder (the folder itself changes only when entries are added
/// to it), so the newest change anywhere in the tree tells a crash from a slow fetch.
///
/// The parked copy of a swap (`<id>-.work-`) is the old clone and keeps its old modification
/// times, so they say nothing about when the swap happened. Its name carries the start of the
/// fetch instead: a parked copy whose fetch started `max_age` ago or earlier is a leftover of a
/// crash between parking and putting the new clone in place. If it was still in use, the cost is
/// one more clone. Failures are ignored: a folder that cannot be removed now is tried again by the
/// next fetch.
pub fn remove_left_behind(cache_root: &Path, max_age: Duration) {
    let Ok(entries) = fs::read_dir(cache_root) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(WORK_PREFIX) {
            continue;
        }
        let leftover = if let Some(id) = parked_id(&name) {
            started_before(id, now, max_age)
        } else {
            is_idle(&entry.path(), now, max_age)
        };
        if leftover {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

/// The work folder id inside the name of a parked copy (`<id>-.work-`), or `None` for any other name.
fn parked_id(name: &str) -> Option<&str> {
    name.strip_suffix(&format!("{WORK_SEPARATOR}{WORK_PREFIX}"))
}

/// The fetch that parked this copy started `max_age` ago or earlier. The start time (nanoseconds
/// since the epoch) is the last part of the work folder name; a name without a usable one is not
/// judged.
fn started_before(id: &str, now: SystemTime, max_age: Duration) -> bool {
    let started = id
        .rsplit(WORK_SEPARATOR)
        .next()
        .and_then(|text| text.parse::<u64>().ok())
        .and_then(|nanos| UNIX_EPOCH.checked_add(Duration::from_nanos(nanos)));
    started
        .and_then(|started| now.duration_since(started).ok())
        .is_some_and(|elapsed| elapsed >= max_age)
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
        let file = top.join(".git").join("objects").join("pack");
        let newer = SystemTime::now() + Duration::from_secs(10_000);
        fs::OpenOptions::new()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(newer)
            .unwrap();
        assert_eq!(
            newest_change(&top),
            Some(newer),
            "the file deep below the folder is the newest change"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_parked_copy_goes_by_the_start_of_its_fetch() {
        let root = cache_with_folders("parked");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        fs::create_dir_all(root.join(".work-7-ThreadId(1)-1000-.work-")).unwrap();
        fs::create_dir_all(root.join(format!(".work-7-ThreadId(2)-{now}-.work-"))).unwrap();
        remove_left_behind(&root, Duration::from_secs(3600));
        assert!(
            !root.join(".work-7-ThreadId(1)-1000-.work-").exists(),
            "a swap that began long ago was left by a crash"
        );
        assert!(
            root.join(format!(".work-7-ThreadId(2)-{now}-.work-"))
                .exists(),
            "a swap that began just now may still be running"
        );
        assert!(
            root.join(".work-1-x-.work-").exists(),
            "a name without a start time is not judged"
        );
        fs::create_dir_all(root.join(".work-7-ThreadId(3)-99999999999999999999999-.work-"))
            .unwrap();
        fs::create_dir_all(root.join(format!(".work-7-ThreadId(4)-{}-.work-", now * 2))).unwrap();
        remove_left_behind(&root, Duration::ZERO);
        assert!(
            root.join(".work-7-ThreadId(3)-99999999999999999999999-.work-")
                .exists(),
            "a number that does not fit is not judged"
        );
        assert!(
            root.join(format!(".work-7-ThreadId(4)-{}-.work-", now * 2))
                .exists(),
            "a start in the future is not judged"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_cache_root_is_no_error() {
        remove_left_behind(Path::new("/does/not/exist/anywhere"), Duration::ZERO);
    }
}
