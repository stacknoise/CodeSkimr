//! Masks secrets in log text (blueprint S-01): URL user info, `Authorization` headers, bearer
//! tokens, well-known token formats and the values of fields named like secrets.
//!
//! Masking errs on the safe side. A value that merely looks like a secret is hidden too.

use codeskimr_source::redact_credentials;

use crate::consts::{
    BEARER_KEY, BLANKS, KEY_SEPARATORS, LINE_END, LINE_KEYS, QUOTES, REDACTED, TOKEN_EXTRA_CHARS,
    TOKEN_MIN_TAIL, TOKEN_PREFIXES, VALUE_END, VALUE_KEYS,
};

/// Where the value of a key ends and how the key is followed by it.
#[derive(Clone, Copy)]
struct Shape<'a> {
    /// Characters between key and value; empty if the value follows the key directly.
    separators: &'a [char],
    /// The value runs to the end of the line instead of the next delimiter.
    to_line_end: bool,
}

/// Returns `text` with every recognised secret replaced by a placeholder.
#[must_use]
pub fn redact(text: &str) -> String {
    let mut result = redact_credentials(text);
    for key in LINE_KEYS {
        result = mask_values(&result, key, line_shape());
    }
    for key in VALUE_KEYS {
        result = mask_values(&result, key, word_shape(&KEY_SEPARATORS));
    }
    result = mask_values(&result, BEARER_KEY, word_shape(&[]));
    for prefix in TOKEN_PREFIXES {
        result = mask_tokens(&result, prefix);
    }
    result
}

fn line_shape() -> Shape<'static> {
    Shape {
        separators: &KEY_SEPARATORS,
        to_line_end: true,
    }
}

fn word_shape(separators: &[char]) -> Shape<'_> {
    Shape {
        separators,
        to_line_end: false,
    }
}

/// The part of `text` from byte `from` on; empty if `from` is out of range.
fn tail(text: &str, from: usize) -> &str {
    text.get(from..).unwrap_or_default()
}

/// The part of `text` between the byte positions `from` and `to`.
fn between(text: &str, from: usize, to: usize) -> &str {
    text.get(from..to).unwrap_or_default()
}

/// Replaces the value after every occurrence of `key` (any case) that is followed by a value.
fn mask_values(text: &str, key: &str, shape: Shape<'_>) -> String {
    // ASCII lowercasing keeps every byte position, so positions found here fit `text`.
    let lower = text.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut position = 0;
    while let Some(offset) = tail(&lower, position).find(key) {
        let key_end = position + offset + key.len();
        result.push_str(between(text, position, key_end));
        position = key_end;
        if let Some((start, quote)) = value_start(tail(text, key_end), shape.separators) {
            let value = tail(text, key_end + start);
            let length = value_length(value, quote, shape.to_line_end);
            result.push_str(between(text, key_end, key_end + start));
            result.push_str(REDACTED);
            position = key_end + start + length;
        }
    }
    result.push_str(tail(text, position));
    result
}

/// Where the value begins inside `rest` (the text behind a key), and the quote that opened it.
fn value_start(rest: &str, separators: &[char]) -> Option<(usize, Option<char>)> {
    let after_key = rest.trim_start_matches(QUOTES);
    let after_separator = if separators.is_empty() {
        after_key
    } else {
        after_key
            .trim_start_matches(BLANKS)
            .strip_prefix(separators)?
    };
    let value = after_separator.trim_start_matches(BLANKS);
    let quote = value.chars().next().filter(|c| QUOTES.contains(c));
    let body = quote.map_or(value, |q| value.strip_prefix(q).unwrap_or(value));
    (!body.is_empty()).then_some((rest.len() - body.len(), quote))
}

/// Length of the value at the start of `value`.
fn value_length(value: &str, quote: Option<char>, to_line_end: bool) -> usize {
    if value.starts_with(REDACTED) {
        return REDACTED.len();
    }
    let end = |c: char| match quote {
        Some(q) => c == q || c == LINE_END,
        None if to_line_end => c == LINE_END,
        None => c.is_whitespace() || VALUE_END.contains(&c),
    };
    value.find(end).unwrap_or(value.len())
}

/// Replaces every well-known token that starts with `prefix`.
fn mask_tokens(text: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut position = 0;
    while let Some(offset) = tail(text, position).find(prefix) {
        let start = position + offset;
        let body = start + prefix.len();
        let run = tail(text, body);
        // Token characters are ASCII, so the length in bytes is the length in characters.
        let length = run.find(|c| !is_token_char(c)).unwrap_or(run.len());
        if starts_word(text, start) && length >= TOKEN_MIN_TAIL {
            result.push_str(between(text, position, start));
            result.push_str(REDACTED);
            position = body + length;
        } else {
            result.push_str(between(text, position, body));
            position = body;
        }
    }
    result.push_str(tail(text, position));
    result
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || TOKEN_EXTRA_CHARS.contains(&c)
}

/// True if the character before `start` does not continue a word, so `risk-...` is no `sk-` token.
fn starts_word(text: &str, start: usize) -> bool {
    !between(text, 0, start)
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::redact;

    const CANARY: &str = "CANARY7f3a9c1e5b2d4f60";

    fn assert_hidden(input: &str) {
        let output = redact(input);
        assert!(!output.contains(CANARY), "leaked: {output}");
        assert!(output.contains("[redacted]"), "nothing masked: {output}");
    }

    #[test]
    fn masks_url_user_info() {
        let output = redact(&format!(
            "clone https://alice:{CANARY}@host.example/r.git failed"
        ));
        assert_eq!(output, "clone https://host.example/r.git failed");
    }

    #[test]
    fn masks_authorization_headers_in_all_shapes() {
        assert_hidden(&format!("Authorization: Bearer {CANARY}"));
        assert_hidden(&format!("authorization=Basic {CANARY}=="));
        assert_hidden(&format!(
            "{{\"Authorization\": \"Bearer {CANARY}\", \"x\": 1}}"
        ));
        assert_hidden(&format!("proxy-authorization: Basic {CANARY}"));
        assert_hidden(&format!(
            "headers: {{\"authorization\": \"Basic {CANARY}\"}}"
        ));
        assert_hidden(&format!("cookie: sid={CANARY}; theme=dark"));
    }

    #[test]
    fn authorization_masks_the_scheme_too_and_keeps_other_lines() {
        let output = redact(&format!("Authorization: Bearer {CANARY}\nnext line"));
        assert_eq!(output, "Authorization: [redacted]\nnext line");
    }

    #[test]
    fn masks_bearer_tokens_without_header_name() {
        assert_hidden(&format!("sent Bearer {CANARY} to server"));
        assert_hidden(&format!("sent bearer \"{CANARY}\" to server"));
    }

    #[test]
    fn masks_fields_named_like_secrets() {
        assert_hidden(&format!("login password={CANARY} user=bob"));
        assert_hidden(&format!("GET /x?access_token={CANARY}&page=2"));
        assert_hidden(&format!("{{\"api_key\":\"{CANARY}\"}}"));
        assert_hidden(&format!("token: '{CANARY}'"));
        let output = redact(&format!("password={CANARY} user=bob"));
        assert_eq!(output, "password=[redacted] user=bob");
    }

    #[test]
    fn masks_well_known_token_formats() {
        for token in [
            format!("ghp_{CANARY}"),
            format!("github_pat_11{CANARY}_abc"),
            format!("glpat-{CANARY}"),
            format!("xoxb-123456-{CANARY}"),
            format!("AKIA{CANARY}"),
            format!("sk-ant-{CANARY}"),
            format!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIx.{CANARY}"),
        ] {
            assert_hidden(&format!("value {token} end"));
        }
    }

    #[test]
    fn keeps_ordinary_text() {
        for text in [
            "scanned 12 files, 3 tokens in the query",
            "risk-assessment-framework-for-everyone",
            "task-queue-management-system-ready",
            "token counts: 5",
            "Authentication failed for the project",
            "short sk-abc and ghp_short",
            "no secrets here ünïcode",
            "",
        ] {
            assert_eq!(redact(text), text);
        }
    }

    #[test]
    fn is_stable_when_applied_twice() {
        let once = redact(&format!("password={CANARY} Bearer {CANARY} ghp_{CANARY}"));
        assert_eq!(redact(&once), once);
    }

    #[test]
    fn survives_unusual_input() {
        for text in [
            "password",
            "password=",
            "password=\"",
            "authorization:",
            "Bearer ",
            "ghp_",
            "ÄÖÜ password=ß",
        ] {
            let _ = redact(text);
        }
    }
}
