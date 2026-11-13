//! Benchmark harness of the product (blueprint 16.1 and 17.1).
//!
//! The criterion benchmarks live in `benches/`. After they ran for the base commit and for the
//! pull request (`--save-baseline ci-base` and `--save-baseline ci-head`), the `bench-check` binary
//! compares both measurements and the absolute budgets and fails on a regression.

pub mod args;
pub mod compare;
pub mod config;
pub mod consts;
pub mod error;
pub mod estimates;
pub mod random;
pub mod report;

pub use error::BenchError;
