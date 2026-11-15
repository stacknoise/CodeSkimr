//! Command-line arguments of `bench-check`.

use std::path::PathBuf;

use crate::consts::{defaults, flags};
use crate::error::BenchError;

/// Parsed arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    /// Directories with the criterion results, one per run of the same benchmarks. With more than
    /// one, the best (smallest) median of each benchmark and side counts, so that one slow run
    /// of a shared machine cannot fail the check alone.
    pub criterion_dirs: Vec<PathBuf>,
    /// Name of the baseline measured on the base commit.
    pub base: String,
    /// Name of the baseline measured on the change.
    pub new: String,
    /// Repository root.
    pub root: PathBuf,
    /// True when the base measurement was skipped on purpose.
    pub no_base: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            criterion_dirs: vec![PathBuf::from(defaults::CRITERION_DIR)],
            base: defaults::BASE.to_owned(),
            new: defaults::NEW.to_owned(),
            root: PathBuf::from(defaults::ROOT),
            no_base: false,
        }
    }
}

/// Parses the arguments after the program name.
///
/// # Errors
/// Returns an error for an unknown flag or a flag without a value.
pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, BenchError> {
    let mut parsed = Args {
        // The flag may be given several times; the default only stands in when it is not given.
        criterion_dirs: Vec::new(),
        ..Args::default()
    };
    while let Some(flag) = args.next() {
        if flag == flags::NO_BASE {
            parsed.no_base = true;
            continue;
        }
        let known = [flags::CRITERION_DIR, flags::BASE, flags::NEW, flags::ROOT];
        if !known.contains(&flag.as_str()) {
            return Err(BenchError::UnknownFlag(flag));
        }
        let value = args
            .next()
            .ok_or_else(|| BenchError::MissingFlagValue(flag.clone()))?;
        assign(&mut parsed, &flag, value);
    }
    if parsed.criterion_dirs.is_empty() {
        parsed.criterion_dirs = Args::default().criterion_dirs;
    }
    Ok(parsed)
}

fn assign(parsed: &mut Args, flag: &str, value: String) {
    match flag {
        flags::CRITERION_DIR => parsed.criterion_dirs.push(PathBuf::from(value)),
        flags::BASE => parsed.base = value,
        flags::NEW => parsed.new = value,
        _ => parsed.root = PathBuf::from(value),
    }
}
