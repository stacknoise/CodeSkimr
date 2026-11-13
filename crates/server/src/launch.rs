//! The one-time launch code that hands the installation token to the browser (ADR 0028).
//!
//! The code is 128 random bits. The store keeps its BLAKE3 digest, never the code, and the token
//! the code stands for, for as long as the code lives. A heap dump therefore gives out a token
//! for at most the lifetime of one code, and the token is overwritten when the code is used or
//! has expired. Only one code is valid at a time.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

use crate::consts::{
    HEX_DIGIT_BITS, HEX_LOW_MASK, HEX_RADIX, LAUNCH_CODE_BYTES, LAUNCH_CODE_HEX_LEN,
};

struct Pending {
    digest: blake3::Hash,
    token: SecretString,
    expires: Instant,
}

/// The launch code that is valid right now, if any.
pub struct LaunchCodes {
    lifetime: Duration,
    slot: Mutex<Option<Pending>>,
}

impl LaunchCodes {
    /// An empty store whose codes live for `lifetime`.
    #[must_use]
    pub const fn new(lifetime: Duration) -> Self {
        Self {
            lifetime,
            slot: Mutex::new(None),
        }
    }

    /// How long a code lives.
    #[must_use]
    pub const fn lifetime(&self) -> Duration {
        self.lifetime
    }

    /// Creates a code for `token` and invalidates the one before. `None` if the system has no
    /// source of random numbers.
    #[must_use]
    pub fn issue(&self, token: &str) -> Option<SecretString> {
        let mut bytes = Zeroizing::new([u8::default(); LAUNCH_CODE_BYTES]);
        getrandom::fill(bytes.as_mut()).ok()?;
        let mut code = Zeroizing::new(String::with_capacity(LAUNCH_CODE_HEX_LEN));
        for byte in bytes.iter() {
            code.extend(char::from_digit(
                u32::from(byte >> HEX_DIGIT_BITS),
                HEX_RADIX,
            ));
            code.extend(char::from_digit(u32::from(byte & HEX_LOW_MASK), HEX_RADIX));
        }
        let pending = Pending {
            digest: blake3::hash(code.as_bytes()),
            token: SecretString::from(token),
            expires: Instant::now() + self.lifetime,
        };
        *self.lock() = Some(pending);
        Some(SecretString::from(code.as_str()))
    }

    /// Whether a code is waiting to be used. An expired code is removed and does not count.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        let mut slot = self.lock();
        Self::drop_expired(&mut slot, Instant::now());
        slot.is_some()
    }

    /// Takes the token that `presented` stands for. The code works once: it is gone afterwards.
    /// A wrong code changes nothing, so that a stranger cannot use up the code of the user.
    #[must_use]
    pub fn redeem(&self, presented: &str) -> Option<SecretString> {
        self.redeem_at(presented, Instant::now())
    }

    fn redeem_at(&self, presented: &str, now: Instant) -> Option<SecretString> {
        let mut slot = self.lock();
        Self::drop_expired(&mut slot, now);
        let matches = slot
            .as_ref()
            .is_some_and(|pending| pending.digest == blake3::hash(presented.as_bytes()));
        if matches {
            slot.take().map(|pending| pending.token)
        } else {
            None
        }
    }

    fn drop_expired(slot: &mut Option<Pending>, now: Instant) {
        if slot.as_ref().is_some_and(|pending| pending.expires <= now) {
            *slot = None;
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Pending>> {
        self.slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl std::fmt::Debug for LaunchCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(LaunchCodes))
            .finish_non_exhaustive()
    }
}

/// The text of `secret`; only for the one place that puts it into an answer.
pub(crate) fn reveal(secret: &SecretString) -> &str {
    secret.expose_secret()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "csk_token";

    fn store() -> LaunchCodes {
        LaunchCodes::new(Duration::from_secs(60))
    }

    #[test]
    fn a_code_is_hex_of_128_bits_and_unique() {
        let codes = store();
        let first = codes.issue(TOKEN).unwrap();
        let second = codes.issue(TOKEN).unwrap();
        assert_eq!(reveal(&first).len(), LAUNCH_CODE_HEX_LEN);
        assert!(reveal(&first).chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(reveal(&first), reveal(&second));
    }

    #[test]
    fn a_code_works_once() {
        let codes = store();
        let code = codes.issue(TOKEN).unwrap();
        assert!(codes.is_pending());
        let token = codes.redeem(reveal(&code)).unwrap();
        assert_eq!(reveal(&token), TOKEN);
        assert!(codes.redeem(reveal(&code)).is_none());
        assert!(!codes.is_pending());
    }

    #[test]
    fn a_new_code_replaces_the_old_one() {
        let codes = store();
        let old = codes.issue(TOKEN).unwrap();
        let new = codes.issue(TOKEN).unwrap();
        assert!(codes.redeem(reveal(&old)).is_none());
        assert!(codes.redeem(reveal(&new)).is_some());
    }

    #[test]
    fn a_wrong_code_does_not_use_up_the_right_one() {
        let codes = store();
        let code = codes.issue(TOKEN).unwrap();
        assert!(codes.redeem("wrong").is_none());
        assert!(codes.redeem("").is_none());
        assert!(codes.redeem(reveal(&code)).is_some());
    }

    #[test]
    fn an_expired_code_is_gone() {
        let codes = LaunchCodes::new(Duration::ZERO);
        let code = codes.issue(TOKEN).unwrap();
        assert!(!codes.is_pending());
        assert!(codes.redeem(reveal(&code)).is_none());
        let live = store();
        let code = live.issue(TOKEN).unwrap();
        let later = Instant::now() + Duration::from_secs(61);
        assert!(live.redeem_at(reveal(&code), later).is_none());
    }

    #[test]
    fn debug_output_shows_nothing() {
        let codes = store();
        assert!(codes.issue(TOKEN).is_some());
        assert!(!format!("{codes:?}").contains("csk_"));
    }
}
