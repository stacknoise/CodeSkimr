//! Versions of external tools, read from their `--version` output.

use std::fmt;

use crate::consts::{VERSION_DECORATION, VERSION_SEPARATOR, VERSION_WORD};

/// A version of up to three numbers. Missing numbers are zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ToolVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl ToolVersion {
    /// Reads `text` such as `1.97.0`, `v20.11` or `17.0.9_2`: numbers separated by dots, each
    /// number counted up to its first non-digit; anything after the third number is ignored. It
    /// needs at least one dot, so that a date or an id is not taken for a version.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::read(text, true)
    }

    /// The first version in the output of a tool. Every word is tried in turn; failing that, the
    /// word after `version` may be a single number, as in `openjdk version "21"`, which the JDK
    /// prints for the first release of a line (`17-ea` and `21` too).
    #[must_use]
    pub fn find_in(output: &str) -> Option<Self> {
        output.split_whitespace().find_map(Self::parse).or_else(|| {
            let mut words = output.split_whitespace();
            words.find(|word| word.eq_ignore_ascii_case(VERSION_WORD))?;
            Self::read(words.next()?, false)
        })
    }

    fn read(text: &str, needs_separator: bool) -> Option<Self> {
        let trimmed = text.trim_matches(VERSION_DECORATION);
        if needs_separator && !trimmed.contains(VERSION_SEPARATOR) {
            return None;
        }
        let mut numbers = trimmed.split(VERSION_SEPARATOR).map(leading_number);
        let major = numbers.next()??;
        let minor = numbers.next().flatten().unwrap_or_default();
        let patch = numbers.next().flatten().unwrap_or_default();
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

fn leading_number(part: &str) -> Option<u32> {
    let end = part
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(part.len());
    part.get(..end)?.parse().ok()
}

impl fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = VERSION_SEPARATOR;
        write!(f, "{}{sep}{}{sep}{}", self.major, self.minor, self.patch)
    }
}
