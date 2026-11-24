//! Source preparation for a grammar that predates the null-conditional assignment (ADR 0094).
//!
//! C# 14 allows `a?.B = x`, `a?.B += x` and `a?[i] = x`. The C# grammar does not parse them: it
//! reads the surrounding member as something else and the rest of the file ends up outside its
//! class, so every call after it is lost. The question mark before the assigned member is
//! replaced by a space of the same width. `a .B = x` is a plain assignment that the grammar
//! knows, and no byte offset, line or column of the file moves.

use std::borrow::Cow;
use std::collections::HashSet;

use crate::consts::{
    ASSIGN_COMPOUND_SIGNS, ASSIGN_SIGN, BLANK, BRACKET_CLOSE, BRACKET_OPEN, COALESCE_ASSIGN,
    MEMBER_DOT, NAME_JOINER, NON_ASSIGN_FOLLOWERS, NULL_CONDITIONAL_MARK,
};

/// The source with the null-conditional mark of every assignment target blanked out; the source
/// itself if there is none.
pub fn prepare(source: &str) -> Cow<'_, str> {
    let marks: HashSet<usize> = source
        .match_indices(NULL_CONDITIONAL_MARK)
        .map(|(at, _)| at)
        .filter(|at| is_assignment_target(source, *at))
        .collect();
    if marks.is_empty() {
        return Cow::Borrowed(source);
    }
    Cow::Owned(
        source
            .char_indices()
            .map(|(at, c)| if marks.contains(&at) { BLANK } else { c })
            .collect(),
    )
}

/// Whether the `?` at `at` starts `?.Member` or `?[...]` that is assigned to.
fn is_assignment_target(source: &str, at: usize) -> bool {
    let Some(rest) = source.get(at + NULL_CONDITIONAL_MARK.len_utf8()..) else {
        return false;
    };
    if let Some(members) = rest.strip_prefix(MEMBER_DOT) {
        assigned_after_members(members)
    } else if rest.starts_with(BRACKET_OPEN) {
        assigned_after_brackets(rest)
    } else {
        false
    }
}

/// Whether `text` is a member name, more `.name` parts, and then an assignment sign.
fn assigned_after_members(text: &str) -> bool {
    let mut rest = text.trim_start();
    loop {
        let name_end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == NAME_JOINER))
            .unwrap_or(rest.len());
        if name_end == 0 {
            return false;
        }
        rest = rest.get(name_end..).unwrap_or_default().trim_start();
        match rest.strip_prefix(MEMBER_DOT) {
            Some(more) => rest = more.trim_start(),
            None => return starts_assignment(rest),
        }
    }
}

/// Whether `text` starts with `[...]` and an assignment sign after it.
fn assigned_after_brackets(text: &str) -> bool {
    let mut depth = 0usize;
    for (at, c) in text.char_indices() {
        if c == BRACKET_OPEN {
            depth += 1;
        } else if c == BRACKET_CLOSE {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let after = text.get(at + c.len_utf8()..).unwrap_or_default();
                return starts_assignment(after.trim_start());
            }
        }
    }
    false
}

/// Whether `text` starts with `=` (not `==` or `=>`), `??=` or a compound sign such as `+=`.
fn starts_assignment(text: &str) -> bool {
    if text.starts_with(COALESCE_ASSIGN) {
        return true;
    }
    let mut chars = text.chars();
    let after_sign = match chars.next() {
        Some(ASSIGN_SIGN) => chars,
        Some(sign) if ASSIGN_COMPOUND_SIGNS.contains(sign) => {
            if chars.next() != Some(ASSIGN_SIGN) {
                return false;
            }
            chars
        }
        _ => return false,
    };
    after_sign
        .clone()
        .next()
        .is_none_or(|next| !NON_ASSIGN_FOLLOWERS.contains(next))
}

#[cfg(test)]
#[path = "null_conditional_tests.rs"]
mod tests;
