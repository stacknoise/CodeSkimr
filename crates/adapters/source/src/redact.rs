//! Removing credentials from texts before they are stored or shown (blueprint E-05).

use crate::consts::{URL_AUTHORITY_END, URL_PASSWORD_START, URL_SCHEME_MARKER, URL_USERINFO_END};

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

/// Removes the password from every URL in `url`: `https://user:token@host/r` becomes
/// `https://user@host/r`. The user name stays because Git needs it (for example `git@` over SSH).
#[must_use]
pub fn without_password(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    let mut rest = url;
    while let Some(position) = rest.find(URL_SCHEME_MARKER) {
        let (head, tail) = rest.split_at(position + URL_SCHEME_MARKER.len());
        result.push_str(head);
        let end = authority_end(tail);
        let (authority, remainder) = tail.split_at(end);
        result.push_str(&drop_password(authority));
        rest = remainder;
    }
    result.push_str(rest);
    result
}

/// Where the authority of the URL in `tail` ends.
fn authority_end(tail: &str) -> usize {
    tail.find(|c: char| URL_AUTHORITY_END.contains(&c) || c.is_whitespace())
        .unwrap_or(tail.len())
}

/// `user:password@host` without the password; an authority without user info is unchanged.
fn drop_password(authority: &str) -> String {
    let Some((userinfo, host)) = authority.rsplit_once(URL_USERINFO_END) else {
        return authority.to_owned();
    };
    let user = userinfo
        .split(URL_PASSWORD_START)
        .next()
        .unwrap_or(userinfo);
    format!("{user}{URL_USERINFO_END}{host}")
}

/// Copies the authority of `tail` to `out` without its user info and returns what follows it.
fn strip_userinfo<'a>(tail: &'a str, out: &mut String) -> &'a str {
    let (authority, remainder) = tail.split_at(authority_end(tail));
    let host = authority
        .rsplit_once(URL_USERINFO_END)
        .map_or(authority, |(_, host)| host);
    out.push_str(host);
    remainder
}

#[cfg(test)]
mod tests {
    use super::{redact_credentials, without_password};

    #[test]
    fn a_password_is_dropped_but_the_user_stays() {
        assert_eq!(
            without_password("https://user:tok@host.example/r.git"),
            "https://user@host.example/r.git"
        );
        assert_eq!(
            without_password("ssh://git@host/x and https://h/p"),
            "ssh://git@host/x and https://h/p"
        );
        assert_eq!(without_password("file:///tmp/repo"), "file:///tmp/repo");
    }

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
