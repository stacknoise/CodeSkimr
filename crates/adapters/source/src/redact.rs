//! Removing credentials from texts before they are stored or shown (blueprint E-05).

use crate::consts::{URL_AUTHORITY_END, URL_SCHEME_MARKER, URL_USERINFO_END};

/// Removes `user:secret@` from every URL in `text`.
///
/// `https://user:token@host/repo` becomes `https://host/repo`. Text without credentials is
/// returned unchanged.
#[must_use]
pub fn redact_credentials(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find(URL_SCHEME_MARKER) {
        let (head, tail) = rest.split_at(position + URL_SCHEME_MARKER.len());
        result.push_str(head);
        rest = strip_userinfo(tail, &mut result);
    }
    result.push_str(rest);
    result
}

/// Copies the authority of `tail` to `out` without its user info and returns what follows it.
fn strip_userinfo<'a>(tail: &'a str, out: &mut String) -> &'a str {
    let end = tail
        .find(|c: char| URL_AUTHORITY_END.contains(&c) || c.is_whitespace())
        .unwrap_or(tail.len());
    let (authority, remainder) = tail.split_at(end);
    let host = authority
        .rsplit_once(URL_USERINFO_END)
        .map_or(authority, |(_, host)| host);
    out.push_str(host);
    remainder
}

#[cfg(test)]
mod tests {
    use super::redact_credentials;

    #[test]
    fn removes_user_and_token() {
        assert_eq!(
            redact_credentials("failed https://user:tok@host.example/r.git: denied"),
            "failed https://host.example/r.git: denied"
        );
    }

    #[test]
    fn keeps_hosts_and_text_and_strips_user_only_urls() {
        let text = "see https://host.example/a@b and ssh://git@host/x? no";
        assert_eq!(
            redact_credentials(text),
            "see https://host.example/a@b and ssh://host/x? no"
        );
        assert_eq!(redact_credentials("nothing here"), "nothing here");
    }
}
