//! `bench-check`: compares criterion baselines and checks the budgets.

use std::process::ExitCode;

use codeskimr_bench::consts::text;
use codeskimr_bench::estimates::Medians;
use codeskimr_bench::{BenchError, args, compare::compare, config, estimates, report};

fn run() -> Result<bool, BenchError> {
    let args = args::parse(std::env::args().skip(1))?;
    let limits = config::load(&args.root)?;
    let new = estimates::collect(&args.criterion_dir, &args.new)?;
    if new.is_empty() {
        eprintln!("{}", text::NOTHING_FOUND);
        return Ok(false);
    }
    let base = if args.no_base {
        println!("{}", text::NO_BASE);
        Medians::new()
    } else {
        let base = estimates::collect(&args.criterion_dir, &args.base)?;
        if base.is_empty() {
            return Err(BenchError::NoBase);
        }
        base
    };
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
