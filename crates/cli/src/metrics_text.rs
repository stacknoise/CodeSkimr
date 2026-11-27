//! The text of `compare --metrics` (ADR 0114), made from the same JSON answer as `--json`: which
//! objects got worse and which got better, with the metrics that moved.
//!
//! Every text from the project (names, paths) loses its control characters.

use codeskimr_codes::i18n::{
    COMPARE_CUT, METRICS_BETTER, METRICS_HEAD, METRICS_INTO_POOR, METRICS_LINE, METRICS_MOVE,
    METRICS_MOVE_CROSSING, METRICS_NAME_CC, METRICS_NAME_CE, METRICS_NAME_INSTABILITY,
    METRICS_NAME_LOC, METRICS_NAME_MI, METRICS_NONE, METRICS_OUT_OF_POOR, METRICS_WORSE,
};
use codeskimr_i18n::I18nError;
use serde_json::Value;

use crate::consts::change::{field_name, name};
use crate::consts::metrics::{
    ARG_BETTER, ARG_CROSSING, ARG_FROM, ARG_LEVEL, ARG_LOOKED, ARG_METRIC, ARG_MOVE, ARG_MOVES,
    ARG_TO, ARG_WORSE, BETTER, CROSSING, FROM, INTO_POOR, LABEL_CC, LABEL_CE, LABEL_INSTABILITY,
    LABEL_LOC, LABEL_MI, LEVEL, LOOKED_AT, METRIC, MOVE_JOIN, MOVES, ROUNDING, TO, WORSE,
};
use crate::consts::{arg, key};
use crate::speech::{Speaker, child, count, entries, field, section, text};

/// The name of a metric as text; its label if it has no name.
fn metric_name(speak: &Speaker<'_>, label: &str) -> Result<String, I18nError> {
    let known = [
        (LABEL_LOC, METRICS_NAME_LOC),
        (LABEL_CC, METRICS_NAME_CC),
        (LABEL_MI, METRICS_NAME_MI),
        (LABEL_CE, METRICS_NAME_CE),
        (LABEL_INSTABILITY, METRICS_NAME_INSTABILITY),
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
    let crossing = match field(found, CROSSING).as_str() {
        "" => return Ok(said),
        label if label == INTO_POOR => speak.say(METRICS_INTO_POOR, &[])?,
        _ => speak.say(METRICS_OUT_OF_POOR, &[])?,
    };
    speak.say(
        METRICS_MOVE_CROSSING,
        &[(ARG_MOVE, text(said)), (ARG_CROSSING, text(crossing))],
    )
}

/// One object with its moves.
fn object_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let moves = entries(found, MOVES)
        .iter()
        .map(|moved| move_text(speak, moved))
        .collect::<Result<Vec<_>, _>>()?;
    speak.say(
        METRICS_LINE,
        &[
            (arg::KIND, text(field(found, key::KIND))),
            (arg::NAME, text(field(found, key::NAME))),
            (arg::PATH, text(field(found, key::PATH))),
            (ARG_MOVES, text(moves.join(MOVE_JOIN))),
        ],
    )
}

/// The heading: both scans, the level and the counts.
fn head(speak: &Speaker<'_>, answer: &Value) -> Result<String, I18nError> {
    let scan_time = |which: &str| crate::change_text::created(&child(answer, which));
    let (base, target) = (
        child(answer, field_name::BASE),
        child(answer, field_name::TARGET),
    );
    let total = |list: &str| count(&child(answer, list), key::TOTAL);
    speak.say(
        METRICS_HEAD,
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
    let mut lines = vec![head(speak, answer)?];
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
