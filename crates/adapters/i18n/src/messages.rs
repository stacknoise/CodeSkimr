//! Fluent bundles for every embedded language.

use std::collections::BTreeMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

use crate::consts::DEFAULT_LANGUAGE;
use crate::error::I18nError;

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

type Bundle = FluentBundle<FluentResource>;

/// All languages, ready to format messages.
pub struct Messages {
    bundles: BTreeMap<String, Bundle>,
}

/// Builds the bundle of one language from its embedded files.
fn build_bundle(language: &str, files: &[(&str, &str)]) -> Result<Bundle, I18nError> {
    let invalid = |file: &str| I18nError::InvalidResource {
        language: language.to_string(),
        file: file.to_string(),
    };
    let id: LanguageIdentifier = language.parse().map_err(|_| invalid(language))?;
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // Isolation marks would end up in CLI output and MCP responses.
    bundle.set_use_isolating(false);
    for (file, text) in files {
        let resource = FluentResource::try_new((*text).to_string()).map_err(|_| invalid(file))?;
        bundle.add_resource(resource).map_err(|_| invalid(file))?;
    }
    Ok(bundle)
}

/// Renders `key` with `bundle`, or `None` when the bundle has no such message.
fn render(bundle: &Bundle, key: &str, args: Option<&FluentArgs>) -> Option<String> {
    let pattern = bundle.get_message(key)?.value()?;
    // Fluent recovers from unresolved references on its own and still returns a usable text.
    let mut recovered = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, args, &mut recovered)
            .into_owned(),
    )
}

impl Messages {
    /// Builds the bundles of all embedded languages.
    ///
    /// # Errors
    /// Returns [`I18nError::InvalidResource`] when an embedded file is invalid.
    pub fn load() -> Result<Self, I18nError> {
        let mut bundles = BTreeMap::new();
        for (language, files) in EMBEDDED {
            bundles.insert((*language).to_string(), build_bundle(language, files)?);
        }
        Ok(Self { bundles })
    }

    /// The available languages, sorted.
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.bundles.keys().map(String::as_str)
    }

    /// Renders the message `key` in `language`. An unknown language or a key missing in that
    /// language falls back to the default language (English).
    ///
    /// # Errors
    /// Returns [`I18nError::MessageKeyMissing`] when the key does not exist in the default
    /// language either.
    pub fn format(
        &self,
        language: &str,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Result<String, I18nError> {
        [language, DEFAULT_LANGUAGE]
            .into_iter()
            .filter_map(|lang| self.bundles.get(lang))
            .find_map(|bundle| render(bundle, key, args))
            .ok_or_else(|| I18nError::MessageKeyMissing {
                key: key.to_string(),
                language: language.to_string(),
            })
    }
}
