//! The definition of the command line with clap, and its translation into an [`Invocation`].
//!
//! Every help text comes from the Fluent files, so `--help` speaks the language of the user.

use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Arg, ArgAction, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ABOUT, CLI_ARG_BRANCH, CLI_ARG_CONFIG, CLI_ARG_DATA_DIR, CLI_ARG_EXPORT_PROJECT,
    CLI_ARG_HTML, CLI_ARG_JSON, CLI_ARG_LANGUAGES, CLI_ARG_NO_BROWSER, CLI_ARG_PATHS,
    CLI_ARG_PROJECT, CLI_ARG_SCAN_CREDENTIAL, CLI_ARG_SET, CLI_ARG_SOURCE, CLI_ARG_WATCH,
    CLI_CMD_CHECK, CLI_CMD_EXPORT, CLI_CMD_MCP, CLI_CMD_OPEN, CLI_CMD_SCAN, CLI_CMD_SERVE,
};
use codeskimr_i18n::Messages;

use crate::args::{
    Action, CheckArgs, ExportArgs, GlobalOptions, Invocation, McpArgs, OpenArgs, Parsed, ScanArgs,
    ServeArgs,
};
use crate::consent;
use crate::consts::{
    ARG_BRANCH, ARG_CONFIG, ARG_CREDENTIAL, ARG_DATA_DIR, ARG_EXPORT_PROJECT, ARG_HTML, ARG_JSON,
    ARG_LANGUAGES, ARG_NO_BROWSER, ARG_PATHS, ARG_PROJECT, ARG_SET, ARG_SOURCE, ARG_WATCH,
    BINARY_NAME, COMMAND_CHECK, COMMAND_CREDENTIAL, COMMAND_EXPORT, COMMAND_LEVEL2, COMMAND_MCP,
    COMMAND_OPEN, COMMAND_SCAN, COMMAND_SERVE, OVERRIDE_SEPARATOR, VALUE_BRANCH, VALUE_DIR,
    VALUE_EXPORT_PROJECT, VALUE_FILE, VALUE_ID, VALUE_OVERRIDE, VALUE_PATH, VALUE_SOURCE, VERSION,
};
use crate::credential;
use crate::error::CliError;

/// Reads `arguments`, the first of which is the program name.
///
/// # Errors
/// - `CLI-101` for invalid arguments; the clap message is the detail.
/// - `I18N-201` if a help text is missing.
pub fn parse<I, T>(arguments: I, messages: &Messages, language: &str) -> Result<Parsed, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    match definition(messages, language)?.try_get_matches_from(arguments) {
        Ok(matches) => invocation(&matches).map(Parsed::Run),
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                Ok(Parsed::Display(error.render().to_string()))
            }
            _ => Err(CliError::Arguments(error.render().to_string())),
        },
    }
}

fn text(messages: &Messages, language: &str, key: &str) -> Result<String, CliError> {
    Ok(messages.format(language, key, None)?)
}

fn definition(messages: &Messages, language: &str) -> Result<Command, CliError> {
    let say = |key: &str| text(messages, language, key);
    Ok(Command::new(BINARY_NAME)
        .version(VERSION)
        .about(say(CLI_ABOUT)?)
        .args(global_args(messages, language)?)
        .subcommand(scan_command(messages, language)?)
        .subcommand(check_command(messages, language)?)
        .subcommand(export_command(messages, language)?)
        .subcommand(credential::command(&say)?)
        .subcommand(consent::command(&say)?)
        .subcommand(
            Command::new(COMMAND_MCP)
                .about(say(CLI_CMD_MCP)?)
                .arg(path_arg(ARG_PROJECT, VALUE_DIR, say(CLI_ARG_PROJECT)?)),
        )
        .subcommand(Command::new(COMMAND_SERVE).about(say(CLI_CMD_SERVE)?))
        .subcommand(
            Command::new(COMMAND_OPEN).about(say(CLI_CMD_OPEN)?).arg(
                Arg::new(ARG_NO_BROWSER)
                    .long(ARG_NO_BROWSER)
                    .action(ArgAction::SetTrue)
                    .help(say(CLI_ARG_NO_BROWSER)?),
            ),
        ))
}

fn scan_command(messages: &Messages, language: &str) -> Result<Command, CliError> {
    let say = |key: &str| text(messages, language, key);
    Ok(Command::new(COMMAND_SCAN)
        .about(say(CLI_CMD_SCAN)?)
        .arg(
            Arg::new(ARG_SOURCE)
                .value_name(VALUE_SOURCE)
                .help(say(CLI_ARG_SOURCE)?)
                .required(true),
        )
        .arg(
            Arg::new(ARG_BRANCH)
                .long(ARG_BRANCH)
                .value_name(VALUE_BRANCH)
                .help(say(CLI_ARG_BRANCH)?),
        )
        .arg(
            Arg::new(ARG_JSON)
                .long(ARG_JSON)
                .action(ArgAction::SetTrue)
                .help(say(CLI_ARG_JSON)?),
        )
        .arg(
            Arg::new(ARG_WATCH)
                .long(ARG_WATCH)
                .action(ArgAction::SetTrue)
                .help(say(CLI_ARG_WATCH)?),
        )
        .arg(
            Arg::new(ARG_CREDENTIAL)
                .long(ARG_CREDENTIAL)
                .value_name(VALUE_ID)
                .help(say(CLI_ARG_SCAN_CREDENTIAL)?),
        ))
}

fn check_command(messages: &Messages, language: &str) -> Result<Command, CliError> {
    let say = |key: &str| text(messages, language, key);
    Ok(Command::new(COMMAND_CHECK)
        .about(say(CLI_CMD_CHECK)?)
        .arg(
            Arg::new(ARG_PATHS)
                .value_name(VALUE_PATH)
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..)
                .help(say(CLI_ARG_PATHS)?),
        )
        .arg(
            Arg::new(ARG_JSON)
                .long(ARG_JSON)
                .action(ArgAction::SetTrue)
                .help(say(CLI_ARG_JSON)?),
        ))
}

fn export_command(messages: &Messages, language: &str) -> Result<Command, CliError> {
    let say = |key: &str| text(messages, language, key);
    Ok(Command::new(COMMAND_EXPORT)
        .about(say(CLI_CMD_EXPORT)?)
        .arg(
            Arg::new(ARG_EXPORT_PROJECT)
                .value_name(VALUE_EXPORT_PROJECT)
                .help(say(CLI_ARG_EXPORT_PROJECT)?)
                .required(true),
        )
        .arg(path_arg(ARG_HTML, VALUE_FILE, say(CLI_ARG_HTML)?).required(true)))
}

fn global_args(messages: &Messages, language: &str) -> Result<Vec<Arg>, CliError> {
    let say = |key: &str| text(messages, language, key);
    Ok(vec![
        path_arg(ARG_DATA_DIR, VALUE_DIR, say(CLI_ARG_DATA_DIR)?).global(true),
        path_arg(ARG_LANGUAGES, VALUE_DIR, say(CLI_ARG_LANGUAGES)?).global(true),
        path_arg(ARG_CONFIG, VALUE_FILE, say(CLI_ARG_CONFIG)?).global(true),
        Arg::new(ARG_SET)
            .long(ARG_SET)
            .value_name(VALUE_OVERRIDE)
            .action(ArgAction::Append)
            .global(true)
            .help(say(CLI_ARG_SET)?),
    ])
}

fn path_arg(id: &'static str, value_name: &'static str, help: String) -> Arg {
    Arg::new(id)
        .long(id)
        .value_name(value_name)
        .value_parser(clap::value_parser!(PathBuf))
        .help(help)
}

fn invocation(matches: &ArgMatches) -> Result<Invocation, CliError> {
    let action = match matches.subcommand() {
        Some((COMMAND_SCAN, scan)) => Action::Scan(scan_args(scan)),
        Some((COMMAND_CREDENTIAL, credential)) => {
            Action::Credential(credential::action(credential)?)
        }
        Some((COMMAND_LEVEL2, level2)) => Action::Level2(consent::action(level2)?),
        Some((COMMAND_MCP, mcp)) => Action::Mcp(McpArgs {
            project: mcp.get_one::<PathBuf>(ARG_PROJECT).cloned(),
        }),
        Some((COMMAND_SERVE, _)) => Action::Serve(ServeArgs),
        Some((COMMAND_CHECK, check)) => Action::Check(check_args(check)),
        Some((COMMAND_OPEN, open)) => Action::Open(OpenArgs {
            no_browser: open.get_flag(ARG_NO_BROWSER),
        }),
        Some((COMMAND_EXPORT, export)) => Action::Export(export_args(export)),
        None => Action::Start,
        _ => return Err(CliError::Arguments(String::new())),
    };
    Ok(Invocation {
        global: global_options(matches)?,
        action,
    })
}

fn scan_args(scan: &ArgMatches) -> ScanArgs {
    ScanArgs {
        source: scan
            .get_one::<String>(ARG_SOURCE)
            .cloned()
            .unwrap_or_default(),
        branch: scan.get_one::<String>(ARG_BRANCH).cloned(),
        json: scan.get_flag(ARG_JSON),
        watch: scan.get_flag(ARG_WATCH),
        credential: scan.get_one::<String>(ARG_CREDENTIAL).cloned(),
    }
}

fn check_args(check: &ArgMatches) -> CheckArgs {
    CheckArgs {
        paths: check
            .get_many::<PathBuf>(ARG_PATHS)
            .into_iter()
            .flatten()
            .cloned()
            .collect(),
        json: check.get_flag(ARG_JSON),
    }
}

fn export_args(export: &ArgMatches) -> ExportArgs {
    ExportArgs {
        project: export
            .get_one::<String>(ARG_EXPORT_PROJECT)
            .cloned()
            .unwrap_or_default(),
        html: export
            .get_one::<PathBuf>(ARG_HTML)
            .cloned()
            .unwrap_or_default(),
    }
}

fn global_options(matches: &ArgMatches) -> Result<GlobalOptions, CliError> {
    let overrides = matches
        .get_many::<String>(ARG_SET)
        .into_iter()
        .flatten()
        .map(|pair| split_override(pair))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GlobalOptions {
        data_dir: matches.get_one::<PathBuf>(ARG_DATA_DIR).cloned(),
        languages_dir: matches.get_one::<PathBuf>(ARG_LANGUAGES).cloned(),
        config_file: matches.get_one::<PathBuf>(ARG_CONFIG).cloned(),
        overrides,
    })
}

fn split_override(pair: &str) -> Result<(String, String), CliError> {
    pair.split_once(OVERRIDE_SEPARATOR)
        .filter(|(key, _)| !key.trim().is_empty())
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .ok_or_else(|| CliError::Arguments(pair.to_owned()))
}
