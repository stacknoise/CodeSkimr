//! Removing credentials from URLs, so that no secret is stored, shown or handed to Git.

use crate::consts::{
    SCP_HOST_EXTRA, SECRET_USER_SCHEMES, URL_AUTHORITY_END, URL_IPV6_START, URL_PASSWORD_START,
    URL_SCHEME_EXTRA, URL_SCHEME_MARKER, URL_USERINFO_END,
};

/// What to cut out of the credentials part (`user:password@`) of a URL.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cut {
    /// Everything for web schemes, whose user can be a token; only the password otherwise
    /// (`ssh://git@host` keeps `git`).
    Secrets,
    /// Everything, for texts that are shown to people.
    All,
}

/// The URL as Git may see and store it: no password, and for `http` and `https` no user either,
/// because a token in the user position is a secret as well.
#[must_use]
pub fn without_secrets(url: &str) -> String {
    rewrite(url, Cut::Secrets)
}

/// The text with the credentials removed from every URL in it (blueprint E-05). When unsure where
/// the credentials end it removes more rather than less.
#[must_use]
pub fn without_userinfo(text: &str) -> String {
    without_scp_credentials(&rewrite(text, Cut::All))
}

/// Removes `user:password@` from scp-style addresses (`user:password@host:path`), which have no
/// `://` and so are not found by [`rewrite`]. A word counts only if the credentials contain a
/// colon and the part after the last `@` is `host:path`; `name@host:22` and addresses without a
/// password stay as they are.
fn without_scp_credentials(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for chunk in text.split_inclusive(char::is_whitespace) {
        let word = chunk.trim_end_matches(char::is_whitespace);
        result.push_str(scp_without_credentials(word));
        result.push_str(chunk.get(word.len()..).unwrap_or_default());
    }
    result
}

/// `word` without its credentials if it is an scp-style address with a password.
fn scp_without_credentials(word: &str) -> &str {
    let Some((userinfo, address)) = word.rsplit_once(URL_USERINFO_END) else {
        return word;
    };
    let has_password =
        userinfo.contains(URL_PASSWORD_START) && !userinfo.contains(URL_SCHEME_MARKER);
    let is_address = address
        .split_once(URL_PASSWORD_START)
        .is_some_and(|(host, path)| {
            !host.is_empty()
                && !path.is_empty()
                && !path.chars().all(|c| c.is_ascii_digit())
                && host
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || SCP_HOST_EXTRA.contains(c))
        });
    if has_password && is_address {
        address
    } else {
        word
    }
}

fn rewrite(text: &str, cut: Cut) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find(URL_SCHEME_MARKER) {
        let (head, tail) = rest.split_at(position + URL_SCHEME_MARKER.len());
        result.push_str(head);
        let (userinfo, remainder) = split_userinfo(tail);
        if let Some(userinfo) = userinfo {
            let user = kept(userinfo, scheme_of(head), cut);
            result.push_str(user);
            if !user.is_empty() {
                result.push(URL_USERINFO_END);
            }
        }
        rest = remainder;
    }
    result.push_str(rest);
    result
}

/// The part of `userinfo` that stays.
fn kept<'a>(userinfo: &'a str, scheme: &str, cut: Cut) -> &'a str {
    let web = SECRET_USER_SCHEMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(scheme));
    if cut == Cut::All || web {
        return <&str>::default();
    }
    userinfo
        .split(URL_PASSWORD_START)
        .next()
        .unwrap_or_default()
}

/// The scheme in front of the `://` at the end of `head`.
fn scheme_of(head: &str) -> &str {
    let without_marker = head.strip_suffix(URL_SCHEME_MARKER).unwrap_or(head);
    let start = without_marker
        .rfind(|c: char| !c.is_ascii_alphanumeric() && !URL_SCHEME_EXTRA.contains(c))
        .map_or(0, |index| index + 1);
    without_marker.get(start..).unwrap_or_default()
}

/// Splits `tail` (the text after `://`) into the credentials and what follows them. Without
/// credentials the whole `tail` is returned as the second part.
///
/// The credentials end at the last `@` of the first word. Normally the authority ends earlier
/// (at `/`, `?`, `#`), but a password may contain such a character unescaped: `u:pa/ss@host`. A
/// colon whose right side is no port number shows that case; a path like `host:80/a@b` is left
/// alone.
fn split_userinfo(tail: &str) -> (Option<&str>, &str) {
    let word_end = tail.find(char::is_whitespace).unwrap_or(tail.len());
    let word = tail.get(..word_end).unwrap_or(tail);
    let authority_end = word.find(URL_AUTHORITY_END).unwrap_or(word.len());
    let authority = word.get(..authority_end).unwrap_or(word);
    let at = match authority.rfind(URL_USERINFO_END) {
        Some(at) => Some(at),
        None if looks_like_password(authority) => word.rfind(URL_USERINFO_END),
        None => None,
    };
    match at {
        Some(at) => (tail.get(..at), tail.get(at + 1..).unwrap_or_default()),
        None => (None, tail),
    }
}

/// Whether `authority` has the form `name:something` where `something` is no port.
fn looks_like_password(authority: &str) -> bool {
    if authority.starts_with(URL_IPV6_START) {
        return false;
    }
    authority
        .split_once(URL_PASSWORD_START)
        .is_some_and(|(_, after)| !after.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::{without_secrets, without_userinfo};

    #[test]
    fn web_urls_lose_user_and_password_and_other_schemes_only_the_password() {
        assert_eq!(
            without_secrets("https://user:tok@host.example/r.git"),
            "https://host.example/r.git"
        );
        assert_eq!(
            without_secrets("https://ghp_TOKEN@github.com/o/r"),
            "https://github.com/o/r"
        );
        assert_eq!(without_secrets("ssh://git:pw@host/x"), "ssh://git@host/x");
        assert_eq!(without_secrets("ssh://git@host/x"), "ssh://git@host/x");
        assert_eq!(without_secrets("file:///tmp/repo"), "file:///tmp/repo");
    }

    #[test]
    fn urls_without_credentials_stay_as_they_are() {
        for url in [
            "https://host.example/a@b",
            "https://host:8080/a@b",
            "https://[::1]:8080/a@b",
            "https://host.example/r.git?x=a@b",
        ] {
            assert_eq!(without_secrets(url), url);
        }
    }

    #[test]
    fn delimiters_inside_a_password_do_not_hide_it() {
        for url in [
            "https://u:pa/ss@h/r",
            "https://u:pa#ss@h/r",
            "https://u:pa?ss@h/r",
            "https://u:p@ss@h/r",
            "https://@h/r",
        ] {
            let clean = without_secrets(url);
            assert!(!clean.contains("ss") || clean.ends_with("h/r"));
            assert!(!clean.contains("pa") && !clean.contains("u:"));
        }
    }

    #[test]
    fn texts_lose_the_credentials_of_every_url() {
        assert_eq!(
            without_userinfo("failed https://user:tok@host.example/r.git: denied"),
            "failed https://host.example/r.git: denied"
        );
        assert_eq!(
            without_userinfo("see https://host.example/a@b and ssh://git@host/x? no"),
            "see https://host.example/a@b and ssh://host/x? no"
        );
        assert_eq!(without_userinfo("nothing here"), "nothing here");
        assert_eq!(without_userinfo("https://u:pa/ss@h/r"), "https://h/r");
    }

    #[test]
    fn scp_style_addresses_lose_the_password() {
        assert_eq!(
            without_userinfo("clone user:pa/ss@host.example:org/r.git failed\nnext"),
            "clone host.example:org/r.git failed\nnext"
        );
        assert_eq!(without_userinfo("u:p@ss@h:r"), "h:r");
    }

    #[test]
    fn scp_style_text_without_a_password_stays() {
        for text in [
            "git@host.example:org/r.git",
            "see user@host:22 now",
            "a: b@c",
            "x:y@host",
        ] {
            assert_eq!(without_userinfo(text), text);
        }
    }
}
