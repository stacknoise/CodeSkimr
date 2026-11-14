//! Errors of the configuration (error group `CFG`, blueprint section 11).

use codeskimr_codes::{Coded, ErrorCode};

/// Which layer of the override chain a problem comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerSource {
    /// `config/default.toml`, embedded in the binary.
    Default,
    /// The user config file.
    User,
    /// `.codeskimr.toml` in the repository.
    Project,
    /// Command line flags.
    Cli,
}

/// Why a value was rejected. Typed so that the texts live in the Fluent files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    /// The value must be greater than zero.
    MustBePositive,
    /// The value must be below the named other key.
    MustBeBelow(&'static str),
    /// The value must be above the named other key.
    MustBeAbove(&'static str),
    /// The value must not be below the named other key.
    MustNotBeBelow(&'static str),
    /// The value must not exceed the named other key.
    MustNotExceed(&'static str),
    /// The value must lie between 0 and 1.
    MustBeShare,
    /// The value must lie between 0 and 100.
    MustBePercent,
    /// The list must not be empty.
    MustNotBeEmpty,
    /// The entry must be a single folder name, not a path or an empty text.
    MustBeFolderName,
    /// The text must be a URL scheme: a letter, then letters, digits, `+`, `-` or `.`.
    MustBeUrlScheme,
    /// The template must start with one of the allowed schemes.
    MustUseAllowedScheme(&'static str),
    /// The template must contain the placeholder for the file path.
    MustContainPath,
    /// A layer needs a valid name that no other layer has, and at least one valid namespace.
    MustBeLayerRule,
}

/// Errors that loading or validating the configuration can produce.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A layer is not valid TOML.
    #[error("{}", ErrorCode::ConfigParseFailed.description())]
    ParseFailed {
        /// The layer that failed.
        source_layer: LayerSource,
        /// The parser error.
        #[source]
        cause: toml::de::Error,
    },
    /// A layer sets a key that does not exist.
    #[error("{}", ErrorCode::ConfigUnknownKey.description())]
    UnknownKey {
        /// The layer that failed.
        source_layer: LayerSource,
        /// Dotted path of the unknown key.
        key: String,
    },
    /// The merged configuration has a value of the wrong type.
    #[error("{}", ErrorCode::ConfigInvalidValue.description())]
    WrongType {
        /// The deserialiser error.
        #[source]
        cause: toml::de::Error,
    },
    /// A value is outside its allowed range.
    #[error("{}", ErrorCode::ConfigInvalidValue.description())]
    OutOfRange {
        /// Dotted path of the key.
        key: &'static str,
        /// The rule that was broken.
        violation: Violation,
    },
}

impl Coded for ConfigError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::ParseFailed { .. } => ErrorCode::ConfigParseFailed,
            Self::UnknownKey { .. } => ErrorCode::ConfigUnknownKey,
            Self::WrongType { .. } | Self::OutOfRange { .. } => ErrorCode::ConfigInvalidValue,
        }
    }
}
