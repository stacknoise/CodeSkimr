//! Dotted numeric versions, as printed by tools and as projects require them.

use std::cmp::Ordering;
use std::fmt;

use crate::consts::{VERSION_PREFIXES, VERSION_SEPARATOR};

/// A version such as `8.0.404`: numbers separated by dots. A missing number counts as zero, so
/// `20` equals `20.0.0`.
#[derive(Debug, Clone)]
pub struct ToolVersion {
    text: String,
    segments: Vec<u64>,
}

fn is_version_char(c: char) -> bool {
    c.is_ascii_digit() || c == VERSION_SEPARATOR
}

fn numbers(text: &str) -> Option<Vec<u64>> {
    text.split(VERSION_SEPARATOR)
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

impl ToolVersion {
    fn from_text(text: &str) -> Option<Self> {
        Some(Self {
            text: text.to_owned(),
            segments: numbers(text)?,
        })
    }

    /// The first version in the output of a tool: the first run of numbers with at least one
    /// dot between them (`Python 3.12.1` gives `3.12.1`, `v20.11.1` gives `20.11.1`).
    #[must_use]
    pub fn find_in(output: &str) -> Option<Self> {
        output
            .split(|c: char| !is_version_char(c))
            .map(|token| token.trim_matches(VERSION_SEPARATOR))
            .filter(|token| token.contains(VERSION_SEPARATOR))
            .find_map(Self::from_text)
    }

    /// The version a project requires, from the text of a file such as `.nvmrc` or
    /// `global.json`: the leading numbers of the text, after an optional `v`. `None` if the text
    /// does not start with a number (`stable`, `lts/*`, `pypy3.10`), so it asks for nothing
    /// that could be compared.
    #[must_use]
    pub fn parse_requirement(text: &str) -> Option<Self> {
        let trimmed = text.trim().trim_start_matches(VERSION_PREFIXES);
        let end = trimmed
            .find(|c: char| !is_version_char(c))
            .unwrap_or(trimmed.len());
        let prefix = trimmed.get(..end)?.trim_end_matches(VERSION_SEPARATOR);
        Self::from_text(prefix)
    }

    /// The number at `index`, zero beyond the end.
    fn segment(&self, index: usize) -> u64 {
        self.segments.get(index).copied().unwrap_or_default()
    }
}

impl Ord for ToolVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let length = self.segments.len().max(other.segments.len());
        (0..length)
            .map(|index| self.segment(index).cmp(&other.segment(index)))
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ToolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ToolVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for ToolVersion {}

impl fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}
