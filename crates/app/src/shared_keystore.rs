//! The keystore the scanner and the credential commands share.

use std::sync::Arc;

use codeskimr_application::ports::AuthError;
use codeskimr_secrets::{Keystore, KeystoreCredentials};
use zeroize::Zeroizing;

/// A keystore that can be shared by everything that needs secrets: the operating system's in the
/// program, a map in memory in the tests of the composition root.
#[derive(Clone)]
pub struct SharedKeystore(Arc<dyn Keystore>);

impl SharedKeystore {
    /// Shares `keystore`.
    #[must_use]
    pub fn new(keystore: impl Keystore + 'static) -> Self {
        Self(Arc::new(keystore))
    }
}

impl Keystore for SharedKeystore {
    fn read(&self, name: &str) -> Result<Zeroizing<Vec<u8>>, AuthError> {
        self.0.read(name)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), AuthError> {
        self.0.write(name, bytes)
    }

    fn remove(&self, name: &str) -> Result<(), AuthError> {
        self.0.remove(name)
    }
}

/// The credential store the scanner and the credential commands share.
pub type Secrets = KeystoreCredentials<SharedKeystore>;
