//! The credential store against the real operating system keystore (issue #197).
//!
//! These tests need a keystore service: a Secret Service on Linux (the CI job starts
//! `gnome-keyring` inside `dbus-run-session`), the Keychain on macOS, the Credential Manager on
//! Windows. They run only when `CODESKIMR_REAL_KEYSTORE` is set, so a developer without a
//! keystore service is not blocked; the CI job sets it, and then a missing service is a failure,
//! not a skip.
#![allow(missing_docs)]

use codeskimr_application::ports::{Credential, CredentialStore};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::CredentialId;
use codeskimr_secrets::{Keystore, OsCredentialStore, OsKeystore};
use secrecy::{ExposeSecret, SecretString};

const SWITCH: &str = "CODESKIMR_REAL_KEYSTORE";
const CANARY: &str = "CANARY-os-keystore-7d41c0";

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn enabled() -> bool {
    std::env::var_os(SWITCH).is_some()
}

/// An id of its own per process, so two runs on one machine never see each other's entry.
fn own_id(tag: &str) -> TestResult<CredentialId> {
    Ok(CredentialId::new(format!(
        "codeskimr-test-{tag}-{}",
        std::process::id()
    ))?)
}

fn credential(secret: &str) -> Credential {
    Credential::new(SecretString::from("anton"), SecretString::from(secret))
}

#[test]
fn a_credential_survives_the_round_trip_through_the_real_keystore() -> TestResult {
    if !enabled() {
        return Ok(());
    }
    let store = OsCredentialStore::new(OsKeystore);
    let id = own_id("round")?;
    store.store(&id, &credential(CANARY))?;
    let seen = store.with_credential(&id, |c| {
        Ok((
            c.user().expose_secret().to_owned(),
            c.secret().expose_secret().to_owned(),
        ))
    });
    store.delete(&id)?;
    assert_eq!(seen?, ("anton".to_owned(), CANARY.to_owned()));
    Ok(())
}

#[test]
fn a_stored_credential_can_be_replaced_and_a_deleted_one_is_gone() -> TestResult {
    if !enabled() {
        return Ok(());
    }
    let store = OsCredentialStore::new(OsKeystore);
    let id = own_id("replace")?;
    store.store(&id, &credential("first"))?;
    store.store(&id, &credential(CANARY))?;
    let second = store.with_credential(&id, |c| Ok(c.secret().expose_secret().to_owned()))?;
    store.delete(&id)?;
    store.delete(&id)?;
    let gone = store.with_credential(&id, |_| Ok(())).unwrap_err();
    assert_eq!(second, CANARY);
    assert_eq!(gone.code(), ErrorCode::CredentialNotFound);
    Ok(())
}

#[test]
fn no_error_of_the_real_keystore_carries_the_secret() -> TestResult {
    if !enabled() {
        return Ok(());
    }
    let store = OsCredentialStore::new(OsKeystore);
    let id = own_id("errors")?;
    store.store(&id, &credential(CANARY))?;
    let rejected = store
        .with_credential(&id, |_| {
            Err::<(), _>(codeskimr_application::ports::AuthError::Rejected)
        })
        .unwrap_err();
    store.delete(&id)?;
    let missing = store.with_credential(&id, |_| Ok(())).unwrap_err();
    for error in [rejected, missing] {
        let texts = [error.to_string(), format!("{error:?}")];
        assert!(texts.iter().all(|text| !text.contains(CANARY)));
    }
    Ok(())
}

#[test]
fn the_raw_entry_is_bytes_that_the_keystore_returns_unchanged() -> TestResult {
    if !enabled() {
        return Ok(());
    }
    let id = own_id("raw")?;
    let name = id.as_str();
    OsKeystore.write(name, CANARY.as_bytes())?;
    let read = OsKeystore.read(name);
    OsKeystore.remove(name)?;
    assert_eq!(read?.as_slice(), CANARY.as_bytes());
    Ok(())
}
