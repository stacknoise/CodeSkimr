//! Credential store and secret handling. No other adapter may be used here.
//!
//! - [`KeystoreCredentials`] implements `CredentialStore` (blueprint section 12): a credential is
//!   read from a [`Keystore`] right before a login, lent to a closure and wiped afterwards.
//! - [`OsKeystore`] is the keystore of the operating system (Windows Credential Manager, macOS
//!   Keychain, Linux Secret Service); [`OsCredentialStore`] combines both.
//!
//! Nothing is cached and nothing is written anywhere else than the keystore.

mod consts;
mod encoding;
mod keystore;
mod store;

pub use keystore::{Keystore, OsKeystore};
pub use store::{KeystoreCredentials, OsCredentialStore};
