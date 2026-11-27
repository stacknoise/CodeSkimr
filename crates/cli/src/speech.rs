//! The pieces the texts of the query commands are made of: the speaker of one language, the
//! reading of fields of an answer, and the sections and lines several commands share.
//!
//! Every text from the project (names, paths, documentation, notes) loses its control
//! characters: a terminal must not act on what a stranger wrote into a comment.

use codeskimr_codes::i18n::CLI_QUERY_USAGE_LINE;
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde_json::{Map, Value};

use crate::consts::{CONTROL_REPLACEMENT, LIST_INDENT, arg, key};

/// The messages in one language.
pub(crate) struct Speaker<'a> {
    pub(crate) messages: &'a Messages,
    pub(crate) language: &'a str,
}

pub(crate) type Args<'a> = [(&'a str, FluentValue<'static>)];

impl Speaker<'_> {
    pub(crate) fn say(&self, message: &str, args: &Args<'_>) -> Result<String, I18nError> {
        let mut named = FluentArgs::new();
        for (name, value) in args {
            named.set(*name, value.clone());
        }
        self.messages.format(self.language, message, Some(&named))
    }
}

/// The text without control characters; a line break or tab becomes a space.
pub(crate) fn plain(text: &str) -> String {
    text.chars()
        .map(|found| {
            if found.is_control() {
                CONTROL_REPLACEMENT
            } else {
                found
            }
        })
        .collect()
}

/// A field of `value` as a plain text; empty if it is missing or `null`.
pub(crate) fn field(value: &Value, name: &str) -> String {
    match value.get(name) {
        Some(Value::String(text)) => plain(text),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

/// The texts of the fields `pairs` name (argument name, field name) of `entry`, as arguments.
pub(crate) fn texts<'a>(
    entry: &Value,
    pairs: &[(&'a str, &str)],
) -> Vec<(&'a str, FluentValue<'static>)> {
    pairs
        .iter()
        .map(|(argument, name)| (*argument, text(field(entry, name))))
        .collect()
}

pub(crate) fn text(value: String) -> FluentValue<'static> {
    FluentValue::from(value)
}

pub(crate) fn count(value: &Value, name: &str) -> FluentValue<'static> {
    FluentValue::from(value.get(name).and_then(Value::as_u64).unwrap_or_default())
}

pub(crate) fn child(value: &Value, name: &str) -> Value {
    value.get(name).cloned().unwrap_or_default()
}

pub(crate) fn entries<'a>(value: &'a Value, name: &str) -> &'a [Value] {
    value
        .get(name)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

pub(crate) fn usage_line(speak: &Speaker<'_>, usage: &Value) -> Result<String, I18nError> {
    let from = field(&child(usage, key::FROM), key::QUALIFIED_NAME);
    speak.say(
        CLI_QUERY_USAGE_LINE,
        &[
            (arg::PATH, text(field(usage, key::PATH))),
            (arg::LINE, count(usage, key::LINE)),
            (arg::COLUMN, count(usage, key::COLUMN)),
            (arg::KIND, text(field(usage, key::KIND))),
            (arg::CONFIDENCE, text(field(usage, key::CONFIDENCE))),
            (arg::NAME, text(from)),
        ],
    )
}

pub(crate) type Line = fn(&Speaker<'_>, &Value) -> Result<String, I18nError>;

/// A list of `context` with a heading that says how many of how many are shown; nothing if it
/// is empty.
pub(crate) fn section(
    speak: &Speaker<'_>,
    (heading, list): (&str, &Value),
    line: Line,
) -> Result<Vec<String>, I18nError> {
    section_with(speak, (heading, list), (line, LIST_INDENT))
}

/// `section` with the indent of its lines chosen.
pub(crate) fn section_with(
    speak: &Speaker<'_>,
    (heading, list): (&str, &Value),
    (line, indent): (Line, &str),
) -> Result<Vec<String>, I18nError> {
    let listed = entries(list, key::ITEMS);
    if listed.is_empty() {
        return Ok(Vec::new());
    }
    let numbers = [
        (arg::SHOWN, FluentValue::from(listed.len())),
        (arg::TOTAL, count(list, key::TOTAL)),
    ];
    let mut lines = vec![speak.say(heading, &numbers)?];
    for entry in listed {
        lines.push([indent, line(speak, entry)?.as_str()].concat());
    }
    Ok(lines)
}

/// A list field of the answer as the `{ total, items }` form that [`section`] reads.
pub(crate) fn json_list(answer: &Value, name: &str) -> Value {
    let items = child(answer, name);
    let total = items.as_array().map_or(0, Vec::len);
    Value::Object(Map::from_iter([
        (key::TOTAL.to_owned(), Value::from(total)),
        (key::ITEMS.to_owned(), items),
    ]))
}
