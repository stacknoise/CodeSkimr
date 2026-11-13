//! Registry model and loading of `errors/error-codes.toml`.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Severity of an error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A failure.
    Error,
    /// A condition that did not stop the operation.
    Warning,
    /// Informational only.
    Info,
}

impl Severity {
    /// Variant name in Rust and TypeScript output.
    #[must_use]
    pub fn variant(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }

    /// Lower-case label as written in the registry.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// Metadata of one code group (prefix).
#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    /// Human-readable title.
    pub title: String,
    /// Crate or area that owns the group.
    #[serde(rename = "crate")]
    pub owner: String,
}

/// One entry of the registry.
#[derive(Debug, Clone, Deserialize)]
pub struct Code {
    /// Id such as `SRC-201`.
    pub id: String,
    /// Snake-case name, becomes the enum variant.
    pub name: String,
    /// Severity.
    pub severity: Severity,
    /// Whether retrying the operation can succeed.
    pub retryable: bool,
    /// English developer description.
    pub description: String,
    /// First release that contains the code.
    pub since: String,
    /// Retired codes stay in the file.
    #[serde(default)]
    pub deprecated: bool,
}

/// The whole registry file.
#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    /// Groups by prefix.
    pub groups: BTreeMap<String, Group>,
    /// Codes in file order.
    pub codes: Vec<Code>,
}

impl Code {
    /// Prefix part of the id (empty when the id has no separator).
    #[must_use]
    pub fn prefix(&self) -> &str {
        self.id
            .split_once(crate::consts::ID_SEPARATOR)
            .map_or("", |(p, _)| p)
    }

    /// Number part of the id.
    #[must_use]
    pub fn number(&self) -> &str {
        self.id
            .split_once(crate::consts::ID_SEPARATOR)
            .map_or("", |(_, n)| n)
    }

    /// Class digit, the first digit of the number.
    #[must_use]
    pub fn class_digit(&self) -> u8 {
        self.number()
            .bytes()
            .next()
            .map_or(0, |b| b.saturating_sub(b'0'))
    }

    /// Enum variant name: the snake-case name in Pascal case.
    #[must_use]
    pub fn variant(&self) -> String {
        pascal_case(&self.name)
    }
}

/// Converts `snake_case` to `PascalCase`.
#[must_use]
pub fn pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

impl Registry {
    /// Parses registry text. Structural validation is a separate step.
    ///
    /// # Errors
    /// Returns the TOML parser message when the text is not a valid registry.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// Declared groups with their prefix, in order of first appearance in the code list.
    #[must_use]
    pub fn group_entries(&self) -> Vec<(&str, &Group)> {
        self.prefixes()
            .into_iter()
            .filter_map(|prefix| self.groups.get(prefix).map(|group| (prefix, group)))
            .collect()
    }

    /// Prefixes in order of first appearance in the code list.
    #[must_use]
    pub fn prefixes(&self) -> Vec<&str> {
        let mut seen: Vec<&str> = Vec::new();
        for code in &self.codes {
            if !seen.contains(&code.prefix()) {
                seen.push(code.prefix());
            }
        }
        seen
    }
}
