//! The `level2` command: definition, translation of its arguments and the texts around the
//! confirmation (ADR 0033, decisions 9 and 10).

use clap::{Arg, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ARG_LEVEL2_PROJECT, CLI_CMD_LEVEL2, CLI_CMD_LEVEL2_CONFIRM, CLI_CMD_LEVEL2_REVOKE,
    CLI_LEVEL2_ANSWER_YES, CLI_LEVEL2_CONFIRMED, CLI_LEVEL2_DECLINED, CLI_LEVEL2_PROMPT,
    CLI_LEVEL2_REVOKED, CLI_LEVEL2_WARNING,
};
use codeskimr_i18n::{I18nError, Messages};

use crate::args::Level2Action;
use crate::consts::{
    ARG_LEVEL2_PROJECT, COMMAND_LEVEL2, COMMAND_LEVEL2_CONFIRM, COMMAND_LEVEL2_REVOKE,
    VALUE_EXPORT_PROJECT, arg,
};
use crate::error::CliError;

fn project_arg(help: String) -> Arg {
    Arg::new(ARG_LEVEL2_PROJECT)
        .value_name(VALUE_EXPORT_PROJECT)
        .help(help)
        .required(true)
}

/// The `level2` command with its two sub-commands.
pub fn command(say: &impl Fn(&str) -> Result<String, CliError>) -> Result<Command, CliError> {
    let confirm = Command::new(COMMAND_LEVEL2_CONFIRM)
        .about(say(CLI_CMD_LEVEL2_CONFIRM)?)
        .arg(project_arg(say(CLI_ARG_LEVEL2_PROJECT)?));
    let revoke = Command::new(COMMAND_LEVEL2_REVOKE)
        .about(say(CLI_CMD_LEVEL2_REVOKE)?)
        .arg(project_arg(say(CLI_ARG_LEVEL2_PROJECT)?));
    Ok(Command::new(COMMAND_LEVEL2)
        .about(say(CLI_CMD_LEVEL2)?)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(confirm)
        .subcommand(revoke))
}

/// Reads the sub-command that clap matched.
pub fn action(matches: &ArgMatches) -> Result<Level2Action, CliError> {
    let project = |sub: &ArgMatches| {
        sub.get_one::<String>(ARG_LEVEL2_PROJECT)
            .cloned()
            .unwrap_or_default()
    };
    match matches.subcommand() {
        Some((COMMAND_LEVEL2_CONFIRM, sub)) => Ok(Level2Action::Confirm {
            project: project(sub),
        }),
        Some((COMMAND_LEVEL2_REVOKE, sub)) => Ok(Level2Action::Revoke {
            project: project(sub),
        }),
        _ => Err(CliError::Arguments(String::new())),
    }
}

/// What the user is told before they confirm: what level 2 runs and with which rights, that the
/// indexers are not sandboxed and that the confirmation is not bound to a commit.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn confirm_warning(messages: &Messages, language: &str) -> Result<String, I18nError> {
    messages.format(language, CLI_LEVEL2_WARNING, None)
}

/// The word the user has to type, in the language of the output.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn confirm_word(messages: &Messages, language: &str) -> Result<String, I18nError> {
    messages.format(language, CLI_LEVEL2_ANSWER_YES, None)
}

fn about_project(
    messages: &Messages,
    language: &str,
    key: &str,
    project: &str,
) -> Result<String, I18nError> {
    let mut args = codeskimr_i18n::FluentArgs::new();
    args.set(arg::PROJECT, project.to_owned());
    messages.format(language, key, Some(&args))
}

/// The question that asks for the confirmation word.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn confirm_prompt(
    messages: &Messages,
    language: &str,
    project: &str,
) -> Result<String, I18nError> {
    let mut args = codeskimr_i18n::FluentArgs::new();
    args.set(arg::PROJECT, project.to_owned());
    args.set(arg::WORD, confirm_word(messages, language)?);
    messages.format(language, CLI_LEVEL2_PROMPT, Some(&args))
}

/// The line that says level 2 is allowed now.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn level2_confirmed(
    messages: &Messages,
    language: &str,
    project: &str,
) -> Result<String, I18nError> {
    about_project(messages, language, CLI_LEVEL2_CONFIRMED, project)
}

/// The line that says the user did not confirm and nothing changed.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn level2_declined(
    messages: &Messages,
    language: &str,
    project: &str,
) -> Result<String, I18nError> {
    about_project(messages, language, CLI_LEVEL2_DECLINED, project)
}

/// The line that says level 2 is forbidden again.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn level2_revoked(
    messages: &Messages,
    language: &str,
    project: &str,
) -> Result<String, I18nError> {
    about_project(messages, language, CLI_LEVEL2_REVOKED, project)
}
