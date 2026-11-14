//! The token of this installation (blueprint S-04, 12.2): created at the first start, kept in the
//! credential store, and handed to the server only as a digest.
//!
//! The token is the proof that a request to the local server comes from something that can read
//! the user's keystore, not from a web page in a browser. It is never written to a file, a log or
//! the database.

use std::fs::{self, File};
use std::path::Path;
use std::time::Instant;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AuthError, Credential, CredentialStore};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::CredentialId;
use codeskimr_server::{InstallationToken, TOKEN_PREFIX};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

use crate::consts::{
    HEX_DIGIT_BITS, HEX_LOW_MASK, HEX_RADIX, TOKEN_CREDENTIAL_ID, TOKEN_CREDENTIAL_USER,
    TOKEN_HEX_LEN, TOKEN_LOCK_POLL, TOKEN_LOCK_WAIT, TOKEN_RANDOM_BYTES,
};

/// The digest of the installation token; the token is created and stored if there is none yet.
///
/// # Errors
/// - `SEC-402` or `SEC-601` if the keystore cannot be used. The token is not replaced then: only
///   a token that is missing is created.
/// - `APP-901` if the system has no source of random numbers.
pub fn load_or_create(store: &impl CredentialStore) -> Result<InstallationToken, CodedError> {
    let id = CredentialId::new(TOKEN_CREDENTIAL_ID)
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    match digest_of(store, &id) {
        Err(AuthError::NotFound) => {
            store.store(&id, &fresh_credential()?)?;
            Ok(digest_of(store, &id)?)
        }
        other => Ok(other?),
    }
}

/// Like [`load_or_create`], but two programs that start together cannot both create a token: the
/// check and the creation happen while this process holds an exclusive lock on `lock_file`
/// (`SEC-S-06`, issue #269). The keystore has no compare-and-set, so the lock is the guard. The
/// file stays empty and holds no secret; the lock is released when the call returns.
///
/// # Errors
/// As [`load_or_create`]; `APP-901` if the lock file cannot be created, or if another program
/// holds the lock for longer than `TOKEN_LOCK_WAIT`.
pub fn load_or_create_locked(
    store: &impl CredentialStore,
    lock_file: &Path,
) -> Result<InstallationToken, CodedError> {
    let internal = |_| CodedError::new(ErrorCode::UnexpectedInternalError);
    if let Some(folder) = lock_file.parent() {
        fs::create_dir_all(folder).map_err(internal)?;
    }
    let file = File::options()
        .create(true)
        .append(true)
        .open(lock_file)
        .map_err(internal)?;
    acquire(&file)?;
    // The lock goes with the file handle, also when the call fails.
    load_or_create(store)
}

/// Waits for the exclusive lock of `file`, but not forever: a program that hangs while it holds
/// the lock must not stop every later start.
fn acquire(file: &File) -> Result<(), CodedError> {
    let deadline = Instant::now() + TOKEN_LOCK_WAIT;
    loop {
        match fs4::FileExt::try_lock(file) {
            Ok(()) => return Ok(()),
            Err(fs4::TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(TOKEN_LOCK_POLL);
            }
            Err(_) => return Err(CodedError::new(ErrorCode::UnexpectedInternalError)),
        }
    }
}

/// The only place that reads the token text: it never leaves the loan of the store.
fn digest_of(
    store: &impl CredentialStore,
    id: &CredentialId,
) -> Result<InstallationToken, AuthError> {
    store.with_credential(id, |credential| {
        Ok(InstallationToken::from_token(
            credential.secret().expose_secret(),
        ))
    })
}

fn fresh_credential() -> Result<Credential, CodedError> {
    let mut bytes = Zeroizing::new([u8::default(); TOKEN_RANDOM_BYTES]);
    getrandom::fill(bytes.as_mut())
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    let mut text = String::with_capacity(TOKEN_PREFIX.len() + TOKEN_HEX_LEN);
    text.push_str(TOKEN_PREFIX);
    for byte in bytes.iter() {
        text.extend(hex_digit(byte >> HEX_DIGIT_BITS));
        text.extend(hex_digit(byte & HEX_LOW_MASK));
    }
    Ok(Credential::new(
        SecretString::from(TOKEN_CREDENTIAL_USER),
        SecretString::from(text),
    ))
}

fn hex_digit(value: u8) -> Option<char> {
    char::from_digit(u32::from(value), HEX_RADIX)
}

/// The token text of a test keystore, for the tests that act as a client of the server.
#[cfg(test)]
pub(crate) fn stored_token(store: &impl CredentialStore) -> String {
    let id = CredentialId::new(TOKEN_CREDENTIAL_ID).unwrap();
    store
        .with_credential(&id, |credential| {
            Ok(credential.secret().expose_secret().to_owned())
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use codeskimr_secrets::{Keystore, KeystoreCredentials};

    use crate::memory_keystore::Memory;

    use super::*;

    struct Broken;

    impl Keystore for Broken {
        fn read(&self, _: &str) -> Result<Zeroizing<Vec<u8>>, AuthError> {
            Err(AuthError::Unavailable)
        }

        fn write(&self, _: &str, _: &[u8]) -> Result<(), AuthError> {
            panic!("a broken keystore must not be written to")
        }

        fn remove(&self, _: &str) -> Result<(), AuthError> {
            Ok(())
        }
    }

    #[test]
    fn the_first_start_creates_a_long_random_token_with_the_prefix() {
        let store = KeystoreCredentials::new(Memory::default());
        let digest = load_or_create(&store).unwrap();
        let token = stored_token(&store);
        assert!(token.starts_with(TOKEN_PREFIX));
        assert_eq!(token.len(), TOKEN_PREFIX.len() + 2 * TOKEN_RANDOM_BYTES);
        assert!(
            token[TOKEN_PREFIX.len()..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        );
        assert!(digest.accepts(&token));
        assert!(!digest.accepts(&format!("{token}0")));
    }

    #[test]
    fn the_next_start_keeps_the_token() {
        let store = KeystoreCredentials::new(Memory::default());
        load_or_create(&store).unwrap();
        let first = stored_token(&store);
        let again = load_or_create(&store).unwrap();
        assert_eq!(stored_token(&store), first);
        assert!(again.accepts(&first));
    }

    #[test]
    fn programs_that_start_together_end_with_one_token() {
        let folder = tempfile::tempdir().unwrap();
        let lock = folder.path().join("token.lock");
        let store = std::sync::Arc::new(KeystoreCredentials::new(Memory::default()));
        let digests: Vec<_> = (0..8)
            .map(|_| {
                let (store, lock) = (std::sync::Arc::clone(&store), lock.clone());
                std::thread::spawn(move || load_or_create_locked(&*store, &lock).unwrap())
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        let token = stored_token(&*store);
        assert!(digests.iter().all(|digest| digest.accepts(&token)));
    }

    #[test]
    fn a_lock_that_stays_taken_ends_the_start_with_an_error() {
        let folder = tempfile::tempdir().unwrap();
        let lock = folder.path().join("token.lock");
        let holder = File::options()
            .create(true)
            .append(true)
            .open(&lock)
            .unwrap();
        fs4::FileExt::try_lock(&holder).unwrap();
        let store = KeystoreCredentials::new(Memory::default());
        let error = load_or_create_locked(&store, &lock).unwrap_err();
        assert_eq!(codeskimr_codes::Coded::code(&error).id(), "APP-901");
    }

    #[test]
    fn two_installations_get_different_tokens() {
        let one = KeystoreCredentials::new(Memory::default());
        let two = KeystoreCredentials::new(Memory::default());
        load_or_create(&one).unwrap();
        load_or_create(&two).unwrap();
        assert_ne!(stored_token(&one), stored_token(&two));
    }

    #[test]
    fn an_unusable_keystore_is_reported_and_never_overwritten() {
        let store = KeystoreCredentials::new(Broken);
        let error = load_or_create(&store).unwrap_err();
        assert_eq!(codeskimr_codes::Coded::code(&error).id(), "SEC-601");
    }
}
