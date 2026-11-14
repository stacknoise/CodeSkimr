//! The key names of TOML and Fluent texts. Values are skipped, including strings over several
//! lines and arrays or inline tables over several lines, so nothing inside a value is ever taken
//! for a key (a value may hold secrets).

use crate::consts::{
    ASSIGNMENT, BACKSLASH, COMMENT_START, KEY_SEPARATOR, LINE_END_CHAR, NAME_EXTRA, QUOTE,
    SINGLE_QUOTE, TABLE_CLOSE, TABLE_OPEN, TABLE_VALUE_CLOSE, TABLE_VALUE_OPEN, TOML_KEY_EXTRA,
    TRIPLE_QUOTE, TRIPLE_SINGLE_QUOTE,
};

/// Where the reader of a TOML text stands between two lines: inside a value that goes on, or not.
#[derive(Default)]
struct Value {
    /// Open `[` and `{` of the value.
    depth: u32,
    /// The closing delimiter of a string over several lines that is still open.
    multiline: Option<&'static str>,
}

impl Value {
    fn is_open(&self) -> bool {
        self.depth > 0 || self.multiline.is_some()
    }

    /// Reads `text`, the rest of a line, and updates what is open.
    fn advance(&mut self, text: &str) {
        let mut rest = text;
        while let Some(next) = rest.chars().next() {
            if let Some(closing) = self.multiline {
                let Some(at) = rest.find(closing) else {
                    return;
                };
                self.multiline = None;
                rest = rest.get(at + closing.len()..).unwrap_or_default();
                continue;
            }
            rest = rest.get(next.len_utf8()..).unwrap_or_default();
            match next {
                COMMENT_START => return,
                TABLE_OPEN | TABLE_VALUE_OPEN => self.depth += 1,
                TABLE_CLOSE | TABLE_VALUE_CLOSE => self.depth = self.depth.saturating_sub(1),
                QUOTE | SINGLE_QUOTE => rest = self.string(next, rest),
                _ => {}
            }
        }
    }

    /// Skips a string that started with `quote`; `rest` follows the opening quote. A string over
    /// several lines (`"""` or `'''`) stays open in `self`.
    fn string<'a>(&mut self, quote: char, rest: &'a str) -> &'a str {
        let triple = if quote == QUOTE {
            TRIPLE_QUOTE
        } else {
            TRIPLE_SINGLE_QUOTE
        };
        // `rest` follows the first quote; the other two make it a triple.
        if let Some(after) = rest.strip_prefix(triple.get(quote.len_utf8()..).unwrap_or_default()) {
            self.multiline = Some(triple);
            return after;
        }
        let mut escaped = false;
        for (at, found) in rest.char_indices() {
            if found == quote && !escaped {
                return rest.get(at + found.len_utf8()..).unwrap_or_default();
            }
            escaped = quote == QUOTE && found == BACKSLASH && !escaped;
        }
        ""
    }
}

/// Whether `key` looks like a TOML key: bare or quoted parts joined by dots.
fn is_toml_key(key: &str) -> bool {
    !key.is_empty()
        && key.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || TOML_KEY_EXTRA.contains(c)
                || c == QUOTE
                || c == SINGLE_QUOTE
        })
}

/// The dotted key paths of a TOML text with their zero-based lines. Section headers set the
/// prefix; a line `key = value` outside a value that goes on over several lines is a key.
pub(crate) fn toml_keys(text: &str) -> Vec<(String, usize)> {
    let mut prefix = String::new();
    let mut value = Value::default();
    let mut found = Vec::new();
    for (line, raw) in text.split(LINE_END_CHAR).enumerate() {
        let trimmed = raw.trim();
        if value.is_open() {
            value.advance(trimmed);
            continue;
        }
        if trimmed.starts_with(COMMENT_START) || trimmed.is_empty() {
            continue;
        }
        if let Some(header) = trimmed.strip_prefix(TABLE_OPEN) {
            let name = header
                .trim_start_matches(TABLE_OPEN)
                .split(TABLE_CLOSE)
                .next()
                .unwrap_or_default();
            name.trim().clone_into(&mut prefix);
        } else if let Some((key, rest)) = trimmed.split_once(ASSIGNMENT) {
            let key = key.trim();
            if is_toml_key(key) {
                let key = key.trim_matches(QUOTE);
                let full = if prefix.is_empty() {
                    key.to_owned()
                } else {
                    [prefix.as_str(), key].join(KEY_SEPARATOR)
                };
                found.push((full, line));
            }
            value.advance(rest);
        }
    }
    found
}

/// The ids of the messages of a Fluent text with their zero-based lines: a line that starts with
/// a letter, then name characters, then `=`. Terms (`-brand`), attributes, comments and the
/// continuation lines of a text (they start with a space) are not.
pub(crate) fn fluent_ids(text: &str) -> Vec<(String, usize)> {
    text.split(LINE_END_CHAR)
        .enumerate()
        .filter_map(|(line, raw)| {
            let (id, _) = raw.split_once(ASSIGNMENT)?;
            let id = id.trim_end();
            let mut chars = id.chars();
            let valid = chars
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || NAME_EXTRA.contains(c));
            valid.then(|| (id.to_owned(), line))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_keys_get_their_section_as_prefix() {
        let text = "top = 1\n# comment = 2\n[mcp]\ndefault_limit = 20\n[[rows]]\nname = \"x\"\n";
        assert_eq!(
            toml_keys(text),
            [
                ("top".to_owned(), 0),
                ("mcp.default_limit".to_owned(), 3),
                ("rows.name".to_owned(), 5)
            ]
        );
    }

    #[test]
    fn nothing_inside_a_value_is_a_key() {
        let text = concat!(
            "a = \"\"\"\nsecret = hunter2\n\"\"\"\n",
            "b = '''\ntoken = abc\n'''\n",
            "c = [\n  \"x = y\",\n  \"z\",\n]\n",
            "d = { e = 1,\n  f = 2 }\n",
            "g = \"quote \\\" = still in\"\n",
            "h = 1\n"
        );
        let names: Vec<String> = toml_keys(text).into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["a", "b", "c", "d", "g", "h"]);
    }

    #[test]
    fn fluent_ids_skip_terms_attributes_and_comments() {
        let text = "# note = x\nerror-MCP-501 = text\n-brand = Name\nhello =\n    .title = t\n";
        assert_eq!(
            fluent_ids(text),
            [("error-MCP-501".to_owned(), 1), ("hello".to_owned(), 3)]
        );
    }
}
