//! Where the bytes of a credential are kept.

use std::sync::LazyLock;

use codeskimr_application::ports::AuthError;
use keyring_core::{Entry, Error};
use zeroize::Zeroizing;

use crate::consts::KEYSTORE_SERVICE;

/// A place that keeps secret bytes under a name. Implementations must not log or cache them.
pub trait Keystore: Send + Sync {
    /// The bytes stored under `name`; the buffer is wiped when dropped.
    ///
    /// # Errors
    /// `SEC-201` if there is no entry, `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn read(&self, name: &str) -> Result<Zeroizing<Vec<u8>>, AuthError>;

    /// Stores `bytes` under `name`, replacing an existing entry.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), AuthError>;

    /// Removes the entry; a missing entry is not an error.
    ///
    /// # Errors
    /// `SEC-402` or `SEC-601` if the keystore cannot be used.
    fn remove(&self, name: &str) -> Result<(), AuthError>;
}

/// The keystore of the operating system (blueprint 12.2, point 3).
#[derive(Debug, Clone, Copy, Default)]
pub struct OsKeystore;

impl OsKeystore {
    fn entry(name: &str) -> Result<Entry, AuthError> {
        if !*STORE_READY {
            return Err(AuthError::Unavailable);
        }
        Entry::new(KEYSTORE_SERVICE, name).map_err(|error| map_error(&error))
    }
}

impl Keystore for OsKeystore {
    fn read(&self, name: &str) -> Result<Zeroizing<Vec<u8>>, AuthError> {
        Self::entry(name)?
            .get_secret()
            .map(Zeroizing::new)
            .map_err(|error| map_error(&error))
    }

    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), AuthError> {
        Self::entry(name)?
            .set_secret(bytes)
            .map_err(|error| map_error(&error))
    }

    fn remove(&self, name: &str) -> Result<(), AuthError> {
        match Self::entry(name)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(map_error(&error)),
        }
    }
}

/// Whether the platform store was set up; it is done once, on first use.
static STORE_READY: LazyLock<bool> = LazyLock::new(|| install_platform_store().is_ok());

/// Makes the keystore of this platform the default of `keyring-core`.
#[cfg(target_os = "windows")]
fn install_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    Ok(())
}

/// Makes the keystore of this platform the default of `keyring-core`.
#[cfg(target_os = "macos")]
fn install_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new()?);
    Ok(())
}

/// Makes the keystore of this platform (the Secret Service) the default of `keyring-core`.
#[cfg(all(unix, not(target_os = "macos")))]
fn install_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(dbus_secret_service_keyring_store::Store::new()?);
    Ok(())
}

/// Platforms without a keystore adapter cannot keep credentials.
#[cfg(not(any(unix, windows)))]
fn install_platform_store() -> keyring_core::Result<()> {
    Err(Error::NoDefaultStore)
}

/// Sorts a keystore failure into the registered codes. The error itself is dropped unread: some
/// variants carry the bytes of the entry.
fn map_error(error: &Error) -> AuthError {
    match error {
        Error::NoEntry => AuthError::NotFound,
        Error::NoStorageAccess(_) => AuthError::AccessDenied,
        _ => AuthError::Unavailable,
    }
}
