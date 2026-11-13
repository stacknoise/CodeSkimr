//! Logging in to a remote with a stored credential (blueprint section 12, adapter: `codeskimr_secrets`).
//!
//! A secret is only ever lent: the store reads it, hands a reference to a closure and wipes it
//! again, so it cannot be kept, cloned or printed by accident (loan pattern, blueprint 12.3).

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::CredentialId;
use secrecy::SecretString;

use crate::error::CodedError;

/// A user name and a secret (password or token) for one login.
///
/// It is neither `Clone`, `Debug` nor `Serialize`, and the memory of both texts is wiped when the
/// value is dropped. The only way to read the content is the explicit `expose_secret()` of
/// `secrecy`, which CI lists for review.
pub struct Credential {
    user: SecretString,
    secret: SecretString,
}

impl Credential {
    /// Combines a user name and a secret.
    #[must_use]
    pub fn new(user: SecretString, secret: SecretString) -> Self {
        Self { user, secret }
    }

    /// The user name.
    #[must_use]
    pub fn user(&self) -> &SecretString {
        &self.user
    }

    /// The password or token.
    #[must_use]
    pub fn secret(&self) -> &SecretString {
        &self.secret
    }
}

/// Why a credential could not be used. No variant carries a secret.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The credential does not exist in the store.
    #[error("{}", ErrorCode::CredentialNotFound.description())]
    NotFound,
    /// The platform rejected the credential.
    #[error("{}", ErrorCode::CredentialRejected.description())]
    Rejected,
    /// The operating system denied access to its keystore.
    #[error("{}", ErrorCode::KeystoreAccessDenied.description())]
    AccessDenied,
    /// The keystore service is not available.
    #[error("{}", ErrorCode::KeystoreUnavailable.description())]
    Unavailable,
    /// Locking the memory of a secret failed.
    #[error("{}", ErrorCode::MemoryLockFailed.description())]
    MemoryLockFailed,
}

impl Coded for AuthError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::NotFound => ErrorCode::CredentialNotFound,
            Self::Rejected => ErrorCode::CredentialRejected,
            Self::AccessDenied => ErrorCode::KeystoreAccessDenied,
            Self::Unavailable => ErrorCode::KeystoreUnavailable,
            Self::MemoryLockFailed => ErrorCode::MemoryLockFailed,
        }
    }
}

impl From<AuthError> for CodedError {
    fn from(error: AuthError) -> Self {
        Self::caused_by(error)
    }
}

/// Keeps credentials and lends them for one login attempt.
pub trait CredentialStore: Send + Sync {
    /// Reads the credential, lends it to `action` and wipes it afterwards. Every call reads again;
    /// nothing is cached between attempts.
    ///
    /// # Errors
    /// `SEC-201` if the credential does not exist, `SEC-402` or `SEC-601` if the keystore cannot
    /// be used, otherwise what `action` returns.
    fn with_credential<T>(
        &self,
        id: &CredentialId,
        action: impl FnOnce(&Credential) -> Result<T, AuthError>,
    ) -> Result<T, AuthError>;

    /// Saves `credential` under `id` and replaces an existing one.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn store(&self, id: &CredentialId, credential: &Credential) -> Result<(), AuthError>;

    /// Removes the credential; a missing one is not an error.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn delete(&self, id: &CredentialId) -> Result<(), AuthError>;
}
