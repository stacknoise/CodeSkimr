//! Fixed names and texts of the benchmark check.

/// File that holds the estimates of one criterion benchmark run.
pub const ESTIMATES_FILE: &str = "estimates.json";

/// Nanoseconds in one millisecond.
pub const NS_PER_MS: f64 = 1_000_000.0;

/// Nanoseconds in one microsecond.
pub const NS_PER_US: f64 = 1_000.0;

/// Percent factor.
pub const PERCENT: f64 = 100.0;

/// Path of the shared default configuration, relative to the repository root.
pub const CONFIG_PATH: &str = "config/default.toml";

/// Path of the budget mapping, relative to the repository root.
pub const BUDGET_MAP_PATH: &str = "tools/bench/budgets.toml";

/// Table of the default configuration that holds the budgets.
pub const BUDGETS_TABLE: &str = "budgets";

/// Key of the benchmark table inside the budget mapping.
pub const BENCHMARKS_KEY: &str = "benchmarks";

/// Zero, as a floating point number.
pub const ZERO: f64 = 0.0;

/// One, as a floating point number.
pub const ONE: f64 = 1.0;

/// Path separator that criterion directories use on Windows.
pub const WINDOWS_SEPARATOR: char = '\\';

/// Separator of benchmark ids.
pub const ID_SEPARATOR: &str = "/";

/// Shown where a measurement is missing.
pub const MISSING: &str = "-";

/// Key of the regression tolerance inside the budgets table.
pub const TOLERANCE_KEY: &str = "regression_tolerance_percent";

/// Command-line flags.
pub mod flags {
    /// Directory with the criterion results.
    pub const CRITERION_DIR: &str = "--criterion-dir";
    /// Name of the baseline measured on the base commit.
    pub const BASE: &str = "--base";
    /// Name of the baseline measured on the pull request.
    pub const NEW: &str = "--new";
    /// Repository root.
    pub const ROOT: &str = "--root";
    /// The base measurement was skipped on purpose (no harness on the base commit).
    pub const NO_BASE: &str = "--no-base";
}

/// Default values of the command-line flags.
pub mod defaults {
    /// Default criterion directory.
    pub const CRITERION_DIR: &str = "target/criterion";
    /// Default base baseline name.
    pub const BASE: &str = "ci-base";
    /// Default new baseline name.
    pub const NEW: &str = "ci-head";
    /// Default repository root.
    pub const ROOT: &str = ".";
}

/// Output texts of the check.
pub mod text {
    /// Marker for a passing benchmark.
    pub const OK: &str = "ok";
    /// Marker for a benchmark slower than the tolerance.
    pub const REGRESSION: &str = "REGRESSION";
    /// Marker for a benchmark above its absolute budget.
    pub const OVER_BUDGET: &str = "OVER BUDGET";
    /// Marker for a benchmark that exists on the base but no longer in the change.
    pub const REMOVED: &str = "removed (no new measurement)";
    /// Marker for a benchmark without a base measurement.
    pub const NEW_BENCH: &str = "new (no base)";
    /// Printed when no benchmark results were found.
    pub const NOTHING_FOUND: &str = "no benchmark results found";
    /// Printed when no base measurement exists at all.
    pub const NO_BASE: &str = "NOTE: no base measurement, only absolute budgets are checked";
    /// Printed on success.
    pub const PASSED: &str = "benchmark check passed";
    /// Printed on failure.
    pub const FAILED: &str = "benchmark check FAILED";
}

/// Multiplier of the pseudo-random generator of the benchmarks (Knuth's MMIX).
pub const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
/// Increment of the pseudo-random generator of the benchmarks.
pub const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;
/// Low bits that are dropped from a generated number, because they repeat quickly.
pub const LCG_SHIFT: u32 = 33;
