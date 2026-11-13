//! Choosing the language of the texts from environment values such as `LANG`.

use crate::consts::{LANGUAGE_DE, LANGUAGE_EN};

/// The language for the first non-empty tag (`de_DE.UTF-8`, `en`, …); English if none matches.
#[must_use]
pub fn language_from<'a>(tags: impl IntoIterator<Item = &'a str>) -> &'static str {
    tags.into_iter()
        .map(str::trim)
        .find(|tag| !tag.is_empty())
        .map_or(LANGUAGE_EN, |tag| {
            if tag.to_ascii_lowercase().starts_with(LANGUAGE_DE) {
                LANGUAGE_DE
            } else {
                LANGUAGE_EN
            }
        })
}
