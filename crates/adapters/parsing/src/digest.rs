//! The `ContentDigest` adapter: the hash function of the content hashes, BLAKE3.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ContentDigest;
use codeskimr_domain::ContentHash;

use crate::consts::TOKEN_SEPARATOR;

/// BLAKE3 over the parts of a text, each followed by a separator.
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Digest;

impl ContentDigest for Blake3Digest {
    fn digest(&self, parts: &[&str]) -> Result<ContentHash, CodedError> {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(part.as_bytes());
            hasher.update(&[TOKEN_SEPARATOR]);
        }
        ContentHash::new(hasher.finalize().to_hex().to_string()).map_err(CodedError::caused_by)
    }
}
