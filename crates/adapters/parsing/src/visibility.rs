//! Visibility rules of a language pack, resolved from the labels in `language.toml`.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use codeskimr_domain::{SymbolKind, Visibility};

use crate::consts::KEYWORD_SEPARATOR;
use crate::manifest::{NameRule, VisibilityRules};

/// How keywords and the place of a declaration decide its visibility.
pub struct Visibilities {
    default: Option<Visibility>,
    keywords: HashMap<String, Visibility>,
    parent_defaults: HashMap<SymbolKind, Visibility>,
    kind_defaults: HashMap<SymbolKind, Visibility>,
    unrated: HashSet<SymbolKind>,
    name_rules: Vec<(NameRule, Visibility)>,
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
        let kind_defaults = rules
            .kind_defaults
            .into_iter()
            .map(|(kind, label)| Ok((parse::<SymbolKind>(&kind)?, parse::<Visibility>(&label)?)))
            .collect::<Result<_, String>>()?;
        let unrated = rules
            .unrated_kinds
            .iter()
            .map(|kind| parse::<SymbolKind>(kind))
            .collect::<Result<_, _>>()?;
        let default = rules.default.map(|label| parse(&label)).transpose()?;
        let name_rules = rules
            .name_rules
            .into_iter()
            .map(|rule| {
                let visibility = parse(&rule.visibility)?;
                Ok((rule, visibility))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            default,
            keywords,
            parent_defaults,
            kind_defaults,
            unrated,
            name_rules,
        })
    }

    /// Visibility of a declaration of `kind` named `name` that carries `words` (the texts of its
    /// visibility captures, in source order) inside a parent of kind `parent`.
    ///
    /// Words that are no keyword of the pack are ignored, so a query may capture every modifier.
    /// Several keywords together may have an entry of their own, such as `protected internal`;
    /// otherwise the first keyword counts. Without a keyword the first name rule that fits the
    /// name counts, then the default of the kind, then the default of the parent kind, then the
    /// default of the pack.
    #[must_use]
    pub fn of(
        &self,
        kind: SymbolKind,
        name: &str,
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
            .or_else(|| self.by_name(name))
            .or_else(|| self.kind_defaults.get(&kind))
            .or_else(|| parent.and_then(|kind| self.parent_defaults.get(&kind)))
            .or(self.default.as_ref())
            .copied()
    }

    fn by_name(&self, name: &str) -> Option<&Visibility> {
        self.name_rules
            .iter()
            .find(|(rule, _)| {
                name.starts_with(rule.prefix.as_str())
                    && rule
                        .suffix
                        .as_deref()
                        .is_none_or(|suffix| name.ends_with(suffix))
            })
            .map(|(_, visibility)| visibility)
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
            [kind_defaults]
            enum_member = "public"
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
            rules.of(class, "a", &["static", "public", "sealed"], None),
            Some(Visibility::Public)
        );
        assert_eq!(
            rules.of(class, "a", &["static"], None),
            Some(Visibility::Private)
        );
    }

    #[test]
    fn combined_keywords_match_in_either_order() {
        let rules = rules();
        let method = SymbolKind::Method;
        for words in [["protected", "internal"], ["internal", "protected"]] {
            assert_eq!(
                rules.of(method, "a", &words, None),
                Some(Visibility::Protected)
            );
        }
    }

    #[test]
    fn without_keyword_the_parent_then_the_pack_default_applies() {
        let rules = rules();
        let method = SymbolKind::Method;
        let module = Some(SymbolKind::Module);
        assert_eq!(
            rules.of(method, "a", &[], module),
            Some(Visibility::Internal)
        );
        assert_eq!(
            rules.of(method, "a", &[], Some(SymbolKind::Interface)),
            Some(Visibility::Public)
        );
        assert_eq!(
            rules.of(method, "a", &[], Some(SymbolKind::Class)),
            Some(Visibility::Private)
        );
        assert_eq!(rules.of(method, "a", &[], None), Some(Visibility::Private));
    }

    #[test]
    fn the_default_of_the_kind_wins_over_the_one_of_the_parent() {
        let rules = rules();
        let parent = Some(SymbolKind::Enum);
        assert_eq!(
            rules.of(SymbolKind::EnumMember, "A", &[], parent),
            Some(Visibility::Public)
        );
        assert_eq!(
            rules.of(SymbolKind::Method, "a", &[], parent),
            Some(Visibility::Private)
        );
    }

    #[test]
    fn unrated_kinds_have_no_visibility() {
        assert_eq!(
            rules().of(SymbolKind::Namespace, "a", &["public"], None),
            None
        );
    }

    fn by_name_rules() -> Visibilities {
        let toml = r#"
            default = "public"
            [[name_rules]]
            prefix = "__"
            suffix = "__"
            visibility = "public"
            [[name_rules]]
            prefix = "__"
            visibility = "private"
            [[name_rules]]
            prefix = "_"
            visibility = "internal"
        "#;
        Visibilities::resolve(toml::from_str(toml).expect("rules")).expect("labels")
    }

    #[test]
    fn the_first_name_rule_that_fits_counts() {
        let rules = by_name_rules();
        let of = |name| rules.of(SymbolKind::Method, name, &[], None);
        assert_eq!(of("run"), Some(Visibility::Public));
        assert_eq!(of("_run"), Some(Visibility::Internal));
        assert_eq!(of("__run"), Some(Visibility::Private));
        assert_eq!(of("__init__"), Some(Visibility::Public));
    }

    #[test]
    fn a_keyword_wins_over_a_name_rule() {
        let toml = r#"
            [keywords]
            open = "public"
            [[name_rules]]
            prefix = "_"
            visibility = "private"
        "#;
        let rules = Visibilities::resolve(toml::from_str(toml).expect("rules")).expect("labels");
        let of = |words: &[&str]| rules.of(SymbolKind::Method, "_x", words, None);
        assert_eq!(of(&["open"]), Some(Visibility::Public));
        assert_eq!(of(&[]), Some(Visibility::Private));
    }
}
