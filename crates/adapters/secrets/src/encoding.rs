//! The stored form of a credential: user name, a separator byte, secret.

use codeskimr_application::ports::{AuthError, Credential};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::consts::FIELD_SEPARATOR;
use crate::memory::LockedBytes;

/// Packs a credential for the keystore; the buffer is locked in memory and wiped when dropped.
///
/// A user name with the separator byte would make the entry ambiguous, so it is refused.
pub fn encode(credential: &Credential) -> Result<LockedBytes, AuthError> {
    let user = credential.user().expose_secret().as_bytes();
    if user.contains(&FIELD_SEPARATOR) {
        return Err(AuthError::Rejected);
    }
    let secret = credential.secret().expose_secret().as_bytes();
    let mut bytes = Vec::with_capacity(user.len() + 1 + secret.len());
    bytes.extend_from_slice(user);
    bytes.push(FIELD_SEPARATOR);
    bytes.extend_from_slice(secret);
    LockedBytes::from_vec(bytes)
}

/// Unpacks a stored entry. A damaged entry is `SEC-202`, not a missing one: the entry exists and has to be replaced.
pub fn decode(bytes: &[u8]) -> Result<Credential, AuthError> {
    let split = bytes
        .iter()
        .position(|byte| *byte == FIELD_SEPARATOR)
        .ok_or(AuthError::Damaged)?;
    let (user, rest) = bytes.split_at(split);
    let secret = rest.get(1..).unwrap_or_default();
    Ok(Credential::new(text(user)?, text(secret)?))
}

fn text(bytes: &[u8]) -> Result<SecretString, AuthError> {
    String::from_utf8(bytes.to_vec())
        .map(SecretString::from)
        .map_err(|error| {
            // The failed copy still holds secret bytes.
            error.into_bytes().zeroize();
            AuthError::Damaged
        })
}

#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, SecretString};

    use super::{Credential, decode, encode};

    fn credential(user: &str, secret: &str) -> Credential {
        Credential::new(SecretString::from(user), SecretString::from(secret))
    }

    #[test]
    fn a_credential_survives_a_round_trip() {
        let packed = encode(&credential("anton", "pa ss\nword"));
        let back = packed.map(|bytes| decode(&bytes));
        assert!(back.is_ok_and(|c| c.is_ok_and(|c| {
            c.user().expose_secret() == "anton" && c.secret().expose_secret() == "pa ss\nword"
        })));
    }

    #[test]
    fn a_user_with_the_separator_is_refused_and_garbage_is_missing() {
        assert!(encode(&credential("a\0b", "x")).is_err());
        assert!(decode(b"no separator").is_err());
        assert!(decode(&[0xff, 0, 0xfe]).is_err());
    }
}
