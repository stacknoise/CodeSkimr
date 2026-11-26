//! Writes the exception list of the unused symbols into the text of a project file (ADR 0101):
//! everything but the `[[unused.ignore]]` tables stays as it is, the tables are written anew at
//! the end.

use serde::Serialize;

use super::consts::{UNUSED_TABLE_HEADER, keys};
use super::error::{ConfigError, Violation};
use super::layer_file::rewrite;
use super::model::UnusedIgnore;

/// The shape of the written tables: `[[unused.ignore]]`.
#[derive(Serialize)]
struct Written<'a> {
    unused: Entries<'a>,
}

#[derive(Serialize)]
struct Entries<'a> {
    ignore: &'a [UnusedIgnore],
}

/// The entries of the exception list in `text`; none if the file has none or cannot be read.
#[must_use]
pub fn unused_ignore_in(text: &str) -> Vec<UnusedIgnore> {
    text.parse::<toml::Table>()
        .ok()
        .and_then(|mut table| table.remove(keys::UNUSED))
        .and_then(|value| value.try_into::<toml::Table>().ok())
        .and_then(|mut unused| unused.remove(keys::IGNORE))
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

/// The text of a project file with `wanted` as its exception list, in that order.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if an entry has no reason, no pattern, or an empty pattern.
pub fn with_unused_ignore(text: &str, wanted: &[UnusedIgnore]) -> Result<String, ConfigError> {
    let block = if wanted.is_empty() {
        String::new()
    } else {
        toml::to_string(&Written {
            unused: Entries { ignore: wanted },
        })
        .map_err(|_| ConfigError::OutOfRange {
            key: keys::UNUSED_IGNORE,
            violation: Violation::MustBeUnusedIgnore,
        })?
    };
    rewrite(text, UNUSED_TABLE_HEADER, &block)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(symbol: &str, reason: &str) -> UnusedIgnore {
        UnusedIgnore {
            paths: Vec::new(),
            symbols: vec![symbol.to_owned()],
            reason: reason.to_owned(),
        }
    }

    #[test]
    fn entries_are_written_at_the_end_and_read_back() {
        let text = "[keys]\nmax_files = 100\n";
        let wanted = vec![entry("*.Hidden", "called by reflection")];
        let written = with_unused_ignore(text, &wanted).unwrap();
        assert!(written.starts_with("[keys]\nmax_files = 100"));
        assert!(written.contains("[[unused.ignore]]"));
        assert_eq!(unused_ignore_in(&written), wanted);
    }

    #[test]
    fn the_other_tables_stay_and_removing_the_last_entry_leaves_none() {
        let wanted = vec![entry("*.A", "one"), entry("*.B", "two")];
        let both = with_unused_ignore("[keys]\nmax_files = 100\n", &wanted).unwrap();
        let kept = with_unused_ignore(&both, &wanted[1..]).unwrap();
        assert_eq!(unused_ignore_in(&kept), wanted[1..]);
        let none = with_unused_ignore(&kept, &[]).unwrap();
        assert_eq!(none, "[keys]\nmax_files = 100\n");
    }

    #[test]
    fn an_entry_without_a_reason_or_a_file_that_is_no_toml_is_refused() {
        assert!(with_unused_ignore("", &[entry("*.A", " ")]).is_err());
        assert!(with_unused_ignore("[[unused.ignore]\n", &[]).is_err());
    }

    #[test]
    fn windows_line_ends_stay_windows_line_ends() {
        let written = with_unused_ignore("[keys]\r\nmax_files = 100\r\n", &[entry("*.A", "x")]);
        assert!(written.unwrap().contains("\r\n[[unused.ignore]]"));
    }
}
