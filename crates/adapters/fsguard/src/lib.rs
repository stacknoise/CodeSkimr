//! Opening a file that a build tool may have tampered with (issues #229 and #324).
//!
//! An indexer is project code. After it ran, the files that `CodeSkimr` reads next (its index, the
//! sources it is to compare) may have been replaced by a link that leads elsewhere or by a named
//! pipe that blocks the open until a writer shows up. [`open_regular`] opens without following a
//! link (`O_NOFOLLOW`; `FILE_FLAG_OPEN_REPARSE_POINT` on Windows) and without waiting
//! (`O_NONBLOCK`), and reads the type from the opened handle, so nothing can change between the
//! check and the read. Only the constants of `libc` and `windows-sys` are used, no `unsafe`.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// Opens `path` for reading and returns it only if the opened handle is a regular file.
///
/// # Errors
/// The error of the open (a link ends in an error on Unix), or `InvalidInput` if the handle is
/// not a regular file (a link or folder on Windows, a device, a pipe).
pub fn open_regular(path: &Path) -> io::Result<File> {
    let file = options().open(path)?;
    if file.metadata()?.is_file() {
        Ok(file)
    } else {
        Err(io::ErrorKind::InvalidInput.into())
    }
}

/// The options of the open: read only, plus what the system offers against a link.
fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true);
    guard(&mut options);
    options
}

#[cfg(unix)]
fn guard(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
}

#[cfg(windows)]
fn guard(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;

    options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
}

// Other systems have no protection against a link; the type check on the handle remains.
#[cfg(not(any(unix, windows)))]
fn guard(_: &mut OpenOptions) {}

#[cfg(all(test, unix))]
mod tests {
    use super::open_regular;

    #[test]
    fn a_regular_file_opens_and_a_link_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::write(&real, b"x").unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(open_regular(&real).is_ok());
        assert!(open_regular(&link).is_err());
        assert!(open_regular(dir.path()).is_err());
    }
}
