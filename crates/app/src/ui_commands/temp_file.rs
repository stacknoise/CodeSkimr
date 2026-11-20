//! The name of the file that a new content is written to before it replaces the real one.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::consts::{TEMP_SEPARATOR, TEMP_SUFFIX};

/// A name beside `path` that no other call and no earlier run has used.
pub(super) fn beside(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(u64::MIN);
    let unique = [
        std::process::id().to_string(),
        COUNTER.fetch_add(1, Ordering::Relaxed).to_string(),
    ]
    .join(TEMP_SEPARATOR);
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(TEMP_SEPARATOR);
    name.push(unique);
    name.push(TEMP_SUFFIX);
    PathBuf::from(name)
}
