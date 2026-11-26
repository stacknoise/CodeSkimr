//! A small glob matcher for the exception lists of a project (ADR 0101): `*` stands for any run
//! of characters, also none and also a path separator, `?` for exactly one character. Everything
//! else matches itself.

use crate::consts::{GLOB_ANY, GLOB_ONE};

/// Whether `text` matches the glob `pattern` as a whole.
#[must_use]
pub fn glob_matches(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let (mut at_pattern, mut at_text) = (0, 0);
    // Where the last `*` stood and how far the text was when it was met.
    let mut resume: Option<(usize, usize)> = None;
    while let Some(&current) = text.get(at_text) {
        match pattern.get(at_pattern) {
            Some(&GLOB_ANY) => {
                resume = Some((at_pattern, at_text));
                at_pattern += 1;
            }
            Some(&sign) if sign == GLOB_ONE || sign == current => {
                at_pattern += 1;
                at_text += 1;
            }
            _ => {
                let Some((star, from)) = resume else {
                    return false;
                };
                at_pattern = star + 1;
                at_text = from + 1;
                resume = Some((star, from + 1));
            }
        }
    }
    pattern
        .get(at_pattern..)
        .is_some_and(|rest| rest.iter().all(|&sign| sign == GLOB_ANY))
}
