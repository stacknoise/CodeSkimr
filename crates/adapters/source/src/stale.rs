//! Work folders that a crashed process left in the Git cache.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::consts::WORK_PREFIX;

/// Removes the work folders in `cache_root` that were last changed more than `max_age` ago. A
/// running clone keeps changing its folder, so only the leftovers of a crash match. Failures are
/// ignored: a folder that cannot be removed now is tried again by the next fetch.
pub fn remove_left_behind(cache_root: &Path, max_age: Duration) {
    let Ok(entries) = fs::read_dir(cache_root) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let is_work_folder = entry.file_name().to_string_lossy().starts_with(WORK_PREFIX);
        if is_work_folder && is_older_than(&entry, now, max_age) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

fn is_older_than(entry: &fs::DirEntry, now: SystemTime, max_age: Duration) -> bool {
    entry
        .metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age >= max_age)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_with_folders(test: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("codeskimr-stale-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".work-1-x").join("clone")).unwrap();
        fs::create_dir_all(root.join(".work-1-x-.work-")).unwrap();
        fs::create_dir_all(root.join("abc123")).unwrap();
        root
    }

    #[test]
    fn old_work_folders_go_and_the_clones_stay() {
        let root = cache_with_folders("old");
        remove_left_behind(&root, Duration::ZERO);
        assert!(!root.join(".work-1-x").exists());
        assert!(!root.join(".work-1-x-.work-").exists());
        assert!(root.join("abc123").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn recent_work_folders_stay() {
        let root = cache_with_folders("recent");
        remove_left_behind(&root, Duration::from_secs(3600));
        assert!(root.join(".work-1-x").exists());
        assert!(root.join(".work-1-x-.work-").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_cache_root_is_no_error() {
        remove_left_behind(Path::new("/does/not/exist/anywhere"), Duration::ZERO);
    }
}
