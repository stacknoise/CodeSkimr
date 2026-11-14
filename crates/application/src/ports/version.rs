//! Versions of external tools, read from their `--version` output.

use std::fmt;

use crate::consts::{
    DATE_MIN_DIGITS, DATE_SEPARATORS, VERSION_DECORATION, VERSION_SEPARATOR, VERSION_WORD,
};

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
            let word = words.next()?;
            (!looks_like_date(word)).then(|| Self::read(word, false))?
        })
    }

    /// The version a project requires, from the text of a file such as `.nvmrc` or `global.json`.
    /// Unlike [`Self::parse`] a single number is enough (`20`). `None` if the text does not start
    /// with a number (`stable`, `lts/*`, `pypy3.10`, `>=8`): that asks for nothing comparable.
    #[must_use]
    pub fn parse_requirement(text: &str) -> Option<Self> {
        let trimmed = text.trim().trim_start_matches(VERSION_DECORATION);
        if !trimmed.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        Self::read(trimmed, false)
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

/// Whether a single word is a date or an id: digits, then a date separator and another digit, or
/// a very long number. The release of a JDK line (`17-ea`) is not.
fn looks_like_date(word: &str) -> bool {
    let trimmed = word.trim_matches(VERSION_DECORATION);
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    let rest = trimmed.get(digits..).unwrap_or_default();
    let after_separator = rest
        .strip_prefix(DATE_SEPARATORS)
        .and_then(|tail| tail.chars().next());
    digits >= DATE_MIN_DIGITS || after_separator.is_some_and(|next| next.is_ascii_digit())
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
