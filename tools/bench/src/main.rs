//! `bench-check`: compares criterion baselines and checks the budgets.

use std::path::PathBuf;
use std::process::ExitCode;

use codeskimr_bench::consts::{defaults, flags, text};
use codeskimr_bench::{BenchError, compare::compare, config, estimates, report};

struct Args {
    criterion_dir: PathBuf,
    base: String,
    new: String,
    root: PathBuf,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, BenchError> {
    let mut parsed = Args {
        criterion_dir: PathBuf::from(defaults::CRITERION_DIR),
        base: defaults::BASE.to_owned(),
        new: defaults::NEW.to_owned(),
        root: PathBuf::from(defaults::ROOT),
    };
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| BenchError::MissingFlagValue(flag.clone()))?;
        match flag.as_str() {
            flags::CRITERION_DIR => parsed.criterion_dir = PathBuf::from(value),
            flags::BASE => parsed.base = value,
            flags::NEW => parsed.new = value,
            flags::ROOT => parsed.root = PathBuf::from(value),
            _ => return Err(BenchError::MissingFlagValue(flag)),
        }
    }
    Ok(parsed)
}

fn run() -> Result<bool, BenchError> {
    let args = parse(std::env::args().skip(1))?;
    let limits = config::load(&args.root)?;
    let new = estimates::collect(&args.criterion_dir, &args.new)?;
    if new.is_empty() {
        eprintln!("{}", text::NOTHING_FOUND);
        return Ok(false);
    }
    let base = estimates::collect(&args.criterion_dir, &args.base).unwrap_or_default();
    if base.is_empty() {
        println!("{}", text::NO_BASE);
    }
    let rows = compare(&new, &base, &limits.budgets_ms, limits.tolerance_percent);
    println!("{}", report::render(&rows));
    Ok(!report::has_failure(&rows))
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => {
            println!("{}", text::PASSED);
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("{}", text::FAILED);
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
