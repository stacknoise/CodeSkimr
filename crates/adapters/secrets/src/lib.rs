//! Credential store and secret handling. No other adapter may be used here.
//!
//! - [`KeystoreCredentials`] implements `CredentialStore` (blueprint section 12): a credential is
//!   read from a [`Keystore`] right before a login, lent to a closure and wiped afterwards.
//! - [`OsKeystore`] is the keystore of the operating system (Windows Credential Manager, macOS
//!   Keychain, Linux Secret Service); [`OsCredentialStore`] combines both.
//!
//! - [`DeviceFlow`] signs in to GitHub or GitLab with the OAuth device flow and returns a token as a
//!   `Credential`; [`Provider`] knows the endpoints.
//!
//! Nothing is cached and nothing is written anywhere else than the keystore.

mod consts;
mod device_flow;
mod encoding;
mod keystore;
mod provider;
mod store;

pub use device_flow::{DeviceFlow, DeviceFlowError, Prompt};
pub use keystore::{Keystore, OsKeystore};
pub use provider::Provider;
pub use store::{KeystoreCredentials, OsCredentialStore};
