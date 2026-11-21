//! Logging in to a remote with a stored credential (blueprint section 12, adapter: `codeskimr_secrets`).
//!
//! A secret is only ever lent: the store reads it into locked memory, hands a reference to a closure and wipes it
//! again, so it cannot be kept, cloned or printed by accident (loan pattern, blueprint 12.3).

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::CredentialId;

pub use crate::consts::FIELD_SEPARATOR;
use crate::consts::SEPARATOR_LEN;
use crate::error::CodedError;

/// Bytes that the operating system may not write to the swap file and that are wiped when the
/// value is dropped. The adapter `codeskimr_secrets` implements it (`LockedBytes`); the application
/// layer holds no `unsafe` (R-08) and only borrows the bytes.
pub trait LockedSecret: Send + Sync {
    /// The bytes of the buffer.
    fn bytes(&self) -> &[u8];
}

/// A user name and a secret (password or token) for one login.
///
/// Both texts lie in one locked buffer, as the stored form `user`, a zero byte, `secret` (ADR 0077),
/// so neither is ever copied into ordinary memory by this type. It is neither `Clone`, `Debug` nor
/// `Serialize`; the buffer is wiped and unlocked when the value is dropped. The only ways to read
/// the content are the explicit `expose_user()` and `expose_secret()`, which CI lists for review.
pub struct Credential {
    packed: Box<dyn LockedSecret>,
    user_len: usize,
}

impl Credential {
    /// Wraps a locked buffer that holds the user name, a zero byte and the secret.
    ///
    /// # Errors
    /// `SEC-202` if the separator is missing or one of the two parts is not UTF-8 text.
    pub fn from_packed(packed: Box<dyn LockedSecret>) -> Result<Self, AuthError> {
        let bytes = packed.bytes();
        let user_len = bytes
            .iter()
            .position(|byte| *byte == FIELD_SEPARATOR)
            .ok_or(AuthError::Damaged)?;
        let (user, rest) = bytes.split_at(user_len);
        let secret = rest.get(SEPARATOR_LEN..).unwrap_or_default();
        if std::str::from_utf8(user).is_err() || std::str::from_utf8(secret).is_err() {
            return Err(AuthError::Damaged);
        }
        Ok(Self { packed, user_len })
    }

    /// The stored form for the keystore: user name, a zero byte, secret.
    #[must_use]
    pub fn packed(&self) -> &[u8] {
        self.packed.bytes()
    }

    /// The user name. The text stays in the locked buffer.
    #[must_use]
    pub fn expose_user(&self) -> &str {
        self.part(..self.user_len)
    }

    /// The password or token. The text stays in the locked buffer.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.part(self.user_len.saturating_add(SEPARATOR_LEN)..)
    }

    /// One part of the buffer; both parts were checked to be UTF-8 by `from_packed`.
    fn part(&self, range: impl std::slice::SliceIndex<[u8], Output = [u8]>) -> &str {
        self.packed
            .bytes()
            .get(range)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or_default()
    }
}

/// Why a credential could not be used. No variant carries a secret.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The credential does not exist in the store.
    #[error("{}", ErrorCode::CredentialNotFound.description())]
    NotFound,
    /// The keystore entry exists but cannot be read as a credential.
    #[error("{}", ErrorCode::CredentialDamaged.description())]
    Damaged,
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
            Self::Damaged => ErrorCode::CredentialDamaged,
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

    /// Whether the keystore holds a credential under `id`. Unlike [`Self::with_credential`] it
    /// never decodes the entry and never lends it, so a caller that only needs the fact (the list
    /// of the logins) does not touch the secret.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn exists(&self, id: &CredentialId) -> Result<bool, AuthError>;

    /// Saves `credential` under `id` and replaces an existing one.
    ///
    /// # Errors
    /// `SEC-401` if the credential cannot be stored in the form of the keystore (a user name with
    /// a zero byte), `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn store(&self, id: &CredentialId, credential: &Credential) -> Result<(), AuthError>;

    /// Removes the credential; a missing one is not an error.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn delete(&self, id: &CredentialId) -> Result<(), AuthError>;
}
