//! Visibility rules of a language pack, resolved from the labels in `language.toml`.

use std::collections::HashMap;
use std::str::FromStr;

use codeskimr_domain::{SymbolKind, Visibility};

use crate::consts::KEYWORD_SEPARATOR;
use crate::manifest::VisibilityRules;

/// How keywords and the place of a declaration decide its visibility.
pub struct Visibilities {
    default: Option<Visibility>,
    keywords: HashMap<String, Visibility>,
    parent_defaults: HashMap<SymbolKind, Visibility>,
}

impl Visibilities {
    /// Resolves the labels of `rules`.
    ///
    /// # Errors
    /// The text of the first label that is not a visibility or symbol kind.
    pub fn resolve(rules: VisibilityRules) -> Result<Self, String> {
        let keywords = rules
            .keywords
            .into_iter()
            .map(|(keyword, label)| Ok((keyword, parse::<Visibility>(&label)?)))
            .collect::<Result<_, String>>()?;
        let parent_defaults = rules
            .parent_defaults
            .into_iter()
            .map(|(kind, label)| Ok((parse::<SymbolKind>(&kind)?, parse::<Visibility>(&label)?)))
            .collect::<Result<_, String>>()?;
        let default = rules.default.map(|label| parse(&label)).transpose()?;
        Ok(Self {
            default,
            keywords,
            parent_defaults,
        })
    }

    /// Visibility of a declaration that carries `words` (the texts of its visibility captures, in
    /// source order) inside a parent of kind `parent`.
    ///
    /// Words that are no keyword of the pack are ignored, so a query may capture every modifier.
    /// Several keywords together may have an entry of their own, such as `protected internal`;
    /// otherwise the first keyword counts. Without a keyword the default of the parent kind, then
    /// the default of the pack applies.
    #[must_use]
    pub fn of(&self, words: &[&str], parent: Option<SymbolKind>) -> Option<Visibility> {
        let known: Vec<&str> = words
            .iter()
            .copied()
            .filter(|word| self.keywords.contains_key(*word))
            .collect();
        self.keywords
            .get(known.join(KEYWORD_SEPARATOR).as_str())
            .or_else(|| known.first().and_then(|word| self.keywords.get(*word)))
            .or_else(|| parent.and_then(|kind| self.parent_defaults.get(&kind)))
            .or(self.default.as_ref())
            .copied()
    }
}

fn parse<T: FromStr>(label: &str) -> Result<T, String> {
    T::from_str(label).map_err(|_| label.to_owned())
}
