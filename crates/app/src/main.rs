//! Composition root: the only place where concrete adapters are created and wired.
//!
//! `main` hands the command line and the environment to [`run::execute`]. The modules:
//! `locations` and `config_files` find and read folders and configuration, `wiring` creates the
//! adapters, `scan` and `mcp` are the two commands.

mod clock;
mod config_files;
mod consts;
mod locations;
mod mcp;
mod run;
mod scan;
mod source_arg;
mod wiring;

use std::process::ExitCode;

fn main() -> ExitCode {
    run::execute(std::env::args_os(), &|name| std::env::var(name).ok())
}
