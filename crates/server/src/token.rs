//! The per-installation token (blueprint S-04): what the server keeps of it, and how it compares.

use std::fmt;

/// The digest of the token of this installation.
///
/// The server never holds the token itself, only its BLAKE3 digest: a heap dump of the server
/// does not give out a token that works. The comparison of two digests takes the same time
/// wherever they differ.
#[derive(Clone, PartialEq, Eq)]
pub struct InstallationToken {
    digest: blake3::Hash,
}

impl InstallationToken {
    /// The digest of `token`.
    #[must_use]
    pub fn from_token(token: &str) -> Self {
        Self {
            digest: blake3::hash(token.as_bytes()),
        }
    }

    /// Whether `presented` is the token this digest belongs to.
    #[must_use]
    pub fn accepts(&self, presented: &str) -> bool {
        self.digest == blake3::hash(presented.as_bytes())
    }
}

impl fmt::Debug for InstallationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(InstallationToken))
            .finish_non_exhaustive()
    }
}
