//! Level 2 references: runs the indexer of a language as a guarded child process (ADR 0033).
//!
//! The adapter implements the `IndexRunner` port. It does not decide whether an indexer may run;
//! the caller has passed `codeskimr_application::level2::authorize_run` before. What it does
//! guarantee: the program comes from the `PATH` of the user and never from the project, the
//! environment is built from an allow list, the process runs in its own process group (a job
//! object on Windows) that is killed on timeout, limit and cancel, its output goes to a temporary
//! folder inside the data directory, and what it printed is redacted before it is logged. It
//! never touches the network itself and never runs `npx` or downloads anything.

mod consts;
mod env;
mod error;
mod memory;
mod process;
mod redact;
mod runner;
mod tools;

pub use env::{Environment, build as build_environment};
pub use error::{FailureReason, IndexerError};
pub use redact::Redact;
pub use runner::ProcessIndexRunner;
