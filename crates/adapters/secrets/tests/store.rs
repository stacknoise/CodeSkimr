//! The credential store on top of a keystore: loan, retry reads, errors without secrets.
#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use codeskimr_application::ports::{AuthError, Credential, CredentialStore};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::CredentialId;
use codeskimr_secrets::{Keystore, KeystoreCredentials};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

const CANARY: &str = "CANARY-3f9c61aa";

#[derive(Default)]
struct MemoryKeystore {
    entries: Mutex<HashMap<String, Vec<u8>>>,
    reads: AtomicUsize,
}

impl Keystore for MemoryKeystore {
    fn read(&self, name: &str) -> Result<Zeroizing<Vec<u8>>, AuthError> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let entries = self.entries.lock().map_err(|_| AuthError::Unavailable)?;
        entries
            .get(name)
            .map(|bytes| Zeroizing::new(bytes.clone()))
            .ok_or(AuthError::NotFound)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), AuthError> {
        let mut entries = self.entries.lock().map_err(|_| AuthError::Unavailable)?;
        entries.insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn remove(&self, name: &str) -> Result<(), AuthError> {
        let mut entries = self.entries.lock().map_err(|_| AuthError::Unavailable)?;
        entries.remove(name);
        Ok(())
    }
}

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn credential(user: &str, secret: &str) -> Credential {
    Credential::new(SecretString::from(user), SecretString::from(secret))
}

fn store_with(id: &CredentialId) -> TestResult<KeystoreCredentials<MemoryKeystore>> {
    let store = KeystoreCredentials::new(MemoryKeystore::default());
    store.store(id, &credential("anton", CANARY))?;
    Ok(store)
}

#[test]
fn a_stored_credential_is_lent_to_the_closure() -> TestResult {
    let id = CredentialId::new("work")?;
    let store = store_with(&id)?;
    let seen = store.with_credential(&id, |c| {
        Ok((
            c.user().expose_secret().to_owned(),
            c.secret().expose_secret().to_owned(),
        ))
    })?;
    assert_eq!(seen, ("anton".to_owned(), CANARY.to_owned()));
    Ok(())
}

#[test]
fn every_attempt_reads_again() -> TestResult {
    let id = CredentialId::new("work")?;
    let store = store_with(&id)?;
    let before = store.keystore().reads.load(Ordering::Relaxed);
    for _ in 0..3 {
        store.with_credential(&id, |_| Ok(()))?;
    }
    assert_eq!(store.keystore().reads.load(Ordering::Relaxed) - before, 3);
    Ok(())
}

#[test]
fn a_missing_credential_has_its_code_and_a_deleted_one_is_gone() -> TestResult {
    let id = CredentialId::new("work")?;
    let store = store_with(&id)?;
    store.delete(&id)?;
    store.delete(&id)?;
    let error = store.with_credential(&id, |_| Ok(())).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CredentialNotFound);
    Ok(())
}

#[test]
fn a_failure_of_the_action_passes_through_without_the_secret() -> TestResult {
    let id = CredentialId::new("work")?;
    let store = store_with(&id)?;
    let error = store
        .with_credential(&id, |_| Err::<(), _>(AuthError::Rejected))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::CredentialRejected);
    let texts = [error.to_string(), format!("{error:?}")];
    assert!(texts.iter().all(|text| !text.contains(CANARY)));
    Ok(())
}

#[test]
fn a_damaged_entry_is_treated_as_missing() -> TestResult {
    let id = CredentialId::new("work")?;
    let store = KeystoreCredentials::new(MemoryKeystore::default());
    store
        .keystore()
        .write("work", b"no separator, only a secret")?;
    let error = store.with_credential(&id, |_| Ok(())).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CredentialNotFound);
    Ok(())
}

#[test]
fn the_os_keystore_answers_with_a_code_and_never_panics() -> TestResult {
    use codeskimr_secrets::OsCredentialStore;
    let store = OsCredentialStore::new(codeskimr_secrets::OsKeystore);
    let id = CredentialId::new("codeskimr-test-entry-that-does-not-exist")?;
    // Without a keystore service (CI, containers) this is `SEC-601`, with one it is `SEC-201`.
    let error = store.with_credential(&id, |_| Ok(())).unwrap_err();
    assert!(matches!(
        error.code(),
        ErrorCode::CredentialNotFound
            | ErrorCode::KeystoreUnavailable
            | ErrorCode::KeystoreAccessDenied
    ));
    Ok(())
}
