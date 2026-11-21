//! Composition root: the only place where concrete adapters are created and wired.
//!
//! `main` hands the command line and the environment to [`run::execute`]. The modules:
//! `logging`, `log_writer` and `redact` write the log with secrets masked, `locations` and
//! `config_files` find and read folders and configuration, `wiring` creates the adapters, `scan`
//! `mcp`, `serve` and `open` are the commands that need wiring, `watch` is the watch mode of `scan`.

#[cfg(test)]
mod alert_tests;
mod annotations_cli;
#[cfg(test)]
mod annotations_cli_tests;
mod browser;
mod check;
mod clock;
mod config_files;
#[cfg(test)]
mod confirm_window_http_tests;
mod confirmation_window;
mod consts;
mod cpu_time;
mod credential;
#[cfg(test)]
mod e2e;
mod embedded_packs;
mod embedded_ui;
mod export;
mod folder_dialog;
#[cfg(test)]
mod incremental_tests;
mod installation_token;
mod launch_file;
mod level2_consent;
mod level2_consent_tests;
mod level2_console;
mod level2_link_tests;
mod level2_redact_tests;
mod level2_runs;
#[cfg(test)]
mod level2_switch_tests;
mod local_http;
mod locations;
mod log_writer;
mod logging;
mod mcp;
#[cfg(test)]
mod mcp_http_tests;
#[cfg(test)]
mod memory_keystore;
#[cfg(test)]
mod oauth_stub;
mod open;
mod principal;
#[cfg(any(windows, test))]
mod private_acl;
mod redact;
mod refresh;
mod retention;
#[cfg(test)]
mod retention_tests;
mod run;
mod running;
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
mod start;
#[cfg(test)]
mod tool_check_tests;
mod tray;
#[cfg(any(windows, target_os = "macos"))]
mod tray_ui;
mod ui_commands;
mod watch;
#[cfg(windows)]
mod windows_owner;
mod wiring;

use std::process::ExitCode;

fn main() -> ExitCode {
    run::execute(std::env::args_os(), &|name| std::env::var(name).ok())
}
