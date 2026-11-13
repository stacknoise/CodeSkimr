//! The stored form of a credential: user name, a separator byte, secret.

use codeskimr_application::ports::{AuthError, Credential};
use secrecy::{ExposeSecret, SecretString};
use zeroize::{Zeroize, Zeroizing};

use crate::consts::FIELD_SEPARATOR;

/// Packs a credential for the keystore; the buffer is wiped when dropped.
///
/// A user name with the separator byte would make the entry ambiguous, so it is refused.
pub fn encode(credential: &Credential) -> Result<Zeroizing<Vec<u8>>, AuthError> {
    let user = credential.user().expose_secret().as_bytes();
    if user.contains(&FIELD_SEPARATOR) {
        return Err(AuthError::Rejected);
    }
    let secret = credential.secret().expose_secret().as_bytes();
    let mut bytes = Zeroizing::new(Vec::with_capacity(user.len() + 1 + secret.len()));
    bytes.extend_from_slice(user);
    bytes.push(FIELD_SEPARATOR);
    bytes.extend_from_slice(secret);
    Ok(bytes)
}

/// Unpacks a stored entry. A damaged entry counts as missing: it cannot be used for a login.
pub fn decode(bytes: &[u8]) -> Result<Credential, AuthError> {
    let split = bytes
        .iter()
        .position(|byte| *byte == FIELD_SEPARATOR)
        .ok_or(AuthError::NotFound)?;
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
            AuthError::NotFound
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
        let packed = encode(&credential("anton", "pa ss\nword")).unwrap_or_default();
        let back = decode(&packed);
        assert!(back.is_ok_and(|c| {
            c.user().expose_secret() == "anton" && c.secret().expose_secret() == "pa ss\nword"
        }));
    }

    #[test]
    fn a_user_with_the_separator_is_refused_and_garbage_is_missing() {
        assert!(encode(&credential("a\0b", "x")).is_err());
        assert!(decode(b"no separator").is_err());
        assert!(decode(&[0xff, 0, 0xfe]).is_err());
    }
}
