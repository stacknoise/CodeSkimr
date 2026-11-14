//! Masks secrets in log text (blueprint S-01): URL user info, `Authorization` headers, bearer
//! tokens, well-known token formats and the values of fields named like secrets.
//!
//! Masking errs on the safe side. A value that merely looks like a secret is hidden too.

use codeskimr_source::redact_credentials;

use crate::consts::{
    ARROW, BEARER_KEY, BLANKS, CALL_OPEN, ESCAPE, ESCAPE_PAIR, KEY_SEPARATORS, LINE_END, LINE_KEYS,
    PEM_BEGIN, PEM_END, PEM_PRIVATE, QUOTES, REDACTED, TOKEN_EXTRA_CHARS, TOKEN_MIN_TAIL,
    TOKEN_PREFIXES, VALUE_END, VALUE_KEYS,
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
    let mut result = mask_private_keys(&redact_credentials(text));
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
        if let Some((start, opening)) = value_start(tail(text, key_end), shape.separators) {
            let value = tail(text, key_end + start);
            let length = value_length(value, opening, shape.to_line_end);
            result.push_str(between(text, key_end, key_end + start));
            result.push_str(REDACTED);
            position = key_end + start + length;
        }
    }
    result.push_str(tail(text, position));
    result
}

/// The quote that opened a value, and whether `Debug` output escaped it with a backslash.
#[derive(Clone, Copy)]
struct Opening {
    quote: char,
    escaped: bool,
}

/// Where the value begins inside `rest` (the text behind a key), and the quote that opened it.
fn value_start(rest: &str, separators: &[char]) -> Option<(usize, Option<Opening>)> {
    let value = if separators.is_empty() {
        rest.strip_prefix(BLANKS)?.trim_start_matches(BLANKS)
    } else {
        after_separator(rest, separators)?
    };
    let unwrapped = strip_wrappers(value);
    // Debug output escapes the quotes of a JSON text with a backslash.
    let unescaped = unwrapped.trim_start_matches(ESCAPE);
    let quote = unescaped.chars().next().filter(|c| QUOTES.contains(c));
    let body = quote.map_or(unwrapped, |q| {
        unescaped.strip_prefix(q).unwrap_or(unwrapped)
    });
    let escaped = unescaped.len() != unwrapped.len();
    let opening = quote.map(|quote| Opening { quote, escaped });
    (!body.is_empty()).then_some((rest.len() - body.len(), opening))
}

/// What follows the separator behind a key. The key may continue (`secret_key`, `PASSWORD_HASH`)
/// and may be quoted or escaped (`"key":`, `\"key\":`).
fn after_separator<'a>(rest: &'a str, separators: &[char]) -> Option<&'a str> {
    let name_end = rest.trim_start_matches(is_token_char);
    let closed = name_end.trim_start_matches(|c| QUOTES.contains(&c) || c == ESCAPE);
    let after = closed.trim_start_matches(BLANKS).strip_prefix(separators)?;
    Some(after.trim_start_matches(|c| BLANKS.contains(&c) || c == ARROW))
}

/// Removes constructor calls that `Debug` puts around a value: `Some("x")`, `Token("x")`.
fn strip_wrappers(mut value: &str) -> &str {
    while let Some(inner) = wrapped_content(value) {
        value = inner;
    }
    value
}

/// The content of `Name(...)` if it starts with a quote or another constructor.
fn wrapped_content(value: &str) -> Option<&str> {
    let name = value.trim_start_matches(is_token_char);
    if !value.starts_with(|c: char| c.is_ascii_uppercase()) {
        return None;
    }
    let inner = name.strip_prefix(CALL_OPEN)?;
    inner
        .starts_with(|c: char| QUOTES.contains(&c) || c == ESCAPE || c.is_ascii_uppercase())
        .then_some(inner)
}

/// Length of the value at the start of `value`.
fn value_length(value: &str, opening: Option<Opening>, to_line_end: bool) -> usize {
    let mut backslashes = 0_usize;
    let mut end = |c: char| {
        // In a plain quoted value a quote behind an odd number of backslashes belongs to the
        // value (`\"`); behind an even number (`\\"`) the backslashes escape each other. An
        // escaped opening quote came from `Debug` output, whose closing quote is escaped too.
        let escaped = backslashes % ESCAPE_PAIR == 1;
        let ended = match opening {
            Some(open) => (c == open.quote && (open.escaped || !escaped)) || c == LINE_END,
            None if to_line_end => c == LINE_END,
            None => c.is_whitespace() || VALUE_END.contains(&c),
        };
        backslashes = if c == ESCAPE { backslashes + 1 } else { 0 };
        ended
    };
    value.find(&mut end).unwrap_or(value.len())
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

/// Replaces PEM private key blocks, from the header to the end of the footer line.
fn mask_private_keys(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut position = 0;
    while let Some(offset) = tail(text, position).find(PEM_BEGIN) {
        let start = position + offset;
        let block = tail(text, start);
        let header = block
            .find(LINE_END)
            .map_or(block, |end| between(block, 0, end));
        if !header.contains(PEM_PRIVATE) {
            result.push_str(between(text, position, start + PEM_BEGIN.len()));
            position = start + PEM_BEGIN.len();
            continue;
        }
        let footer = block.find(PEM_END).unwrap_or(block.len());
        let line_end = tail(block, footer)
            .find(LINE_END)
            .unwrap_or(block.len() - footer);
        result.push_str(between(text, position, start));
        result.push_str(REDACTED);
        position = start + footer + line_end;
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
    fn masks_secrets_in_debug_escaped_json() {
        for text in [
            format!(r#"\"password\":\"{CANARY}\""#),
            format!(r#"\"token\": \"{CANARY}\", \"x\": 1"#),
            format!(r#"\"Authorization\": \"Basic {CANARY}\""#),
            format!(r#"\\\"api_key\\\":\\\"{CANARY}\\\""#),
            format!(r"\'secret\': \'{CANARY}\'"),
        ] {
            assert_hidden(&text);
        }
    }

    #[test]
    fn masks_debug_shapes_of_options_and_newtypes() {
        assert_hidden(&format!("token: Some(\"{CANARY}\")"));
        assert_hidden(&format!("Token(\"{CANARY}\")"));
        assert_hidden(&format!("secret: Some(Token(\"{CANARY}\"))"));
        assert_hidden(&format!("\"password\" => \"{CANARY}\""));
        assert_hidden(&format!("password: Some(\\\"{CANARY}\\\")"));
    }

    #[test]
    fn masks_longer_key_names_and_values_with_separators() {
        assert_hidden(&format!("AWS_SECRET_ACCESS_KEY={CANARY}"));
        assert_hidden(&format!("secret_key={CANARY}"));
        assert_hidden(&format!("passphrase: {CANARY}"));
        let output = redact(&format!("password=ab,{CANARY};x) next"));
        assert_eq!(output, "password=[redacted] next");
    }

    #[test]
    fn masks_private_key_blocks_and_more_token_formats() {
        let pem = format!(
            "key -----BEGIN RSA PRIVATE KEY-----\n{CANARY}\n-----END RSA PRIVATE KEY----- done"
        );
        assert_eq!(redact(&pem), "key [redacted]");
        assert_hidden(&format!("-----BEGIN PRIVATE KEY-----\n{CANARY}"));
        assert_eq!(
            redact("-----BEGIN CERTIFICATE-----\nabc"),
            "-----BEGIN CERTIFICATE-----\nabc"
        );
        for token in [
            format!("ASIA{CANARY}"),
            format!("sk_live_{CANARY}"),
            format!("npm_{CANARY}"),
            format!("xapp-1-{CANARY}"),
        ] {
            assert_hidden(&format!("v {token} w"));
        }
    }

    #[test]
    fn masks_presigned_url_parameters() {
        assert_hidden(&format!("GET /f?X-Amz-Signature={CANARY}&X-Amz-Date=1"));
        assert_hidden(&format!("GET /f?X-Amz-Credential={CANARY}&x=1"));
        assert_hidden(&format!("GET /blob?sv=1&sig={CANARY}&se=2"));
        assert_hidden(&format!("GET /blob?sig={CANARY}"));
        assert_eq!(redact("GET /f?page=1&design=2"), "GET /f?page=1&design=2");
    }

    #[test]
    fn masks_scp_style_addresses_with_a_password() {
        let output = redact(&format!(
            "fetch user:{CANARY}@host.example:org/r.git failed"
        ));
        assert_eq!(output, "fetch host.example:org/r.git failed");
    }

    #[test]
    fn a_quoted_value_with_an_escaped_quote_is_masked_to_its_end() {
        let output = redact(&format!(r#"{{"password": "ab\"{CANARY}", "x": 1}}"#));
        assert!(!output.contains(CANARY), "leaked: {output}");
        assert_eq!(output, r#"{"password": "[redacted]", "x": 1}"#);
    }

    #[test]
    fn a_value_that_ends_in_a_backslash_ends_at_its_closing_quote() {
        let output = redact(&format!(r#"{{"password": "{CANARY}\\", "x": 1}}"#));
        assert_eq!(output, r#"{"password": "[redacted]", "x": 1}"#);
    }

    #[test]
    fn scp_style_edge_cases_are_pinned() {
        // Over-masking of a lookalike is accepted; `user:pw@host` has no path and stays.
        assert_eq!(redact("at 12:30@host:path ok"), "at host:path ok");
        assert_eq!(redact("see u:p@host ok"), "see u:p@host ok");
        // The opening quote of a quoted address goes with the credentials.
        assert_eq!(redact(r#""u:p@h:p""#.trim_matches('\\')), r#"h:p""#);
    }

    #[test]
    fn masks_bearer_after_a_tab() {
        assert_hidden(&format!("sent bearer\t{CANARY}"));
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
            format!("{}{CANARY}", codeskimr_server::TOKEN_PREFIX),
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
