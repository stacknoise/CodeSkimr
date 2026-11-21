//! Finding the symbol of an annotation again in a snapshot (ADR 0068, decision 3).

use std::cmp::Reverse;

use codeskimr_domain::{
    AnnotationTarget, QualifiedName, SymbolName, name_parts, signature_parameters,
};

use crate::ports::{SameName, SymbolState};

/// The symbol that the target names. Kind and qualified name decide. If several symbols share
/// them (overloads), the parameters of the target pick the one whose signature has exactly them,
/// else the first whose signature names them in order, else the only one with as many parameters
/// (a renamed parameter); none of them matching means the target is
/// gone. A single symbol is the target whatever
/// its parameters say, so a changed parameter list is a change of the signature, not a loss.
#[must_use]
pub fn pick<'a>(states: &'a [SymbolState], target: &AnnotationTarget) -> Option<&'a SymbolState> {
    let mut candidates = states
        .iter()
        .filter(|state| state.kind == target.kind && state.name == target.name);
    let first = candidates.next()?;
    if target.parameters.is_empty() || candidates.clone().next().is_none() {
        return Some(first);
    }
    let overloads: Vec<&SymbolState> = std::iter::once(first).chain(candidates).collect();
    overloads
        .iter()
        .find(|state| has_parameters(state.signature.as_deref(), &target.parameters))
        .or_else(|| {
            overloads
                .iter()
                .find(|state| names_in_order(state.signature.as_deref(), &target.parameters))
        })
        .copied()
        .or_else(|| only_with_count(&overloads, target.parameters.len()))
}

/// The one overload with `count` parameters, if there is exactly one: a renamed parameter or a
/// changed type does not make the target lost while no other overload could be meant.
fn only_with_count<'a>(overloads: &[&'a SymbolState], count: usize) -> Option<&'a SymbolState> {
    let mut same = overloads.iter().copied().filter(|state| {
        state
            .signature
            .as_deref()
            .is_some_and(|text| signature_parameters(text).len() == count)
    });
    let found = same.next()?;
    same.next().is_none().then_some(found)
}

/// Whether the parameter list of the signature is exactly `parameters`.
fn has_parameters(signature: Option<&str>, parameters: &[String]) -> bool {
    signature.is_some_and(|text| signature_parameters(text) == parameters)
}

/// Whether every parameter type stands in the signature, one after the other.
fn names_in_order(signature: Option<&str>, parameters: &[String]) -> bool {
    let mut rest = signature.unwrap_or_default();
    for parameter in parameters {
        let Some(at) = rest.find(parameter.as_str()) else {
            return false;
        };
        rest = rest.get(at + parameter.len()..).unwrap_or_default();
    }
    true
}

/// The simple name of a qualified name: its last segment.
#[must_use]
pub fn simple_name(name: &QualifiedName) -> Option<SymbolName> {
    name_parts(name.as_str())
        .last()
        .and_then(|part| SymbolName::new(*part).ok())
}

/// The symbol of the same simple name that is closest to the lost target: of the same kind first,
/// then the one whose qualified name shares the longest start with the target's, then the
/// alphabetically first.
#[must_use]
pub fn closest(target: &AnnotationTarget, same: &[SameName]) -> Option<QualifiedName> {
    same.iter()
        .max_by_key(|found| {
            (
                found.kind == target.kind,
                shared_start(found.name.as_str(), target.name.as_str()),
                Reverse(found.name.as_str()),
            )
        })
        .map(|found| found.name.clone())
}

/// How many characters two names share at their start.
fn shared_start(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(a, b)| a == b)
        .count()
}
