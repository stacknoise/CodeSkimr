//! Composition root: the only place where concrete adapters are created and wired.
//!
//! `main` hands the command line and the environment to [`run::execute`]. The modules:
//! `logging`, `log_writer` and `redact` write the log with secrets masked, `locations` and
//! `config_files` find and read folders and configuration, `wiring` creates the adapters, `scan`
//! `mcp` and `serve` are the commands that need wiring.

mod clock;
mod config_files;
mod consts;
mod installation_token;
mod locations;
mod log_writer;
mod logging;
mod mcp;
#[cfg(test)]
mod memory_keystore;
mod principal;
mod redact;
mod run;
mod scan;
mod serve;
mod source_arg;
mod wiring;

use std::process::ExitCode;

fn main() -> ExitCode {
    run::execute(std::env::args_os(), &|name| std::env::var(name).ok())
}
