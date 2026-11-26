//! A small glob matcher for the exception lists of a project (ADR 0101): `*` stands for any run
//! of characters, also none and also a path separator, `?` for exactly one character. Everything
//! else matches itself.

use std::str::Chars;

use crate::consts::{GLOB_ANY, GLOB_ONE};

/// Whether `text` matches the glob `pattern` as a whole. It allocates nothing: the exception list
/// is asked once per symbol, which is millions of times for a large project.
#[must_use]
pub fn glob_matches(pattern: &str, text: &str) -> bool {
    let (mut rest_pattern, mut rest_text) = (pattern.chars(), text.chars());
    // Where the last `*` stood (the pattern after it) and the text it has swallowed so far.
    let mut resume: Option<(Chars<'_>, Chars<'_>)> = None;
    loop {
        match (rest_pattern.clone().next(), rest_text.clone().next()) {
            (Some(GLOB_ANY), _) => {
                rest_pattern.next();
                resume = Some((rest_pattern.clone(), rest_text.clone()));
            }
            (Some(sign), Some(current)) if sign == GLOB_ONE || sign == current => {
                rest_pattern.next();
                rest_text.next();
            }
            (None, None) => return true,
            _ => {
                let Some((after_star, mut swallowed)) = resume.take() else {
                    return false;
                };
                if swallowed.next().is_none() {
                    return false;
                }
                (rest_pattern, rest_text) = (after_star.clone(), swallowed.clone());
                resume = Some((after_star, swallowed));
            }
        }
    }
}
