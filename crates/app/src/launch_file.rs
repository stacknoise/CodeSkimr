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
use std::mem;
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
        let dir = private_dir(dir).map_err(unavailable)?;
        let dir = dir.as_path();
        remove_stale(
            dir,
            lifetime + Duration::from_secs(LAUNCH_FILE_GRACE_SECONDS),
        );
        let path = dir.join(random_name()?);
        let mut file = create_private(&path).map_err(unavailable)?;
        // From here on the file is removed again when this value is dropped, also on an error.
        let mut launch = Self { path };
        check_owner(dir, launch.path(), &file).map_err(unavailable)?;
        launch.path = opener_path(mem::take(&mut launch.path));
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

/// The path as the system opener and the user know it: Windows resolves a folder to a `\\?\C:\...`
/// path, which `explorer` does not reliably open. The file is removed through this path too.
fn opener_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        // A path that is not valid text stays as it is, so that it still names the file.
        path.to_str().map_or_else(
            || path.clone(),
            |text| PathBuf::from(crate::source_arg::opener_form(text)),
        )
    }
    #[cfg(not(windows))]
    path
}

/// Makes sure `dir` exists and nobody but the owner can enter it. Returns the folder to use from
/// here on: on Windows the folder resolved once, so that the access rules and the file go to the
/// place that was checked and not to whatever the name leads to a moment later.
fn private_dir(dir: &Path) -> std::io::Result<PathBuf> {
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
fn check_private(dir: &Path) -> std::io::Result<PathBuf> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::consts::{LAUNCH_DIR_MODE, OTHERS_MODE_MASK};

    let meta = fs::symlink_metadata(dir)?;
    if !meta.is_dir() {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    if meta.mode() & OTHERS_MODE_MASK != 0 {
        fs::set_permissions(dir, fs::Permissions::from_mode(LAUNCH_DIR_MODE))?;
    }
    Ok(dir.to_path_buf())
}

/// On Windows the folder gets access rules for the user alone: what it inherits from its parent
/// may let others in (a data folder below a drive root). The folder is resolved once, the rules
/// are set on the resolved path and that path is returned (see `still_there`).
#[cfg(windows)]
fn check_private(dir: &Path) -> std::io::Result<PathBuf> {
    if !fs::symlink_metadata(dir)?.is_dir() || is_reparse_point(dir)? {
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }
    let resolved = fs::canonicalize(dir)?;
    // Before the rules are set: a folder of someone else, or below a folder of someone else, is
    // refused as it is. Nothing is changed and no file is made in a place that another account
    // could lead elsewhere.
    crate::windows_owner::check_folder(&resolved)?;
    crate::windows_owner::check_parents(&resolved)?;
    crate::private_acl::restrict_to_user(&resolved)?;
    Ok(resolved)
}

/// Other systems without a Unix mode only need a folder.
#[cfg(not(any(unix, windows)))]
fn check_private(dir: &Path) -> std::io::Result<PathBuf> {
    if !fs::symlink_metadata(dir)?.is_dir() {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    Ok(dir.to_path_buf())
}

/// The folder and everything above it must belong to the user (the file just created tells who
/// that is) or to the administrator, and nobody else may be able to replace a part of the path:
/// a folder that others can write to is only fine if it is sticky. The path is checked with all
/// links resolved, so a link that someone else put in the way is seen for what it points to.
#[cfg(unix)]
fn check_owner(dir: &Path, path: &Path, own_file: &File) -> std::io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::consts::{OTHERS_WRITE_MASK, ROOT_UID, STICKY_MASK};

    let own = own_file.metadata()?;
    let user = own.uid();
    let refused = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    let resolved = dir.canonicalize()?;
    // The file must live in the folder that was resolved and checked, not in one that a link
    // pointed to a moment ago: both names must lead to the file that was opened.
    let name = path.file_name().ok_or_else(refused)?;
    let found = fs::metadata(resolved.join(name))?;
    if found.dev() != own.dev() || found.ino() != own.ino() {
        return Err(refused());
    }
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

/// On Windows the folder and the file themselves are checked, and the folder must still resolve
/// to the path that was resolved before the access rules were set (see `still_there`). The path
/// of the file must lead to the file that was opened, by the identity the system gives the file
/// (see `is_open_file`): a folder that was swapped while the file was made and swapped back is seen
/// by that, because the file is then not where the path says. This narrows the window a lot but
/// does not close it: an attacker who controls a folder above and toggles the redirect in step with
/// the checks can still pass; closing that needs the place of the open handle (issue #372). The
/// owner of the folder and of the file must be the user, the administrators or the system (see
/// `windows_owner`), and so must the owner of every folder above it up to the drive root, because
/// the owner of a folder above can replace a part of the path. Not checked there: whether the
/// access rules of a folder above let others write to it (the Unix check does look at that). The
/// access rules of the folder are set for the user alone
/// (see `check_private`). A folder or file that is a reparse point (a junction or a link) is
/// refused: it could lead to a place that others can read, and the access rules would be set on
/// that place instead.
#[cfg(windows)]
fn check_owner(dir: &Path, path: &Path, own_file: &File) -> std::io::Result<()> {
    if is_reparse_point(dir)? || is_reparse_point(path)? {
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }
    still_there(dir)?;
    crate::windows_owner::check_folder(dir)?;
    crate::windows_owner::check_parents(dir)?;
    crate::windows_owner::check_handle(own_file)?;
    is_open_file(own_file, path)
}

/// Whether `path` leads to the very file that `open` holds open.
#[cfg(windows)]
fn is_open_file(open: &File, path: &Path) -> std::io::Result<()> {
    let held = same_file::Handle::from_file(open.try_clone()?)?;
    if held == same_file::Handle::from_path(path)? {
        Ok(())
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

/// The folder that was resolved before its access rules were set must resolve to itself still:
/// if a part of its path was turned into a link since, it leads somewhere else now and the file
/// was created there. A swap that is undone before this check is not seen; closing that needs the
/// open file (issue #372).
#[cfg(windows)]
fn still_there(resolved: &Path) -> std::io::Result<()> {
    if fs::canonicalize(resolved)? == resolved {
        Ok(())
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

/// Whether `path` itself (not what it points to) is a reparse point.
#[cfg(windows)]
fn is_reparse_point(path: &Path) -> std::io::Result<bool> {
    use std::os::windows::fs::MetadataExt;

    use crate::consts::FILE_ATTRIBUTE_REPARSE_POINT;

    Ok(fs::symlink_metadata(path)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
}

/// Other systems without a Unix mode have nothing more to check.
#[cfg(not(any(unix, windows)))]
fn check_owner(_dir: &Path, _path: &Path, _own_file: &File) -> std::io::Result<()> {
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

    /// A launch file is refused for a folder that is a link, and nothing is made at its target.
    #[cfg(any(unix, windows))]
    fn assert_link_refused(link: &Path, target: &Path) {
        let error = LaunchFile::create(link, URL, LIFETIME).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::LaunchFileUnavailable
        );
        assert_eq!(fs::read_dir(target).unwrap().count(), 0);
    }

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
        assert_link_refused(&link, &target);
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

    #[cfg(unix)]
    #[test]
    fn a_file_in_another_folder_than_the_checked_one_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let file = create_private(&other.path().join("x.html")).unwrap();
        let claimed = dir.path().join("x.html");
        let error = check_owner(dir.path(), &claimed, &file).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        std::fs::write(&claimed, "other").unwrap();
        let error = check_owner(dir.path(), &claimed, &file).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
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

    /// A junction is made with the system shell: the standard library cannot create one.
    #[cfg(windows)]
    fn junction(link: &Path, target: &Path) {
        let made = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(made.success());
    }

    #[cfg(windows)]
    #[test]
    fn a_junction_as_the_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        let link = dir.path().join("link");
        junction(&link, &target);
        assert_link_refused(&link, &target);
    }

    #[cfg(windows)]
    #[test]
    fn the_file_is_made_in_a_plain_folder_and_goes_on_windows() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("launch");
        let file = LaunchFile::create(&folder, URL, LIFETIME).unwrap();
        assert!(!is_reparse_point(&folder).unwrap());
        let path = file.path().to_owned();
        assert!(path.is_file());
        drop(file);
        assert!(!path.exists());
    }

    #[cfg(windows)]
    #[test]
    fn a_folder_that_leads_elsewhere_now_is_not_the_one_that_was_checked() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("launch");
        fs::create_dir(&folder).unwrap();
        let resolved = fs::canonicalize(&folder).unwrap();
        assert!(still_there(&resolved).is_ok());
        let elsewhere = dir.path().join("elsewhere");
        fs::create_dir(&elsewhere).unwrap();
        fs::remove_dir(&folder).unwrap();
        junction(&folder, &elsewhere);
        assert!(still_there(&resolved).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn a_junction_above_the_folder_is_followed_once_and_the_path_is_plain() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        let parent = dir.path().join("parent");
        junction(&parent, &target);
        let file = LaunchFile::create(&parent.join("launch"), URL, LIFETIME).unwrap();
        assert!(file.path().is_file());
        assert!(!file.path().to_string_lossy().starts_with("\\\\?\\"));
        assert!(target.join("launch").is_dir());
    }

    #[cfg(windows)]
    #[test]
    fn a_parent_that_was_turned_into_a_junction_is_seen() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("parent");
        fs::create_dir_all(parent.join("launch")).unwrap();
        let resolved = fs::canonicalize(parent.join("launch")).unwrap();
        let elsewhere = dir.path().join("elsewhere");
        fs::create_dir_all(elsewhere.join("launch")).unwrap();
        fs::remove_dir_all(&parent).unwrap();
        junction(&parent, &elsewhere);
        assert!(still_there(&resolved).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn a_path_that_leads_to_another_file_is_not_the_open_one() {
        let dir = tempfile::tempdir().unwrap();
        let open = dir.path().join("open");
        let other = dir.path().join("other");
        let held = File::create(&open).unwrap();
        File::create(&other).unwrap();
        assert!(is_open_file(&held, &open).is_ok());
        assert!(is_open_file(&held, &other).is_err());
    }
}
