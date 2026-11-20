//! The stored form of a credential: user name, a separator byte, secret.
//!
//! The credential keeps exactly this form in one locked buffer (ADR 0077), so storing it needs no
//! further copy and reading it back needs no copy outside locked memory.

use codeskimr_application::ports::{AuthError, Credential, FIELD_SEPARATOR};
use zeroize::Zeroizing;

use crate::memory::LockedBytes;

/// Packs a user name and a secret into a credential whose buffer is locked in memory and wiped
/// when dropped. The caller's texts are not touched; the temporary packing buffer is wiped.
///
/// A user name with the separator byte would make the entry ambiguous, so it is refused.
///
/// # Errors
/// `SEC-401` for a user name with the separator, `SEC-901` if the memory cannot be locked.
pub fn credential(user: &str, secret: &str) -> Result<Credential, AuthError> {
    if user.as_bytes().contains(&FIELD_SEPARATOR) {
        return Err(AuthError::Rejected);
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(user.len() + 1 + secret.len()));
    bytes.extend_from_slice(user.as_bytes());
    bytes.push(FIELD_SEPARATOR);
    bytes.extend_from_slice(secret.as_bytes());
    Credential::from_packed(Box::new(LockedBytes::take_from(&mut bytes)?))
}

/// Unpacks a stored entry that already lies in locked memory. A damaged entry is `SEC-202`, not a
/// missing one: the entry exists and has to be replaced.
pub fn decode(packed: LockedBytes) -> Result<Credential, AuthError> {
    Credential::from_packed(Box::new(packed))
}

#[cfg(test)]
mod tests {
    use super::{LockedBytes, credential, decode};

    #[test]
    fn a_credential_survives_a_round_trip() {
        let built = credential("anton", "pa ss\nword").unwrap();
        let back = decode(LockedBytes::copy_of(built.packed()).unwrap()).unwrap();
        assert_eq!(back.expose_user(), "anton");
        assert_eq!(back.expose_secret(), "pa ss\nword");
    }

    #[test]
    fn a_user_with_the_separator_is_refused_and_garbage_is_damaged() {
        assert!(credential("a\0b", "x").is_err());
        assert!(decode(LockedBytes::copy_of(b"no separator").unwrap()).is_err());
        assert!(decode(LockedBytes::copy_of(&[0xff, 0, 0xfe]).unwrap()).is_err());
    }
}
