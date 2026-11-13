//! Visibility rules of a language pack, resolved from the labels in `language.toml`.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use codeskimr_domain::{SymbolKind, Visibility};

use crate::consts::KEYWORD_SEPARATOR;
use crate::manifest::VisibilityRules;

/// How keywords and the place of a declaration decide its visibility.
pub struct Visibilities {
    default: Option<Visibility>,
    keywords: HashMap<String, Visibility>,
    parent_defaults: HashMap<SymbolKind, Visibility>,
    unrated: HashSet<SymbolKind>,
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
        let unrated = rules
            .unrated_kinds
            .iter()
            .map(|kind| parse::<SymbolKind>(kind))
            .collect::<Result<_, _>>()?;
        let default = rules.default.map(|label| parse(&label)).transpose()?;
        Ok(Self {
            default,
            keywords,
            parent_defaults,
            unrated,
        })
    }

    /// Visibility of a declaration of `kind` that carries `words` (the texts of its visibility captures, in
    /// source order) inside a parent of kind `parent`.
    ///
    /// Words that are no keyword of the pack are ignored, so a query may capture every modifier.
    /// Several keywords together may have an entry of their own, such as `protected internal`;
    /// otherwise the first keyword counts. Without a keyword the default of the parent kind, then
    /// the default of the pack applies.
    #[must_use]
    pub fn of(
        &self,
        kind: SymbolKind,
        words: &[&str],
        parent: Option<SymbolKind>,
    ) -> Option<Visibility> {
        if self.unrated.contains(&kind) {
            return None;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Visibilities {
        let toml = r#"
            default = "private"
            unrated_kinds = ["namespace"]
            [keywords]
            public = "public"
            protected = "protected"
            internal = "internal"
            "protected internal" = "protected"
            "internal protected" = "protected"
            [parent_defaults]
            module = "internal"
            interface = "public"
        "#;
        Visibilities::resolve(toml::from_str(toml).expect("rules")).expect("labels")
    }

    #[test]
    fn keywords_win_and_unknown_modifiers_are_ignored() {
        let rules = rules();
        let class = SymbolKind::Class;
        assert_eq!(
            rules.of(class, &["static", "public", "sealed"], None),
            Some(Visibility::Public)
        );
        assert_eq!(
            rules.of(class, &["static"], None),
            Some(Visibility::Private)
        );
    }

    #[test]
    fn combined_keywords_match_in_either_order() {
        let rules = rules();
        let method = SymbolKind::Method;
        for words in [["protected", "internal"], ["internal", "protected"]] {
            assert_eq!(rules.of(method, &words, None), Some(Visibility::Protected));
        }
    }

    #[test]
    fn without_keyword_the_parent_then_the_pack_default_applies() {
        let rules = rules();
        let method = SymbolKind::Method;
        let module = Some(SymbolKind::Module);
        assert_eq!(rules.of(method, &[], module), Some(Visibility::Internal));
        assert_eq!(
            rules.of(method, &[], Some(SymbolKind::Interface)),
            Some(Visibility::Public)
        );
        assert_eq!(
            rules.of(method, &[], Some(SymbolKind::Class)),
            Some(Visibility::Private)
        );
        assert_eq!(rules.of(method, &[], None), Some(Visibility::Private));
    }

    #[test]
    fn unrated_kinds_have_no_visibility() {
        assert_eq!(rules().of(SymbolKind::Namespace, &["public"], None), None);
    }
}
