//! The private file that carries the launch code to the browser (ADR 0028).
//!
//! The code must not appear in a process list, so it is not passed on a command line. It is
//! written into a small page in a folder that only the user can enter, under a name nobody can
//! guess, and the browser opens that file. The page redirects to the server; the code is in the
//! address fragment, which the browser never sends anywhere.
//!
//! The file is created exclusively (`create_new`): an existing name, also a link that someone has
//! put there, makes the creation fail instead of being followed or overwritten.

use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use zeroize::Zeroizing;

use crate::consts::{
    HEX_DIGIT_BITS, HEX_LOW_MASK, HEX_RADIX, LAUNCH_FILE_GRACE_SECONDS, LAUNCH_FILE_PREFIX,
    LAUNCH_FILE_RANDOM_BYTES, LAUNCH_FILE_SUFFIX, LAUNCH_PAGE_HEAD, LAUNCH_PAGE_TAIL,
};

/// A launch file. It is removed when the value is dropped.
#[derive(Debug)]
pub struct LaunchFile {
    path: PathBuf,
}

impl LaunchFile {
    /// Creates a file in `dir` (made private if needed) whose page redirects to `url`. Files that
    /// an earlier run left behind are removed first, once their code has certainly expired.
    ///
    /// # Errors
    /// `APP-202` if the folder cannot be made private or the file cannot be created.
    pub fn create(dir: &Path, url: &str, lifetime: Duration) -> Result<Self, CodedError> {
        let unavailable = |_| CodedError::new(ErrorCode::LaunchFileUnavailable);
        private_dir(dir).map_err(unavailable)?;
        remove_stale(
            dir,
            lifetime + Duration::from_secs(LAUNCH_FILE_GRACE_SECONDS),
        );
        let path = dir.join(random_name()?);
        let mut file = create_private(&path).map_err(unavailable)?;
        let launch = Self { path };
        check_owner(dir, &file).map_err(unavailable)?;
        let page = Zeroizing::new(format!("{LAUNCH_PAGE_HEAD}{url}{LAUNCH_PAGE_TAIL}"));
        file.write_all(page.as_bytes()).map_err(unavailable)?;
        Ok(launch)
    }

    /// Where the file is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LaunchFile {
    fn drop(&mut self) {
        // The file is gone or cannot be removed; either way there is nobody to tell.
        let _ = fs::remove_file(&self.path);
    }
}

/// Makes sure `dir` exists and nobody but the owner can enter it.
fn private_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, crate::consts::LAUNCH_DIR_MODE);
    builder.create(dir)?;
    check_private(dir)
}

/// A folder that is a link, or that others may enter, is refused: a launch file there could be
/// read by someone else.
#[cfg(unix)]
fn check_private(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::consts::{LAUNCH_DIR_MODE, OTHERS_MODE_MASK};

    let meta = fs::symlink_metadata(dir)?;
    if !meta.is_dir() {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    if meta.mode() & OTHERS_MODE_MASK != 0 {
        fs::set_permissions(dir, fs::Permissions::from_mode(LAUNCH_DIR_MODE))?;
    }
    Ok(())
}

/// On Windows the folder gets access rules for the user alone: what it inherits from its parent
/// may let others in (a data folder below a drive root).
#[cfg(not(unix))]
fn check_private(dir: &Path) -> std::io::Result<()> {
    if !fs::symlink_metadata(dir)?.is_dir() {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    #[cfg(windows)]
    crate::private_acl::restrict_to_user(dir)?;
    Ok(())
}

/// The folder and everything above it must belong to the user (the file just created tells who
/// that is) or to the administrator, and nobody else may be able to replace a part of the path:
/// a folder that others can write to is only fine if it is sticky. The path is checked with all
/// links resolved, so a link that someone else put in the way is seen for what it points to.
#[cfg(unix)]
fn check_owner(dir: &Path, own_file: &File) -> std::io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::consts::{OTHERS_WRITE_MASK, ROOT_UID, STICKY_MASK};

    let user = own_file.metadata()?.uid();
    let refused = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    let resolved = dir.canonicalize()?;
    if fs::metadata(&resolved)?.uid() != user {
        return Err(refused());
    }
    for part in resolved.ancestors().skip(1) {
        let meta = fs::metadata(part)?;
        let known_owner = meta.uid() == user || meta.uid() == ROOT_UID;
        let mode = meta.permissions().mode();
        let open_to_others = mode & OTHERS_WRITE_MASK != 0 && mode & STICKY_MASK == 0;
        if !known_owner || open_to_others {
            return Err(refused());
        }
    }
    Ok(())
}

/// On Windows the access rules of the folder are set for the user alone (see `check_private`).
#[cfg(not(unix))]
fn check_owner(_dir: &Path, _own_file: &File) -> std::io::Result<()> {
    Ok(())
}

fn create_private(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, crate::consts::LAUNCH_FILE_MODE);
    options.open(path)
}

/// A name that cannot be guessed: 128 random bits.
fn random_name() -> Result<String, CodedError> {
    let mut bytes = [u8::default(); LAUNCH_FILE_RANDOM_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| CodedError::new(ErrorCode::LaunchFileUnavailable))?;
    let mut name = String::from(LAUNCH_FILE_PREFIX);
    for byte in bytes {
        name.extend(char::from_digit(
            u32::from(byte >> HEX_DIGIT_BITS),
            HEX_RADIX,
        ));
        name.extend(char::from_digit(u32::from(byte & HEX_LOW_MASK), HEX_RADIX));
    }
    name.push_str(LAUNCH_FILE_SUFFIX);
    Ok(name)
}

/// Removes launch files older than `age`. Only files with the name of a launch file are touched.
fn remove_stale(dir: &Path, age: Duration) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let is_launch_file =
            name.starts_with(LAUNCH_FILE_PREFIX) && name.ends_with(LAUNCH_FILE_SUFFIX);
        let is_old = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|elapsed| elapsed > age);
        if is_launch_file && is_old {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "http://127.0.0.1:7421/#code=abc";
    const LIFETIME: Duration = Duration::from_secs(60);

    #[test]
    fn the_page_redirects_and_the_file_goes_with_the_value() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("launch");
        let file = LaunchFile::create(&folder, URL, LIFETIME).unwrap();
        let path = file.path().to_owned();
        let page = fs::read_to_string(&path).unwrap();
        assert!(page.contains(&format!("url={URL}")), "{page}");
        assert!(!page.contains("<script"), "{page}");
        drop(file);
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn file_and_folder_are_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("launch");
        let file = LaunchFile::create(&folder, URL, LIFETIME).unwrap();
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(file.path()), 0o600);
        assert_eq!(mode(&folder), 0o700);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_that_others_can_enter_is_made_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        let file = LaunchFile::create(dir.path(), URL, LIFETIME).unwrap();
        let mode = fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
        drop(file);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_in_place_of_the_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("elsewhere");
        fs::create_dir(&target).unwrap();
        let link = dir.path().join("launch");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let error = LaunchFile::create(&link, URL, LIFETIME).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::LaunchFileUnavailable
        );
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_that_others_can_replace_is_refused() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let open = dir.path().join("open");
        fs::create_dir(&open).unwrap();
        fs::set_permissions(&open, fs::Permissions::from_mode(0o777)).unwrap();
        let error = LaunchFile::create(&open.join("launch"), URL, LIFETIME).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::LaunchFileUnavailable
        );
        fs::set_permissions(&open, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(LaunchFile::create(&open.join("launch"), URL, LIFETIME).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_in_place_of_the_file_is_never_followed() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("victim");
        fs::write(&target, "keep").unwrap();
        let link = dir.path().join("planted.html");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(create_private(&link).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "keep");
    }

    #[test]
    fn two_files_never_share_a_name() {
        let dir = tempfile::tempdir().unwrap();
        let first = LaunchFile::create(dir.path(), URL, LIFETIME).unwrap();
        let second = LaunchFile::create(dir.path(), URL, LIFETIME).unwrap();
        assert_ne!(first.path(), second.path());
        assert!(first.path().exists() && second.path().exists());
    }

    #[test]
    fn only_old_launch_files_are_removed() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir
            .path()
            .join(format!("{LAUNCH_FILE_PREFIX}old{LAUNCH_FILE_SUFFIX}"));
        let other = dir.path().join("notes.txt");
        fs::write(&old, "x").unwrap();
        fs::write(&other, "x").unwrap();
        remove_stale(dir.path(), Duration::ZERO);
        std::thread::sleep(Duration::from_millis(20));
        remove_stale(dir.path(), Duration::ZERO);
        assert!(!old.exists());
        assert!(other.exists());
        let fresh = LaunchFile::create(dir.path(), URL, LIFETIME).unwrap();
        assert!(fresh.path().exists());
    }
}
