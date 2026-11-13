//! The `CredentialStore` port on top of a keystore.

use codeskimr_application::ports::{AuthError, Credential, CredentialStore};
use codeskimr_domain::CredentialId;

use crate::encoding::{decode, encode};
use crate::keystore::{Keystore, OsKeystore};

/// Lends credentials that a [`Keystore`] holds.
#[derive(Debug, Clone)]
pub struct KeystoreCredentials<K> {
    keystore: K,
}

/// The credential store of the operating system.
pub type OsCredentialStore = KeystoreCredentials<OsKeystore>;

impl<K: Keystore> KeystoreCredentials<K> {
    /// Uses `keystore` as the place of the secrets.
    #[must_use]
    pub fn new(keystore: K) -> Self {
        Self { keystore }
    }

    /// The keystore behind the store.
    #[must_use]
    pub fn keystore(&self) -> &K {
        &self.keystore
    }
}

impl<K: Keystore> CredentialStore for KeystoreCredentials<K> {
    fn with_credential<T>(
        &self,
        id: &CredentialId,
        action: impl FnOnce(&Credential) -> Result<T, AuthError>,
    ) -> Result<T, AuthError> {
        // Both the packed bytes and the credential are wiped when they leave this function, on
        // every path including a panic inside `action`.
        let packed = self.keystore.read(id.as_str())?;
        let credential = decode(&packed)?;
        action(&credential)
    }

    fn store(&self, id: &CredentialId, credential: &Credential) -> Result<(), AuthError> {
        let packed = encode(credential)?;
        self.keystore.write(id.as_str(), &packed)
    }

    fn delete(&self, id: &CredentialId) -> Result<(), AuthError> {
        self.keystore.remove(id.as_str())
    }
}
