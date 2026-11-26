//! The output of the query commands: JSON for machines (the answer of the MCP tool of the same
//! name, with `schema` and `stale`), text for people.
//!
//! The text is made from that same answer, so both say the same. Every text from the project
//! (names, paths, documentation, notes) loses its control characters: a terminal must not act on
//! what a stranger wrote into a comment.

use codeskimr_codes::i18n::{
    CLI_CONTEXT_CALLERS, CLI_CONTEXT_DEPENDENCIES, CLI_CONTEXT_DEPENDENCY_LINE, CLI_CONTEXT_FILE,
    CLI_CONTEXT_FILES, CLI_CONTEXT_HEAD, CLI_CONTEXT_LAYER, CLI_CONTEXT_NOTE_LINE,
    CLI_CONTEXT_NOTES, CLI_CONTEXT_SIGNATURE, CLI_QUERY_CUT, CLI_QUERY_IMPACT_LINE, CLI_QUERY_NONE,
    CLI_QUERY_PRELIMINARY, CLI_QUERY_RULES_HEAD, CLI_QUERY_RULES_LINE, CLI_QUERY_RULES_NONE,
    CLI_QUERY_RULES_RULE, CLI_QUERY_STALE, CLI_QUERY_SYMBOL_LINE, CLI_QUERY_UNUSED_LEFT_OUT,
    CLI_QUERY_UNUSED_LEFT_OUT_LINE, CLI_QUERY_UNUSED_LINE, CLI_QUERY_USAGE_LINE,
    CLI_RENAME_CONFLICT_LINE, CLI_RENAME_CONFLICTS, CLI_RENAME_DECLARATION_LINE,
    CLI_RENAME_DISCLAIMER, CLI_RENAME_HEAD, CLI_RENAME_NOT_SEARCHED, CLI_RENAME_PUBLIC,
    CLI_RENAME_SITES, CLI_RENAME_TEXT_LINE, CLI_RENAME_TEXTS,
};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde_json::{Map, Value};

use crate::change_text::{change_lines, compare_lines, scans_lines};
use crate::consts::{
    CONTROL_REPLACEMENT, JSON_NULL_TEXT, KIND_DECLARATION, LINE_BREAK, LIST_INDENT,
    QUERY_SCHEMA_VERSION, TITLE_SEPARATOR, arg, key,
};

/// Which query command an answer belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    /// `symbols`.
    Symbols,
    /// `usages`.
    Usages,
    /// `impact`.
    Impact,
    /// `context`.
    Context,
    /// `unused`.
    Unused,
    /// `rules`.
    Rules,
    /// `preview-rename`.
    Rename,
    /// `scans`.
    Scans,
    /// `compare`.
    Compare,
    /// `change`.
    Change,
}

/// The answer of a tool as the JSON of a command: the fields of the answer plus `schema` (the
/// version of this format) and `stale` (`true` if files changed since the scan the answer is
/// based on, `null` if that cannot be told, for example for a Git source).
///
/// # Errors
/// A JSON error if the text cannot be built, which does not happen for a `Value`.
pub fn query_json(answer: Value, stale: Option<bool>) -> Result<String, serde_json::Error> {
    let mut fields = match answer {
        Value::Object(fields) => fields,
        other => Map::from_iter([(key::RESULT.to_owned(), other)]),
    };
    fields.insert(key::SCHEMA.to_owned(), Value::from(QUERY_SCHEMA_VERSION));
    fields.insert(key::STALE.to_owned(), Value::from(stale));
    serde_json::to_string_pretty(&Value::Object(fields))
}

/// The JSON of a failure: `schema` and `error` with the registered code, its description and its
/// parameters, as the MCP tools report them. It cannot fail: should the pretty form not be
/// built, the compact one of the same value is returned.
#[must_use]
pub fn query_error_json(report: &Value) -> String {
    let mut fields = Map::new();
    fields.insert(key::SCHEMA.to_owned(), Value::from(QUERY_SCHEMA_VERSION));
    fields.insert(key::ERROR.to_owned(), report.clone());
    let value = Value::Object(fields);
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
}

/// The report of a failure that only knows its code, for when the full one cannot be built.
#[must_use]
pub fn query_code_report(code: &str) -> Value {
    Value::Object(Map::from_iter([(key::CODE.to_owned(), Value::from(code))]))
}

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

/// A line about a symbol: kind, name, file, id and the step a walk reached it in.
fn named_line(speak: &Speaker<'_>, message: &str, found: &Value) -> Result<String, I18nError> {
    let more = [
        (arg::ID, count(found, key::ID)),
        (arg::DEPTH, count(found, key::DEPTH)),
    ];
    identified_line(speak, message, found, more)
}

/// A line that starts with what names a symbol (kind, qualified name, file) and adds `more`.
fn identified_line<const N: usize>(
    speak: &Speaker<'_>,
    message: &str,
    found: &Value,
    more: [(&'static str, FluentValue<'static>); N],
) -> Result<String, I18nError> {
    let mut named = vec![
        (arg::KIND, text(field(found, key::KIND))),
        (arg::NAME, text(field(found, key::QUALIFIED_NAME))),
        (arg::PATH, text(field(found, key::PATH))),
    ];
    named.extend(more);
    speak.say(message, &named)
}

fn symbol_line(speak: &Speaker<'_>, hit: &Value) -> Result<String, I18nError> {
    named_line(speak, CLI_QUERY_SYMBOL_LINE, hit)
}

fn impact_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    named_line(speak, CLI_QUERY_IMPACT_LINE, found)
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

fn dependency_line(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    let symbol = child(entry, key::SYMBOL);
    speak.say(
        CLI_CONTEXT_DEPENDENCY_LINE,
        &[
            (arg::COUNT, count(entry, key::COUNT)),
            (arg::KIND, text(field(&symbol, key::KIND))),
            (arg::NAME, text(field(&symbol, key::QUALIFIED_NAME))),
            (arg::PATH, text(field(entry, key::PATH))),
        ],
    )
}

fn file_line(speak: &Speaker<'_>, file: &Value) -> Result<String, I18nError> {
    let path = plain(file.as_str().unwrap_or_default());
    speak.say(CLI_CONTEXT_FILE, &[(arg::PATH, text(path))])
}

fn note_line(speak: &Speaker<'_>, note: &Value) -> Result<String, I18nError> {
    let said = [field(note, key::TITLE), field(note, key::TEXT)];
    let both = said.join(TITLE_SEPARATOR);
    speak.say(
        CLI_CONTEXT_NOTE_LINE,
        &[
            (arg::KIND, text(field(note, key::KIND))),
            (arg::TEXT, text(both.trim().to_owned())),
        ],
    )
}

fn unused_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let more = [
        (arg::ASSESSMENT, text(field(found, key::ASSESSMENT))),
        (arg::REASON, text(field(found, key::REASON))),
    ];
    identified_line(speak, CLI_QUERY_UNUSED_LINE, found, more)
}

fn left_out_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let reason = field(&child(found, key::RULE), key::REASON);
    identified_line(
        speak,
        CLI_QUERY_UNUSED_LEFT_OUT_LINE,
        found,
        [(arg::TEXT, text(reason))],
    )
}

fn conflict_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let more = [(arg::CLASH, text(field(found, key::SAME_SIGNATURE)))];
    identified_line(speak, CLI_RENAME_CONFLICT_LINE, found, more)
}

fn text_hit_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    speak.say(
        CLI_RENAME_TEXT_LINE,
        &[
            (arg::PATH, text(field(found, key::PATH))),
            (arg::LINE, count(found, key::LINE)),
            (arg::KIND, text(field(found, key::KIND))),
            (arg::TEXT, text(field(found, key::TEXT))),
        ],
    )
}

/// A place of `preview-rename`: a declaration or a usage, as `usages` shows it.
fn site_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    if field(found, key::KIND) != KIND_DECLARATION {
        return usage_line(speak, found);
    }
    speak.say(
        CLI_RENAME_DECLARATION_LINE,
        &[
            (arg::PATH, text(field(found, key::PATH))),
            (arg::LINE, count(found, key::LINE)),
            (arg::COLUMN, count(found, key::COLUMN)),
        ],
    )
}

/// A place of `preview-rename` and, under it, the line of code it stands on if the answer has one.
fn site_block(speak: &Speaker<'_>, found: &Value) -> Result<Vec<String>, I18nError> {
    let mut block = vec![[LIST_INDENT, site_line(speak, found)?.as_str()].concat()];
    if let Some(code) = found.get(key::SITE_CODE).and_then(Value::as_str) {
        block.push([LIST_INDENT, LIST_INDENT, code].concat());
    }
    Ok(block)
}

pub(crate) type Line = fn(&Speaker<'_>, &Value) -> Result<String, I18nError>;

/// A list of `context` with a heading that says how many of how many are shown; nothing if it
/// is empty.
pub(crate) fn section(
    speak: &Speaker<'_>,
    (heading, list): (&str, &Value),
    line: Line,
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
        lines.push([LIST_INDENT, line(speak, entry)?.as_str()].concat());
    }
    Ok(lines)
}

/// The head of `context`: what the symbol is, its signature, layer and documentation.
fn context_head(speak: &Speaker<'_>, subject: &Value) -> Result<Vec<String>, I18nError> {
    let head = [
        (arg::KIND, text(field(subject, key::KIND))),
        (arg::NAME, text(field(subject, key::QUALIFIED_NAME))),
        (arg::PATH, text(field(subject, key::PATH))),
        (arg::LINE, count(subject, key::FIRST_LINE)),
    ];
    let mut lines = vec![speak.say(CLI_CONTEXT_HEAD, &head)?];
    for (message, name) in [
        (CLI_CONTEXT_SIGNATURE, key::SIGNATURE),
        (CLI_CONTEXT_LAYER, key::LAYER),
    ] {
        let value = field(subject, name);
        if !value.is_empty() && value != JSON_NULL_TEXT {
            lines.push(speak.say(message, &[(arg::TEXT, text(value))])?);
        }
    }
    let doc = field(subject, key::DOC);
    if !doc.is_empty() {
        lines.push(doc);
    }
    Ok(lines)
}

fn context_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let mut lines = context_head(speak, &child(answer, key::SYMBOL))?;
    let sections: &[(&str, &str, Line)] = &[
        (CLI_CONTEXT_CALLERS, key::CALLERS, usage_line),
        (CLI_CONTEXT_DEPENDENCIES, key::DEPENDENCIES, dependency_line),
        (CLI_CONTEXT_FILES, key::AFFECTED_FILES, file_line),
        (CLI_CONTEXT_NOTES, key::ANNOTATIONS, note_line),
    ];
    for &(heading, name, line) in sections {
        lines.extend(section(speak, (heading, &child(answer, name)), line)?);
    }
    Ok(lines)
}

fn list_lines(
    speak: &Speaker<'_>,
    (answer, list): (&Value, &str),
    line: Line,
) -> Result<Vec<String>, I18nError> {
    let listed = entries(answer, list);
    if listed.is_empty() {
        return Ok(vec![speak.say(CLI_QUERY_NONE, &[])?]);
    }
    listed.iter().map(|entry| line(speak, entry)).collect()
}

/// The symbols without a usage, then those the exception list takes out with their reason.
fn unused_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let mut lines = list_lines(speak, (answer, key::UNUSED), unused_line)?;
    let left_out = entries(answer, key::LEFT_OUT);
    if !left_out.is_empty() {
        let numbers = [
            (arg::SHOWN, FluentValue::from(left_out.len())),
            (
                arg::TOTAL,
                child(&child(answer, key::COUNTS), key::LEFT_OUT)
                    .as_u64()
                    .map_or_else(|| FluentValue::from(left_out.len()), FluentValue::from),
            ),
        ];
        lines.push(speak.say(CLI_QUERY_UNUSED_LEFT_OUT, &numbers)?);
        for entry in left_out {
            lines.push([LIST_INDENT, left_out_line(speak, entry)?.as_str()].concat());
        }
    }
    Ok(lines)
}

fn rule_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let place = entries(found, key::PLACES)
        .first()
        .cloned()
        .unwrap_or_default();
    let numbers = [
        (arg::FROM, text(field(found, key::FROM))),
        (arg::TO, text(field(found, key::TO))),
        (arg::PATH, text(field(&place, key::PATH))),
        (arg::LINE, count(&place, key::LINE)),
        (arg::COLUMN, count(&place, key::COLUMN)),
        (
            arg::HELD,
            text(
                found
                    .get(key::OLD)
                    .and_then(Value::as_bool)
                    .unwrap_or_default()
                    .to_string(),
            ),
        ),
        (
            arg::PLACES,
            FluentValue::from(entries(found, key::PLACES).len()),
        ),
    ];
    speak.say(CLI_QUERY_RULES_LINE, &numbers)
}

/// The rules with their violations, the held ones marked, under a line that counts them.
fn rules_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let rules = entries(answer, key::RULES);
    if rules.is_empty() {
        return Ok(vec![speak.say(CLI_QUERY_RULES_NONE, &[])?]);
    }
    let head = [
        (arg::NEW, count(answer, key::NEW)),
        (arg::HELD, count(answer, key::OLD)),
        (arg::RESOLVED, count(answer, key::RESOLVED)),
    ];
    let mut lines = vec![speak.say(CLI_QUERY_RULES_HEAD, &head)?];
    for rule in rules {
        let numbers = [
            (arg::RULE, text(field(rule, key::ID))),
            (arg::NEW, count(rule, key::NEW)),
            (arg::TOTAL, count(rule, key::TOTAL)),
        ];
        lines.push(speak.say(CLI_QUERY_RULES_RULE, &numbers)?);
        for found in entries(rule, key::VIOLATIONS) {
            lines.push([LIST_INDENT, rule_line(speak, found)?.as_str()].concat());
        }
    }
    Ok(lines)
}

/// The preview of a rename: what the places, conflicts and text lines are, and what it cannot see.
fn rename_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let sites = child(answer, key::SITES);
    let texts = child(answer, key::TEXT_HITS);
    let head = [
        (
            arg::NAME,
            text(field(&child(answer, key::SYMBOL), key::QUALIFIED_NAME)),
        ),
        (arg::NEW, text(field(answer, key::NEW_NAME))),
    ];
    let numbers = [
        (arg::SHOWN, count(&sites, key::TOTAL)),
        (arg::FILES, count(&sites, key::FILES)),
        (arg::VERIFIED, count(&sites, key::VERIFIED)),
    ];
    let mut lines = vec![
        speak.say(CLI_RENAME_HEAD, &head)?,
        speak.say(CLI_RENAME_SITES, &numbers)?,
    ];
    for entry in entries(&sites, key::ITEMS) {
        lines.extend(site_block(speak, entry)?);
    }
    let conflicts = json_list(answer, key::CONFLICTS);
    lines.extend(section(
        speak,
        (CLI_RENAME_CONFLICTS, &conflicts),
        conflict_line,
    )?);
    if texts.get(key::SEARCHED).and_then(Value::as_bool) == Some(true) {
        lines.extend(section(speak, (CLI_RENAME_TEXTS, &texts), text_hit_line)?);
    } else {
        lines.push(speak.say(CLI_RENAME_NOT_SEARCHED, &[])?);
    }
    if answer.get(key::PUBLIC_INTERFACE).and_then(Value::as_bool) == Some(true) {
        lines.push(speak.say(CLI_RENAME_PUBLIC, &[])?);
    }
    lines.push(speak.say(CLI_RENAME_DISCLAIMER, &[])?);
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

fn body(speak: &Speaker<'_>, kind: QueryKind, answer: &Value) -> Result<Vec<String>, I18nError> {
    match kind {
        QueryKind::Symbols => list_lines(speak, (answer, key::HITS), symbol_line),
        QueryKind::Usages => list_lines(speak, (answer, key::USAGES), usage_line),
        QueryKind::Impact => list_lines(speak, (answer, key::SYMBOLS), impact_line),
        QueryKind::Context => context_lines(speak, answer),
        QueryKind::Unused => unused_lines(speak, answer),
        QueryKind::Rules => rules_lines(speak, answer),
        QueryKind::Rename => rename_lines(speak, answer),
        QueryKind::Scans => scans_lines(speak, answer),
        QueryKind::Compare => compare_lines(speak, answer),
        QueryKind::Change => change_lines(speak, answer),
    }
}

/// The notes below a list: that it was cut, that it is preliminary, that it may be out of date.
fn footer(
    speak: &Speaker<'_>,
    answer: &Value,
    stale: Option<bool>,
) -> Result<Vec<String>, I18nError> {
    let cut = answer
        .get(key::NOTICE)
        .is_some_and(|notice| !notice.is_null());
    let preliminary = answer.get(key::PRELIMINARY).and_then(Value::as_bool) == Some(true);
    let mut lines = Vec::new();
    for (wanted, message) in [
        (cut, CLI_QUERY_CUT),
        (preliminary, CLI_QUERY_PRELIMINARY),
        (stale == Some(true), CLI_QUERY_STALE),
    ] {
        if wanted {
            lines.push(speak.say(message, &[])?);
        }
    }
    Ok(lines)
}

/// The answer as lines for a terminal.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn query_text(
    messages: &Messages,
    language: &str,
    kind: QueryKind,
    (answer, stale): (&Value, Option<bool>),
) -> Result<String, I18nError> {
    let speak = Speaker { messages, language };
    let mut lines = body(&speak, kind, answer)?;
    lines.extend(footer(&speak, answer, stale)?);
    Ok(lines.join(LINE_BREAK))
}
