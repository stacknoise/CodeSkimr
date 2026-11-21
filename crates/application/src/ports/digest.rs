//! The hash function of content hashes (adapter: `codeskimr_parsing`).

use codeskimr_domain::ContentHash;

use crate::error::CodedError;

/// Hashes text with the function that also hashes the symbols of a scan, so that every symbol hash
/// of the product is comparable.
pub trait ContentDigest: Send + Sync {
    /// The hash over `parts`, taken in order. Where one part ends and the next begins is part of
    /// the hash, so `["ab", "c"]` and `["a", "bc"]` differ.
    ///
    /// # Errors
    /// A coded failure if the digest is not a valid hash, which is a bug of the adapter.
    fn digest(&self, parts: &[&str]) -> Result<ContentHash, CodedError>;
}
