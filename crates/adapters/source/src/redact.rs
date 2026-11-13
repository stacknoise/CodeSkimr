//! Removing credentials from texts before they are stored or shown (blueprint E-05).

use codeskimr_domain::without_userinfo;

/// Removes `user:secret@` from every URL in `text`.
///
/// `https://user:token@host/repo` becomes `https://host/repo`. Text without credentials is
/// returned unchanged.
#[must_use]
pub fn redact_credentials(text: &str) -> String {
    without_userinfo(text)
}
