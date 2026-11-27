//! The name of the file that a new content is written to before it replaces the real one.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::consts::{TEMP_SEPARATOR, TEMP_SUFFIX};

/// A name beside `path` that no other call and no earlier run has used. A random part keeps a
/// leftover of a crashed run from blocking the same pid and counter later.
pub(super) fn beside(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(u64::MIN);
    let mut bytes = [u8::MIN; size_of::<u64>()];
    let random = getrandom::fill(&mut bytes)
        .map(|()| u64::from_le_bytes(bytes))
        .unwrap_or_default();
    let unique = [
        std::process::id().to_string(),
        COUNTER.fetch_add(1, Ordering::Relaxed).to_string(),
        random.to_string(),
    ]
    .join(TEMP_SEPARATOR);
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(TEMP_SEPARATOR);
    name.push(unique);
    name.push(TEMP_SUFFIX);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::beside;

    #[test]
    fn the_names_beside_a_file_differ_and_stay_beside_it() {
        let path = Path::new("dir").join("file");
        let (first, second) = (beside(&path), beside(&path));
        assert_ne!(first, second);
        assert_eq!(first.parent(), path.parent());
    }
}
