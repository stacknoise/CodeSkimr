//! Minimal reader for Fluent files. It only extracts message ids; syntax checks belong to the
//! `codeskimr_i18n` tests, which parse every file with the real Fluent parser.

/// `true` for characters allowed in a Fluent message id.
fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// Message ids defined at the start of a line, in file order.
///
/// Comments, attributes (`.name = …`), indented continuation lines and variants are skipped.
#[must_use]
pub fn message_ids(ftl: &str) -> Vec<String> {
    ftl.lines()
        .filter(|line| line.starts_with(|c: char| c.is_ascii_alphabetic()))
        .filter_map(|line| line.split_once('='))
        .map(|(id, _)| id.trim_end())
        .filter(|id| id.chars().all(is_id_char))
        .map(str::to_string)
        .collect()
}
