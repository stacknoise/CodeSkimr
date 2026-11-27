//! The text of `compare --metrics` (ADR 0114), made from the same JSON answer as `--json`: which
//! objects got worse and which got better, with the metrics that moved.
//!
//! Every text from the project (names, paths) loses its control characters.

use codeskimr_application::compare::CROSSING_INTO_POOR;
use codeskimr_codes::i18n::{
    COMPARE_CUT, METRICS_BETTER, METRICS_HEAD, METRICS_INTO_POOR, METRICS_LINE, METRICS_MD_BETTER,
    METRICS_MD_COUNTS, METRICS_MD_LINE, METRICS_MD_NONE, METRICS_MD_TITLE, METRICS_MD_WORSE,
    METRICS_MOVE, METRICS_MOVE_CROSSING, METRICS_NAME_CC, METRICS_NAME_CE,
    METRICS_NAME_INSTABILITY, METRICS_NAME_LOC, METRICS_NAME_MI, METRICS_NONE, METRICS_OUT_OF_POOR,
    METRICS_WORSE,
};
use codeskimr_domain::MetricKind;
use codeskimr_i18n::{I18nError, Messages};
use serde_json::Value;

use crate::change_text::code;
use crate::consts::change::{field_name, name};
use crate::consts::metrics::{
    ARG_BETTER, ARG_CROSSING, ARG_FROM, ARG_LEVEL, ARG_LOOKED, ARG_METRIC, ARG_MOVE, ARG_MOVES,
    ARG_TO, ARG_WORSE, BETTER, CROSSING, FROM, LEVEL, LOOKED_AT, METRIC, MOVE_JOIN, MOVES,
    ROUNDING, TO, WORSE,
};
use crate::consts::{LINE_BREAK, METRICS_MARKDOWN_MARKER, NO_INDENT, arg, key};
use crate::speech::{Speaker, child, count, entries, field, section, section_with, text};

/// The name of a metric as text; its label if it has no name.
fn metric_name(speak: &Speaker<'_>, label: &str) -> Result<String, I18nError> {
    let known = [
        (MetricKind::Loc.label(), METRICS_NAME_LOC),
        (MetricKind::CyclomaticComplexity.label(), METRICS_NAME_CC),
        (MetricKind::MaintainabilityIndex.label(), METRICS_NAME_MI),
        (MetricKind::EfferentCoupling.label(), METRICS_NAME_CE),
        (MetricKind::Instability.label(), METRICS_NAME_INSTABILITY),
    ];
    known
        .iter()
        .find(|(label_of, _)| *label_of == label)
        .map_or_else(|| Ok(label.to_owned()), |(_, key)| speak.say(key, &[]))
}

/// A value with at most two decimals.
fn shown(value: &Value, name: &str) -> String {
    let number = value.get(name).and_then(Value::as_f64).unwrap_or_default();
    ((number * ROUNDING).round() / ROUNDING).to_string()
}

/// One metric that moved: `complexity 6 -> 14`, with a note where it crossed the rating bound.
fn move_text(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let numbers = [
        (ARG_METRIC, text(metric_name(speak, &field(found, METRIC))?)),
        (ARG_FROM, text(shown(found, FROM))),
        (ARG_TO, text(shown(found, TO))),
    ];
    let said = speak.say(METRICS_MOVE, &numbers)?;
    let label = field(found, CROSSING);
    if label.is_empty() {
        return Ok(said);
    }
    let crossing = if label == CROSSING_INTO_POOR {
        speak.say(METRICS_INTO_POOR, &[])?
    } else {
        speak.say(METRICS_OUT_OF_POOR, &[])?
    };
    speak.say(
        METRICS_MOVE_CROSSING,
        &[(ARG_MOVE, text(said)), (ARG_CROSSING, text(crossing))],
    )
}

/// One object with its moves, in the message `line`; `quote` shapes its name and file (a code span
/// for Markdown, as they are for the console).
fn object_as(
    speak: &Speaker<'_>,
    found: &Value,
    (line, quote): (&str, fn(&str) -> String),
) -> Result<String, I18nError> {
    let moves = entries(found, MOVES)
        .iter()
        .map(|moved| move_text(speak, moved))
        .collect::<Result<Vec<_>, _>>()?;
    speak.say(
        line,
        &[
            (arg::KIND, text(field(found, key::KIND))),
            (arg::NAME, text(quote(&field(found, key::NAME)))),
            (arg::PATH, text(quote(&field(found, key::PATH)))),
            (ARG_MOVES, text(moves.join(MOVE_JOIN))),
        ],
    )
}

/// One object of the console text.
fn object_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    object_as(speak, found, (METRICS_LINE, str::to_owned))
}

/// The heading: both scans, the level and the counts.
fn head(speak: &Speaker<'_>, answer: &Value, message: &str) -> Result<String, I18nError> {
    let scan_time = |which: &str| crate::change_text::created(&child(answer, which));
    let (base, target) = (
        child(answer, field_name::BASE),
        child(answer, field_name::TARGET),
    );
    let total = |list: &str| count(&child(answer, list), key::TOTAL);
    speak.say(
        message,
        &[
            (name::BASE, count(&base, field_name::SNAPSHOT)),
            (name::BASE_TIME, text(scan_time(field_name::BASE))),
            (name::TARGET, count(&target, field_name::SNAPSHOT)),
            (name::TARGET_TIME, text(scan_time(field_name::TARGET))),
            (ARG_LEVEL, text(field(answer, LEVEL))),
            (ARG_WORSE, total(WORSE)),
            (ARG_BETTER, total(BETTER)),
            (ARG_LOOKED, count(answer, LOOKED_AT)),
        ],
    )
}

/// `compare --metrics`: the objects that got worse and those that got better. `sides` names the
/// lists to print.
pub(crate) fn metrics_lines(
    speak: &Speaker<'_>,
    answer: &Value,
    sides: (bool, bool),
) -> Result<Vec<String>, I18nError> {
    let mut lines = vec![head(speak, answer, METRICS_HEAD)?];
    let (worse, better) = (child(answer, WORSE), child(answer, BETTER));
    if sides.0 {
        lines.extend(section(speak, (METRICS_WORSE, &worse), object_line)?);
    }
    if sides.1 {
        lines.extend(section(speak, (METRICS_BETTER, &better), object_line)?);
    }
    if lines.len() == 1 {
        lines.push(speak.say(METRICS_NONE, &[])?);
    }
    if crate::change_text::is_cut(answer) {
        lines.push(speak.say(COMPARE_CUT, &[])?);
    }
    Ok(lines)
}

/// One object as a line of Markdown: its kind, name and file in code spans and its moves.
fn markdown_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    object_as(speak, found, (METRICS_MD_LINE, code))
}

/// A list of the answer as an empty line, a heading with its count and one line for each object;
/// nothing if the list is empty.
fn markdown_side(
    speak: &Speaker<'_>,
    (heading, list): (&str, &Value),
) -> Result<Vec<String>, I18nError> {
    let mut block = section_with(speak, (heading, list), (markdown_line, NO_INDENT))?;
    if !block.is_empty() {
        block.insert(0, String::new());
    }
    Ok(block)
}

/// The Markdown of `compare --metrics --markdown`: the marker line, the counts and the two lists,
/// made from the same answer as `--json`. `sides` names the lists to print.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn metrics_markdown(
    messages: &Messages,
    language: &str,
    answer: &Value,
    sides: (bool, bool),
) -> Result<String, I18nError> {
    let speak = &Speaker { messages, language };
    let heading = head(speak, answer, METRICS_MD_TITLE)?;
    let mut lines = vec![
        METRICS_MARKDOWN_MARKER.to_owned(),
        heading,
        head(speak, answer, METRICS_MD_COUNTS)?,
    ];
    let (worse, better) = (child(answer, WORSE), child(answer, BETTER));
    let mut blocks = Vec::new();
    if sides.0 {
        blocks.push(markdown_side(speak, (METRICS_MD_WORSE, &worse))?);
    }
    if sides.1 {
        blocks.push(markdown_side(speak, (METRICS_MD_BETTER, &better))?);
    }
    if blocks.iter().all(Vec::is_empty) {
        lines.extend([String::new(), speak.say(METRICS_MD_NONE, &[])?]);
    }
    lines.extend(blocks.into_iter().flatten());
    Ok(lines.join(LINE_BREAK))
}
