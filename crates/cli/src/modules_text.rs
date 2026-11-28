//! The text of the command `modules` (ADR 0108), made from the same JSON answer as `--json`:
//! the modules with their coupling and stability, the rings with the dependencies that end
//! them, the dependencies against the layering and the modules that stand out.
//!
//! Every text from the project (names) loses its control characters.

use codeskimr_codes::i18n::{
    CLI_QUERY_MODULES_CYCLE, CLI_QUERY_MODULES_CYCLES, CLI_QUERY_MODULES_FLAG_CYCLE,
    CLI_QUERY_MODULES_FLAG_VIOLATIONS, CLI_QUERY_MODULES_HEAD, CLI_QUERY_MODULES_LAYER,
    CLI_QUERY_MODULES_LINE, CLI_QUERY_MODULES_NONE, CLI_QUERY_MODULES_PLACE,
    CLI_QUERY_MODULES_PLACES_HEAD, CLI_QUERY_MODULES_PLACES_NONE, CLI_QUERY_MODULES_RESOLVE,
    CLI_QUERY_MODULES_RISK, CLI_QUERY_MODULES_RISKS, CLI_QUERY_MODULES_STABILITY,
    CLI_QUERY_MODULES_STABILITY_NONE, CLI_QUERY_MODULES_VIOLATION, CLI_QUERY_MODULES_VIOLATIONS,
};
use codeskimr_i18n::{FluentValue, I18nError};
use serde_json::Value;

use crate::consts::{
    CHAIN_JOIN, FLAG_JOIN, FLAG_PREFIX, LAYER_PREFIX, LIST_INDENT, STABILITY_DIGITS,
    STABILITY_HALF_STEP, STABILITY_HUNDRED, STABILITY_STEP, arg, key,
};
use crate::speech::{Speaker, child, count, entries, field, json_list, plain, section, text};

/// A stability in thousandths as a number with two digits (the separator is the language's);
/// "none" for a module without a dependency.
fn stability_text(speak: &Speaker<'_>, value: &Value, name: &str) -> Result<String, I18nError> {
    let Some(thousandths) = value.get(name).and_then(Value::as_u64) else {
        return speak.say(CLI_QUERY_MODULES_STABILITY_NONE, &[]);
    };
    let rounded = (thousandths + STABILITY_HALF_STEP) / STABILITY_STEP;
    let hundredths = rounded % STABILITY_HUNDRED;
    let width = STABILITY_DIGITS;
    let shown = [
        (arg::WHOLE, FluentValue::from(rounded / STABILITY_HUNDRED)),
        (arg::HUNDREDTHS, text(format!("{hundredths:0>width$}"))),
    ];
    speak.say(CLI_QUERY_MODULES_STABILITY, &shown)
}

/// What stands out about a module: in a cycle, dependencies against the layering.
fn flags(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let mut said = Vec::new();
    if found.get(key::IN_CYCLE).and_then(Value::as_bool) == Some(true) {
        said.push(speak.say(CLI_QUERY_MODULES_FLAG_CYCLE, &[])?);
    }
    if found.get(key::VIOLATIONS).and_then(Value::as_u64) > Some(0) {
        let numbers = [(arg::COUNT, count(found, key::VIOLATIONS))];
        said.push(speak.say(CLI_QUERY_MODULES_FLAG_VIOLATIONS, &numbers)?);
    }
    if said.is_empty() {
        return Ok(String::new());
    }
    Ok([FLAG_PREFIX, said.join(FLAG_JOIN).as_str()].concat())
}

fn module_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let layer = match field(found, key::LAYER) {
        owner if owner.is_empty() => String::new(),
        owner => {
            let named = speak.say(CLI_QUERY_MODULES_LAYER, &[(arg::NAME, text(owner))])?;
            [LAYER_PREFIX, named.as_str()].concat()
        }
    };
    let numbers = [
        (arg::NAME, text(field(found, key::NAME))),
        (arg::LAYER, text(layer)),
        (arg::INCOMING, count(found, key::INCOMING)),
        (arg::OUTGOING, count(found, key::OUTGOING)),
        (
            arg::STABILITY,
            text(stability_text(speak, found, key::STABILITY)?),
        ),
        (arg::FLAGS, text(flags(speak, found)?)),
    ];
    speak.say(CLI_QUERY_MODULES_LINE, &numbers)
}

fn edge_arguments(edge: &Value) -> Vec<(&'static str, FluentValue<'static>)> {
    vec![
        (arg::FROM, text(field(edge, key::FROM))),
        (arg::TO, text(field(edge, key::TO))),
        (arg::COUNT, count(edge, key::COUNT)),
    ]
}

/// The places of one dependency between two modules, one line each.
fn places_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    let places = entries(answer, key::PLACES);
    let ends = [
        (arg::FROM, text(field(answer, key::FROM))),
        (arg::TO, text(field(answer, key::TO))),
    ];
    if places.is_empty() {
        return Ok(vec![speak.say(CLI_QUERY_MODULES_PLACES_NONE, &ends)?]);
    }
    let mut head = ends.to_vec();
    head.push((arg::SHOWN, FluentValue::from(places.len())));
    head.push((arg::TOTAL, count(answer, key::TOTAL)));
    let mut lines = vec![speak.say(CLI_QUERY_MODULES_PLACES_HEAD, &head)?];
    for place in places {
        let named = [
            (arg::PATH, text(field(place, key::PATH))),
            (arg::LINE, count(place, key::LINE)),
            (arg::FROM, text(field(place, key::FROM))),
            (arg::TO, text(field(place, key::TO))),
        ];
        lines.push(
            [
                LIST_INDENT,
                speak.say(CLI_QUERY_MODULES_PLACE, &named)?.as_str(),
            ]
            .concat(),
        );
    }
    Ok(lines)
}

/// A ring: the way round it and the dependencies to remove to end it.
fn cycle_lines(speak: &Speaker<'_>, ring: &Value) -> Result<Vec<String>, I18nError> {
    let chain: Vec<String> = entries(ring, key::MODULES)
        .iter()
        .filter_map(Value::as_str)
        .map(plain)
        .collect();
    let numbers = [
        (arg::MODULES, FluentValue::from(chain.len())),
        (arg::CHAIN, text(chain.join(CHAIN_JOIN))),
    ];
    let mut lines = vec![speak.say(CLI_QUERY_MODULES_CYCLE, &numbers)?];
    for edge in entries(ring, key::RESOLUTION) {
        let said = speak.say(CLI_QUERY_MODULES_RESOLVE, &edge_arguments(edge))?;
        lines.push([LIST_INDENT, said.as_str()].concat());
    }
    Ok(lines)
}

fn violation_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let edge = child(found, key::EDGE);
    let mut named = edge_arguments(&edge);
    named.push((arg::FROM_LAYER, text(field(found, key::FROM_LAYER))));
    named.push((arg::TO_LAYER, text(field(found, key::TO_LAYER))));
    speak.say(CLI_QUERY_MODULES_VIOLATION, &named)
}

fn risk_line(speak: &Speaker<'_>, found: &Value) -> Result<String, I18nError> {
    let numbers = [
        (arg::NAME, text(field(found, key::MODULE))),
        (
            arg::STABILITY,
            text(stability_text(speak, found, key::STABILITY)?),
        ),
        (arg::INCOMING, count(found, key::INCOMING)),
        (arg::OTHER, text(field(found, key::DEPENDS_ON))),
        (
            arg::OTHER_STABILITY,
            text(stability_text(speak, found, key::DEPENDS_ON_STABILITY)?),
        ),
    ];
    speak.say(CLI_QUERY_MODULES_RISK, &numbers)
}

/// The modules of the answer, then the rings, the dependencies against the layering and the
/// modules that stand out, each only if there are any.
pub(crate) fn modules_lines(speak: &Speaker<'_>, answer: &Value) -> Result<Vec<String>, I18nError> {
    if answer.get(key::PLACES).is_some() {
        return places_lines(speak, answer);
    }
    let modules = entries(answer, key::MODULES);
    if modules.is_empty() {
        return Ok(vec![speak.say(CLI_QUERY_MODULES_NONE, &[])?]);
    }
    let head = [
        (arg::SHOWN, FluentValue::from(modules.len())),
        (arg::TOTAL, count(answer, key::TOTAL)),
    ];
    let mut lines = vec![speak.say(CLI_QUERY_MODULES_HEAD, &head)?];
    for found in modules {
        lines.push([LIST_INDENT, module_line(speak, found)?.as_str()].concat());
    }
    let rings = entries(answer, key::CYCLES);
    if !rings.is_empty() {
        let shown = [(arg::SHOWN, FluentValue::from(rings.len()))];
        lines.push(speak.say(CLI_QUERY_MODULES_CYCLES, &shown)?);
        for ring in rings {
            lines.extend(cycle_lines(speak, ring)?);
        }
    }
    let violations = json_list(answer, key::VIOLATIONS);
    lines.extend(section(
        speak,
        (CLI_QUERY_MODULES_VIOLATIONS, &violations),
        violation_line,
    )?);
    let risks = json_list(answer, key::RISKS);
    lines.extend(section(
        speak,
        (CLI_QUERY_MODULES_RISKS, &risks),
        risk_line,
    )?);
    Ok(lines)
}
