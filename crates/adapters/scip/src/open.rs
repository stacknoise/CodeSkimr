//! Opening the index file of an indexer without following a link.
//!
//! The folder belongs to the runner and the indexer has ended, so a link that replaces the file
//! between a check and the open is hardly possible. The open itself refuses the link anyway
//! (`O_NOFOLLOW` on Unix, `FILE_FLAG_OPEN_REPARSE_POINT` on Windows), and the type is read from
//! the handle that was opened, so there is nothing left to race (issue #229).

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// Opens `path` for reading and returns it only if the opened handle is a regular file.
///
/// # Errors
/// The error of the open (a link ends in an error on Unix), or `InvalidInput` if the handle is
/// not a regular file (a link or folder on Windows, a device, a pipe).
pub(crate) fn open_regular(path: &Path) -> io::Result<File> {
    let file = options().open(path)?;
    if file.metadata()?.is_file() {
        Ok(file)
    } else {
        Err(io::ErrorKind::InvalidInput.into())
    }
}

#[cfg(unix)]
fn options() -> OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    // `O_NONBLOCK`: opening a named pipe for reading would wait for a writer, and the indexer
    // could hang the scan by putting a pipe in place of the index. A regular file is not affected.
    options
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    options
}

#[cfg(windows)]
fn options() -> OpenOptions {
    use std::os::windows::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    options
}

// On other systems there is no protection against a link; the type check on the handle remains.
#[cfg(not(any(unix, windows)))]
fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true);
    options
}
