//! The text of the commands `scans`, `compare` and `change` (ADR 0097), made from the same JSON
//! answer as `--json`, and the Markdown of `change --markdown` for a comment on a pull request.
//!
//! Every text from the project (names, paths) loses its control characters; in the Markdown it
//! also loses its backticks, so a name cannot break out of the code span it stands in.

use codeskimr_codes::i18n::{
    CHANGE_CAUSE_CHANGED, CHANGE_CAUSE_REMOVED, CHANGE_HEAD, CHANGE_MD_CHANGED,
    CHANGE_MD_CHECKLIST, CHANGE_MD_CRITICAL, CHANGE_MD_CRITICAL_LINE, CHANGE_MD_CUT,
    CHANGE_MD_CYCLES, CHANGE_MD_EDGE, CHANGE_MD_ITEM, CHANGE_MD_NONE, CHANGE_MD_NOTE,
    CHANGE_MD_OPEN, CHANGE_MD_TITLE, CHANGE_MD_USAGES, CHANGE_MD_VIOLATIONS, CHANGE_NO_IMPACT,
    CHANGE_STATUS_ADAPTED, CHANGE_STATUS_NOT_TOUCHED, CHANGE_SYMBOL, CHANGE_USAGE_LINE,
    CHANGE_USAGES, COMPARE_ADDED, COMPARE_CHANGED, COMPARE_CHANGED_LINE, COMPARE_COUNTS,
    COMPARE_CRITICAL, COMPARE_CRITICAL_LINE, COMPARE_CUT, COMPARE_CYCLES, COMPARE_DIRECT_ONLY_NOTE,
    COMPARE_DURATION, COMPARE_EDGE_LINE, COMPARE_HEAD, COMPARE_MOVED, COMPARE_MOVED_LINE,
    COMPARE_NARROWED, COMPARE_REMOVED, COMPARE_SCANS_HEAD, COMPARE_SCANS_LINE, COMPARE_SCANS_NONE,
    COMPARE_SYMBOL_LINE, COMPARE_UNKNOWN, COMPARE_VIOLATIONS,
};
use codeskimr_i18n::{FluentValue, I18nError, Messages};
use serde_json::Value;

use crate::consts::change::{
    APOSTROPHE, BACKTICK, CENTURY, CENTURY_DAYS, EPOCH_SHIFT, ERA_DAYS, FOUR_YEARS_DAYS,
    LEAP_CYCLE, LIST_SECTIONS, MARCH, MARCH_BASED_NEW_YEAR, MONTH_CYCLE_DAYS, MONTH_CYCLE_MONTHS,
    MONTH_OFFSET, MONTHS_SHIFT_BACK, NOT_TOUCHED_STATUS, REASON_SEPARATOR, REMOVED_CAUSE,
    SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, SUMMARY_ARGUMENTS, YEAR_DAYS,
    YEARS_PER_ERA, field_name, name,
};
use crate::consts::{LINE_BREAK, LIST_INDENT, MARKDOWN_MARKER, key};
use crate::speech::{
    Line, Speaker, child, count, entries, field, json_list, plain, section, text, texts,
    usage_line as query_usage_line,
};

/// What a value of the answer is, as a number; the text for "not known" if it is `null`.
fn number_or_unknown(speak: &Speaker<'_>, value: &Value, name: &str) -> Result<String, I18nError> {
    match value.get(name).and_then(Value::as_u64) {
        Some(number) => Ok(number.to_string()),
        None => speak.say(COMPARE_UNKNOWN, &[]),
    }
}

/// A duration as text: milliseconds, or "not known".
fn duration_text(speak: &Speaker<'_>, value: &Value, name: &str) -> Result<String, I18nError> {
    match value.get(name).and_then(Value::as_u64) {
        Some(ms) => speak.say(COMPARE_DURATION, &[(name::MS, FluentValue::from(ms))]),
        None => speak.say(COMPARE_UNKNOWN, &[]),
    }
}

/// Seconds since the epoch as `YYYY-MM-DD HH:MM` in UTC.
fn time_text(seconds: i64) -> String {
    let days = seconds.div_euclid(SECONDS_PER_DAY);
    let rest = seconds.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let (hour, minute) = (
        rest / SECONDS_PER_HOUR,
        rest % SECONDS_PER_HOUR / SECONDS_PER_MINUTE,
    );
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

/// The calendar date of a day counted from 1970-01-01 (the proleptic Gregorian calendar).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + EPOCH_SHIFT;
    let era = shifted.div_euclid(ERA_DAYS);
    let day_of_era = shifted.rem_euclid(ERA_DAYS);
    let year_of_era = (day_of_era - day_of_era / FOUR_YEARS_DAYS + day_of_era / CENTURY_DAYS
        - day_of_era / (ERA_DAYS - 1))
        / YEAR_DAYS;
    let day_of_year =
        day_of_era - (YEAR_DAYS * year_of_era + year_of_era / LEAP_CYCLE - year_of_era / CENTURY);
    let month_index = (MONTH_CYCLE_MONTHS * day_of_year + MONTH_OFFSET) / MONTH_CYCLE_DAYS;
    let day =
        day_of_year - (MONTH_CYCLE_DAYS * month_index + MONTH_OFFSET) / MONTH_CYCLE_MONTHS + 1;
    let month = if month_index < MARCH_BASED_NEW_YEAR {
        month_index + MARCH
    } else {
        month_index - MONTHS_SHIFT_BACK
    };
    let year = year_of_era + era * YEARS_PER_ERA + i64::from(month <= MONTH_OFFSET);
    (year, month, day)
}

fn created(value: &Value) -> String {
    time_text(
        value
            .get(field_name::CREATED)
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    )
}

fn scan_line(speak: &Speaker<'_>, scan: &Value) -> Result<String, I18nError> {
    let commit = field(scan, field_name::COMMIT);
    speak.say(
        COMPARE_SCANS_LINE,
        &[
            (name::SNAPSHOT, count(scan, field_name::SNAPSHOT)),
            (name::CREATED, text(created(scan))),
            (name::MODE, text(field(scan, field_name::MODE))),
            (name::FILES, count(scan, field_name::FILES)),
            (
                name::CHANGED,
                text(number_or_unknown(speak, scan, field_name::FILES_CHANGED)?),
            ),
            (
                name::DURATION,
                text(duration_text(speak, scan, field_name::DURATION)?),
            ),
            (name::COMMIT, text(commit)),
        ],
    )
}

/// `scans`: the retained scans, newest first.
pub(crate) fn scans_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let scans = entries(answer, field_name::SCANS);
    if scans.is_empty() {
        return Ok(vec![speak.say(COMPARE_SCANS_NONE, &[])?]);
    }
    let head = [
        (name::COUNT, FluentValue::from(scans.len())),
        (name::KEEP, count(answer, field_name::KEEP)),
        (
            name::AVERAGE,
            text(duration_text(speak, answer, field_name::AVERAGE)?),
        ),
    ];
    let mut lines = vec![speak.say(COMPARE_SCANS_HEAD, &head)?];
    for scan in scans {
        lines.push([LIST_INDENT, scan_line(speak, scan)?.as_str()].concat());
    }
    Ok(lines)
}

/// A line about a symbol: kind, name and one more field (`last`: argument name, field name).
fn symbol_with(
    speak: &Speaker<'_>,
    (message, entry): (&str, &Value),
    last: (&str, &str),
) -> Result<String, I18nError> {
    let pairs = [
        (name::KIND, key::KIND),
        (name::NAME, key::QUALIFIED_NAME),
        last,
    ];
    speak.say(message, &texts(entry, &pairs))
}

fn symbol_line(speak: &Speaker<'_>, symbol: &Value) -> Result<String, I18nError> {
    symbol_with(
        speak,
        (COMPARE_SYMBOL_LINE, symbol),
        (name::PATH, key::PATH),
    )
}

/// The reasons of a change as one text, with the mark for a narrowed visibility.
fn reasons_text(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    let mut reasons: Vec<String> = entries(entry, field_name::REASONS)
        .iter()
        .filter_map(|reason| reason.as_str().map(plain))
        .collect();
    if entry.get(field_name::NARROWED).and_then(Value::as_bool) == Some(true) {
        reasons.push(speak.say(COMPARE_NARROWED, &[])?);
    }
    Ok(reasons.join(REASON_SEPARATOR))
}

/// The arguments that name the target side of a changed or moved symbol.
fn matched_args(entry: &Value) -> Vec<(&'static str, FluentValue<'static>)> {
    let pairs = [
        (name::KIND, key::KIND),
        (name::NAME, key::QUALIFIED_NAME),
        (name::PATH, key::PATH),
    ];
    texts(&child(entry, field_name::TARGET_SYMBOL), &pairs)
}

fn changed_line(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    let mut args = matched_args(entry);
    args.push((name::REASONS, text(reasons_text(speak, entry)?)));
    speak.say(COMPARE_CHANGED_LINE, &args)
}

fn moved_line(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    let mut args = matched_args(entry);
    let from = field(&child(entry, field_name::BASE_SYMBOL), key::PATH);
    args.push((name::FROM, text(from)));
    speak.say(COMPARE_MOVED_LINE, &args)
}

fn critical_line(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    let last = (name::OUTCOME, field_name::OUTCOME);
    symbol_with(speak, (COMPARE_CRITICAL_LINE, entry), last)
}

fn edge_line(speak: &Speaker<'_>, edge: &Value) -> Result<String, I18nError> {
    let mut args = texts(edge, &[(name::FROM, key::FROM), (name::TO, field_name::TO)]);
    args.push((name::COUNT, count(edge, key::COUNT)));
    speak.say(COMPARE_EDGE_LINE, &args)
}

/// The critical symbols and the new layer violations and cycles of the answer.
fn consequences(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let layers = child(answer, field_name::LAYERS);
    let critical = json_list(answer, field_name::CRITICAL);
    let violations = json_list(&layers, field_name::NEW_VIOLATIONS);
    let cycles = json_list(&layers, field_name::NEW_CYCLES);
    let mut lines = Vec::new();
    lines.extend(section(
        speak,
        (COMPARE_CRITICAL, &critical),
        critical_line,
    )?);
    lines.extend(section(
        speak,
        (COMPARE_VIOLATIONS, &violations),
        edge_line,
    )?);
    lines.extend(section(speak, (COMPARE_CYCLES, &cycles), edge_line)?);
    Ok(lines)
}

fn scan_time(answer: &Value, which: &str) -> String {
    created(&child(answer, which))
}

/// `compare`: the symbols that were added, removed, changed or moved, and what that touches.
pub(crate) fn compare_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let (base, target) = (
        child(answer, field_name::BASE),
        child(answer, field_name::TARGET),
    );
    let head = [
        (name::BASE, count(&base, field_name::SNAPSHOT)),
        (name::BASE_TIME, text(scan_time(answer, field_name::BASE))),
        (name::TARGET, count(&target, field_name::SNAPSHOT)),
        (
            name::TARGET_TIME,
            text(scan_time(answer, field_name::TARGET)),
        ),
    ];
    let total = |which: &str| count(&child(answer, which), key::TOTAL);
    let counts = [
        (name::ADDED, total(field_name::ADDED)),
        (name::REMOVED, total(field_name::REMOVED)),
        (name::CHANGED, total(field_name::CHANGED)),
        (name::MOVED, total(field_name::MOVED)),
    ];
    let mut lines = vec![
        speak.say(COMPARE_HEAD, &head)?,
        speak.say(COMPARE_COUNTS, &counts)?,
    ];
    let lists: [(&str, &str, Line); LIST_SECTIONS] = [
        (COMPARE_ADDED, field_name::ADDED, symbol_line),
        (COMPARE_REMOVED, field_name::REMOVED, symbol_line),
        (COMPARE_CHANGED, field_name::CHANGED, changed_line),
        (COMPARE_MOVED, field_name::MOVED, moved_line),
    ];
    for (heading, list, line) in lists {
        lines.extend(section(speak, (heading, &child(answer, list)), line)?);
    }
    lines.extend(consequences(speak, answer)?);
    if is_cut(answer) {
        lines.push(speak.say(COMPARE_CUT, &[])?);
    }
    Ok(lines)
}

fn is_cut(answer: &Value) -> bool {
    answer.get(field_name::IS_CUT).and_then(Value::as_bool) == Some(true)
}

/// The cause of an impact as text: removed, or changed with its reasons.
fn cause_text(speak: &Speaker<'_>, entry: &Value) -> Result<String, I18nError> {
    if field(entry, field_name::CAUSE) == REMOVED_CAUSE {
        return speak.say(CHANGE_CAUSE_REMOVED, &[]);
    }
    let reasons = [(name::REASONS, text(reasons_text(speak, entry)?))];
    speak.say(CHANGE_CAUSE_CHANGED, &reasons)
}

fn status_text(speak: &Speaker<'_>, usage: &Value) -> Result<String, I18nError> {
    let message = if field(usage, field_name::STATUS) == NOT_TOUCHED_STATUS {
        CHANGE_STATUS_NOT_TOUCHED
    } else {
        CHANGE_STATUS_ADAPTED
    };
    speak.say(message, &[])
}

/// A usage as `usages` shows it, with whether the change adapted it or not.
fn usage_line(speak: &Speaker<'_>, usage: &Value) -> Result<String, I18nError> {
    let status = [
        (name::LINE, text(query_usage_line(speak, usage)?)),
        (name::STATUS, text(status_text(speak, usage)?)),
    ];
    speak.say(CHANGE_USAGE_LINE, &status)
}

/// A symbol of the impact with its cause, and under it its usages.
fn impact_block(speak: &Speaker<'_>, entry: &Value) -> Result<Vec<String>, I18nError> {
    let symbol = child(entry, field_name::SYMBOL);
    let head = speak.say(
        CHANGE_SYMBOL,
        &[
            (name::KIND, text(field(&symbol, key::KIND))),
            (name::NAME, text(field(&symbol, key::QUALIFIED_NAME))),
            (name::PATH, text(field(&symbol, key::PATH))),
            (name::CAUSE, text(cause_text(speak, entry)?)),
        ],
    )?;
    let mut lines = vec![head];
    for usage in entries(&child(entry, key::USAGES), key::ITEMS) {
        lines.push([LIST_INDENT, usage_line(speak, usage)?.as_str()].concat());
    }
    Ok(lines)
}

/// The numbers at the head of a change: the symbols and the usages.
fn summary_args(answer: &Value) -> [(&'static str, FluentValue<'static>); SUMMARY_ARGUMENTS] {
    let summary = child(answer, field_name::SUMMARY);
    let base = child(answer, field_name::BASE);
    let target = child(answer, field_name::TARGET);
    [
        (name::BASE, count(&base, field_name::SNAPSHOT)),
        (name::TARGET, count(&target, field_name::SNAPSHOT)),
        (name::CHANGED, count(&summary, field_name::CHANGED_SYMBOLS)),
        (name::PUBLIC, count(&summary, field_name::PUBLIC_SYMBOLS)),
        (name::CERTAIN, count(&summary, field_name::USAGES_CERTAIN)),
        (name::PROBABLE, count(&summary, field_name::USAGES_PROBABLE)),
        (name::OPEN, count(&summary, field_name::USAGES_NOT_TOUCHED)),
    ]
}

/// `change`: the symbols a change can break with their usages, those that were not touched, and
/// what the change does to the layers.
pub(crate) fn change_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let args = summary_args(answer);
    let mut lines = vec![
        speak.say(CHANGE_HEAD, &args)?,
        speak.say(CHANGE_USAGES, &args)?,
    ];
    let impact = child(answer, field_name::IMPACT);
    let symbols = entries(&impact, key::ITEMS);
    if symbols.is_empty() {
        lines.push(speak.say(CHANGE_NO_IMPACT, &[])?);
    }
    for entry in symbols {
        lines.extend(impact_block(speak, entry)?);
    }
    lines.extend(consequences(speak, answer)?);
    if is_cut(answer) {
        lines.push(speak.say(COMPARE_CUT, &[])?);
    }
    lines.push(speak.say(COMPARE_DIRECT_ONLY_NOTE, &[])?);
    Ok(lines)
}

/// A name or path for a code span of Markdown: no control characters, no backticks.
fn code(value: &str) -> String {
    plain(value).replace(BACKTICK, APOSTROPHE)
}

fn markdown_checklist(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let impact = child(answer, field_name::IMPACT);
    let mut lines = Vec::new();
    for entry in entries(&impact, key::ITEMS) {
        let symbol = field(&child(entry, field_name::SYMBOL), key::QUALIFIED_NAME);
        let cause = cause_text(speak, entry)?;
        for usage in entries(&child(entry, key::USAGES), key::ITEMS) {
            if field(usage, field_name::STATUS) != NOT_TOUCHED_STATUS {
                continue;
            }
            lines.push(speak.say(
                CHANGE_MD_ITEM,
                &[
                    (name::PATH, text(code(&field(usage, key::PATH)))),
                    (name::LINE, count(usage, key::LINE)),
                    (name::KIND, text(plain(&field(usage, key::KIND)))),
                    (name::SYMBOL, text(code(&symbol))),
                    (name::CAUSE, text(plain(&cause))),
                    (
                        name::HOLDER,
                        text(code(&field(&child(usage, key::FROM), key::QUALIFIED_NAME))),
                    ),
                ],
            )?);
        }
    }
    Ok(lines)
}

/// A line of the Markdown made from the fields `pairs` of `entry`, each in a code span.
fn markdown_line(
    speak: &Speaker<'_>,
    message: &str,
    (entry, pairs): (&Value, &[(&str, &str)]),
    more: Vec<(&str, FluentValue<'static>)>,
) -> Result<String, I18nError> {
    let mut args: Vec<(&str, FluentValue<'static>)> = pairs
        .iter()
        .map(|(argument, name)| (*argument, text(code(&field(entry, name)))))
        .collect();
    args.extend(more);
    speak.say(message, &args)
}

/// A heading and its lines, nothing if there are no lines. A blank line comes before the heading.
fn titled(
    speak: &Speaker<'_>,
    heading: &str,
    lines: Vec<String>,
) -> Result<Vec<String>, I18nError> {
    if lines.is_empty() {
        return Ok(Vec::new());
    }
    let mut block = vec![String::new(), speak.say(heading, &[])?];
    block.extend(lines);
    Ok(block)
}

fn markdown_sections(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let layers = child(answer, field_name::LAYERS);
    let edges = |list: &str| -> Result<Vec<String>, I18nError> {
        entries(&layers, list)
            .iter()
            .map(|edge| {
                let pairs = [(name::FROM, key::FROM), (name::TO, field_name::TO)];
                let more = vec![(name::COUNT, count(edge, key::COUNT))];
                markdown_line(speak, CHANGE_MD_EDGE, (edge, &pairs), more)
            })
            .collect()
    };
    let critical = entries(answer, field_name::CRITICAL)
        .iter()
        .map(|entry| {
            let pairs = [
                (name::KIND, key::KIND),
                (name::NAME, key::QUALIFIED_NAME),
                (name::OUTCOME, field_name::OUTCOME),
            ];
            markdown_line(speak, CHANGE_MD_CRITICAL_LINE, (entry, &pairs), Vec::new())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut lines = titled(
        speak,
        CHANGE_MD_VIOLATIONS,
        edges(field_name::NEW_VIOLATIONS)?,
    )?;
    lines.extend(titled(
        speak,
        CHANGE_MD_CYCLES,
        edges(field_name::NEW_CYCLES)?,
    )?);
    lines.extend(titled(speak, CHANGE_MD_CRITICAL, critical)?);
    Ok(lines)
}

/// The Markdown of `change --markdown`: the marker line, the numbers, the checklist of the usages
/// that were not touched, the new layer violations and cycles, and what the comparison does not
/// see. It is made from the same answer as `--json`.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn change_markdown(
    messages: &Messages,
    language: &str,
    answer: &Value,
) -> Result<String, I18nError> {
    let speak = Speaker { messages, language };
    let args = summary_args(answer);
    let mut lines = vec![
        MARKDOWN_MARKER.to_owned(),
        speak.say(CHANGE_MD_TITLE, &args)?,
        String::new(),
        speak.say(CHANGE_MD_CHANGED, &args)?,
        speak.say(CHANGE_MD_USAGES, &args)?,
        speak.say(CHANGE_MD_OPEN, &args)?,
    ];
    let checklist = markdown_checklist(&speak, answer)?;
    if checklist.is_empty() {
        lines.extend([String::new(), speak.say(CHANGE_MD_NONE, &[])?]);
    } else {
        lines.extend(titled(&speak, CHANGE_MD_CHECKLIST, checklist)?);
    }
    lines.extend(markdown_sections(&speak, answer)?);
    lines.push(String::new());
    if is_cut(answer) {
        lines.extend([speak.say(CHANGE_MD_CUT, &[])?, String::new()]);
    }
    let note = [(name::NOTE, text(speak.say(COMPARE_DIRECT_ONLY_NOTE, &[])?))];
    lines.push(speak.say(CHANGE_MD_NOTE, &note)?);
    Ok(lines.join(LINE_BREAK))
}

#[cfg(test)]
#[path = "change_text_tests.rs"]
mod tests;
