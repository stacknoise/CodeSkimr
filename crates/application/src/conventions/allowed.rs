//! The reasoned exception from `CHK-102` for pure mapping functions (ADR 0060).

use codeskimr_domain::FolderPath;

use super::length::Callable;
use crate::consts::{
    CHECK_ALLOW_MARKER, CHECK_MARKER_LOOKBACK, CHECK_MARKER_MAX_FILE_BYTES, CHECK_MARKER_MIN_REASON,
};
use crate::ports::SourceText;

/// Whether `callable` is a pure mapping function that says so: its cognitive complexity is 0 (no
/// branch, no loop, so it only moves values from one shape into another) and the comment block
/// directly above its declaration holds the marker with a reason. A function whose language pack
/// does not count the cognitive complexity is never exempt, because the claim cannot be checked.
/// A file that cannot be read gives no exemption.
pub(super) fn is_allowed(text: &dyn SourceText, root: &FolderPath, callable: &Callable) -> bool {
    if callable.cognitive != Some(0) {
        return false;
    }
    let declaration = callable.location.span.start().line.saturating_add(1);
    let first = declaration.saturating_sub(CHECK_MARKER_LOOKBACK).max(1);
    let Ok(excerpt) = text.excerpt(
        root,
        &callable.location.path,
        (first, declaration),
        (
            CHECK_MARKER_LOOKBACK.saturating_add(1),
            CHECK_MARKER_MAX_FILE_BYTES,
        ),
    ) else {
        return false;
    };
    excerpt
        .lines
        .iter()
        .rev()
        .take_while(|line| !line.trim().is_empty())
        .any(|line| has_reason(line))
}

/// Whether the line holds the marker followed by a reason.
fn has_reason(line: &str) -> bool {
    line.split_once(CHECK_ALLOW_MARKER)
        .is_some_and(|(_, reason)| {
            reason.chars().filter(|c| c.is_alphanumeric()).count() >= CHECK_MARKER_MIN_REASON
        })
}
