//! Content hashes for incremental scans.

use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::Path;

use codeskimr_domain::ContentHash;

use crate::consts::HASH_BUFFER_BYTES;

/// Opens `path` for reading and keeps the handle only if it is a regular file.
///
/// A link is not followed (`O_NOFOLLOW`; `FILE_FLAG_OPEN_REPARSE_POINT` on Windows) and opening
/// never waits for a writer (`O_NONBLOCK`), so a named pipe or a link that a build tool put in
/// place of a file cannot hang the scan or lead it elsewhere. The type is read from the opened
/// handle, so nothing can change between the check and the read.
///
/// # Errors
/// The error of the open, or `InvalidInput` if the handle is not a regular file.
pub fn open_regular(path: &Path) -> io::Result<File> {
    let file = guarded().open(path)?;
    if file.metadata()?.is_file() {
        Ok(file)
    } else {
        Err(io::ErrorKind::InvalidInput.into())
    }
}

#[cfg(unix)]
fn guarded() -> OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut open = OpenOptions::new();
    open.read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    open
}

#[cfg(windows)]
fn guarded() -> OpenOptions {
    use std::os::windows::fs::OpenOptionsExt;

    let mut open = OpenOptions::new();
    open.read(true)
        .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    open
}

// Other systems have no protection against a link; the type check on the handle remains.
#[cfg(not(any(unix, windows)))]
fn guarded() -> OpenOptions {
    let mut open = OpenOptions::new();
    open.read(true);
    open
}

/// Hashes the file at `path` with BLAKE3 and returns its size in bytes with the hash.
///
/// # Errors
/// Any I/O error while reading, and `InvalidInput` for anything but a regular file.
pub fn hash_file(path: &Path) -> io::Result<(u64, ContentHash)> {
    hash_reader(open_regular(path)?)
}

/// Hashes everything `reader` yields.
///
/// # Errors
/// Any I/O error while reading.
pub fn hash_reader(mut reader: impl Read) -> io::Result<(u64, ContentHash)> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![u8::MIN; HASH_BUFFER_BYTES];
    let mut size = u64::MIN;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        size += read as u64;
        hasher.update(buffer.get(..read).unwrap_or_default());
    }
    let hash = ContentHash::new(hasher.finalize().to_hex().to_string())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok((size, hash))
}
