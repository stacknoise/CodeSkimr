//! Composition root: the only place where concrete adapters are created and wired.
//!
//! `main` hands the command line and the environment to [`run::execute`]. The modules:
//! `logging`, `log_writer` and `redact` write the log with secrets masked, `locations` and
//! `config_files` find and read folders and configuration, `wiring` creates the adapters, `scan`
//! `mcp`, `serve` and `open` are the commands that need wiring, `watch` is the watch mode of `scan`.

mod browser;
mod check;
mod clock;
mod config_files;
mod consts;
mod credential;
#[cfg(test)]
mod e2e;
mod embedded_ui;
mod export;
mod installation_token;
mod launch_file;
mod level2_consent;
mod level2_consent_tests;
mod level2_link_tests;
mod level2_redact_tests;
mod local_http;
mod locations;
mod log_writer;
mod logging;
mod mcp;
#[cfg(test)]
mod mcp_http_tests;
#[cfg(test)]
mod memory_keystore;
mod open;
mod principal;
mod redact;
mod run;
mod scan;
#[cfg(test)]
mod security_http_tests;
#[cfg(test)]
mod security_login_tests;
#[cfg(test)]
mod security_tests;
mod serve;
mod shared_keystore;
mod source_arg;
mod ui_commands;
mod watch;
mod wiring;

use std::process::ExitCode;

fn main() -> ExitCode {
    run::execute(std::env::args_os(), &|name| std::env::var(name).ok())
}
