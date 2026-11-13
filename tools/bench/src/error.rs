//! Errors of the benchmark check.

use std::path::PathBuf;

/// Everything that can go wrong while reading results or configuration.
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    /// A file or directory could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// A criterion estimates file is not valid JSON of the expected shape.
    #[error("invalid estimates in {path}: {source}")]
    Estimates {
        /// File that failed.
        path: PathBuf,
        /// Underlying error.
        source: serde_json::Error,
    },
    /// A TOML file is invalid.
    #[error("invalid TOML in {path}: {source}")]
    Toml {
        /// File that failed.
        path: PathBuf,
        /// Underlying error.
        source: toml::de::Error,
    },
    /// A required configuration value is missing.
    #[error("missing configuration value {0}")]
    MissingConfig(String),
    /// A budget key from the mapping does not exist in the configuration.
    #[error("unknown budget key {0}")]
    UnknownBudget(String),
    /// A budget value is negative, too large or not a number.
    #[error("invalid value for budget key {0}")]
    InvalidBudget(String),
    /// A command-line flag is not known.
    #[error("unknown flag {0}")]
    UnknownFlag(String),
    /// The base measurement is missing although it was not skipped on purpose.
    #[error("no base measurement found; pass --no-base when the base has no harness")]
    NoBase,
    /// A command-line flag has no value.
    #[error("flag {0} needs a value")]
    MissingFlagValue(String),
}
