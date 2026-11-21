//! The output of `codeskimr codes`: the list of the error codes, or the explanation of one.

use clap::{Arg, ArgMatches, Command};
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_ARG_CODE, CLI_CMD_CODES, CLI_CODES_EXPLAIN, CLI_CODES_LINE, CLI_CODES_NOT_FOUND,
    CLI_CODES_RETIRED,
};
use codeskimr_i18n::{FluentArgs, I18nError, Messages};

use crate::args::CodesArgs;
use crate::consts::{ARG_CODE, COMMAND_CODES, LINE_BREAK, SELECT_NO, SELECT_YES, VALUE_CODE, arg};
use crate::error::CliError;

/// The `codes` command with its optional code.
pub fn command(say: &impl Fn(&str) -> Result<String, CliError>) -> Result<Command, CliError> {
    let code = Arg::new(ARG_CODE)
        .value_name(VALUE_CODE)
        .help(say(CLI_ARG_CODE)?);
    Ok(Command::new(COMMAND_CODES)
        .about(say(CLI_CMD_CODES)?)
        .arg(code))
}

/// The arguments of `codes` as the request that runs it.
pub fn args(matches: &ArgMatches) -> CodesArgs {
    CodesArgs {
        code: matches.get_one::<String>(ARG_CODE).cloned(),
    }
}

/// Every registered code with its message in `language`, one per line, or the explanation of the
/// code that was asked for.
///
/// # Errors
/// - `CLI-101` if the id names no registered code; the text says so.
/// - `I18N-201` if a message is missing.
pub fn codes_text(
    messages: &Messages,
    language: &str,
    args: &CodesArgs,
) -> Result<String, CliError> {
    match &args.code {
        None => Ok(list(messages, language)?),
        Some(id) => match ErrorCode::from_id(&id.trim().to_ascii_uppercase()) {
            Some(code) => Ok(explanation(messages, language, code)?),
            None => Err(CliError::Arguments(not_found(messages, language, id)?)),
        },
    }
}

fn list(messages: &Messages, language: &str) -> Result<String, I18nError> {
    let lines = ErrorCode::ALL
        .iter()
        .filter(|code| !code.entry().deprecated)
        .map(|code| line(messages, language, *code))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lines.join(LINE_BREAK))
}

fn line(messages: &Messages, language: &str, code: ErrorCode) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::CODE, code.id());
    args.set(arg::MESSAGE, message(messages, language, code)?);
    messages.format(language, CLI_CODES_LINE, Some(&args))
}

fn explanation(messages: &Messages, language: &str, code: ErrorCode) -> Result<String, I18nError> {
    let entry = code.entry();
    let mut args = FluentArgs::new();
    args.set(arg::CODE, code.id());
    args.set(arg::MESSAGE, message(messages, language, code)?);
    args.set(arg::DESCRIPTION, code.description());
    args.set(arg::GROUP, entry.group.prefix());
    args.set(arg::SEVERITY, entry.severity.label());
    args.set(arg::RETRYABLE, yes_no(code.retryable()));
    args.set(arg::SINCE, entry.since);
    let text = messages.format(language, CLI_CODES_EXPLAIN, Some(&args))?;
    if !entry.deprecated {
        return Ok(text);
    }
    let retired = messages.format(language, CLI_CODES_RETIRED, None)?;
    Ok([text, retired].join(LINE_BREAK))
}

fn message(messages: &Messages, language: &str, code: ErrorCode) -> Result<String, I18nError> {
    messages.format(language, code.message_key(), None)
}

fn not_found(messages: &Messages, language: &str, id: &str) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::ID, id);
    messages.format(language, CLI_CODES_NOT_FOUND, Some(&args))
}

const fn yes_no(value: bool) -> &'static str {
    if value { SELECT_YES } else { SELECT_NO }
}
