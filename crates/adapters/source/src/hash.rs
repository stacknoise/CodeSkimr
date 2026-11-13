//! Content hashes for incremental scans.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use codeskimr_domain::ContentHash;

use crate::consts::HASH_BUFFER_BYTES;

/// Hashes the file at `path` with BLAKE3 and returns its size in bytes with the hash.
///
/// # Errors
/// Any I/O error while reading.
pub fn hash_file(path: &Path) -> io::Result<(u64, ContentHash)> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![u8::MIN; HASH_BUFFER_BYTES];
    let mut size = u64::MIN;
    loop {
        let read = file.read(&mut buffer)?;
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
