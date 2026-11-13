//! Fluent message bundles for the backend (blueprint section 13).
//!
//! All files under `locales/<lang>/` are embedded at build time. [`Messages::format`] renders a
//! message in the requested language and falls back to English when the language or the key is
//! missing. Keys come from `codeskimr_codes::i18n`, so an unknown key does not compile.

mod consts;
mod error;
mod messages;

pub use consts::DEFAULT_LANGUAGE;
pub use error::I18nError;
pub use fluent_bundle::{FluentArgs, FluentValue};
pub use messages::Messages;
