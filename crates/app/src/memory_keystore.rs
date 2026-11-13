//! An in-memory keystore for the tests of the composition root.

use std::collections::HashMap;
use std::sync::Mutex;

use codeskimr_application::ports::AuthError;
use codeskimr_secrets::Keystore;
use zeroize::Zeroizing;

/// Keeps secrets in a map; nothing touches the keystore of the machine.
#[derive(Default)]
pub struct Memory(Mutex<HashMap<String, Vec<u8>>>);

impl Keystore for Memory {
    fn read(&self, name: &str) -> Result<Zeroizing<Vec<u8>>, AuthError> {
        self.0
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .map(Zeroizing::new)
            .ok_or(AuthError::NotFound)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), AuthError> {
        self.0
            .lock()
            .unwrap()
            .insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn remove(&self, name: &str) -> Result<(), AuthError> {
        self.0.lock().unwrap().remove(name);
        Ok(())
    }
}

/// A credential store over a fresh in-memory keystore, in the type of the composition root.
pub fn secrets() -> std::sync::Arc<crate::shared_keystore::Secrets> {
    use codeskimr_secrets::KeystoreCredentials;

    use crate::shared_keystore::SharedKeystore;

    std::sync::Arc::new(KeystoreCredentials::new(SharedKeystore::new(
        Memory::default(),
    )))
}
