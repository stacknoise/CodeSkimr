//! The `credential` command: definition and translation of its arguments.

use clap::{Arg, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ARG_CLIENT_ID, CLI_ARG_HOST, CLI_ARG_ID, CLI_ARG_PROVIDER, CLI_ARG_USER,
    CLI_CMD_CREDENTIAL, CLI_CMD_CREDENTIAL_DELETE, CLI_CMD_CREDENTIAL_LOGIN,
    CLI_CMD_CREDENTIAL_SET,
};

use crate::args::{CredentialAction, ProviderChoice};
use crate::consts::{
    ARG_CLIENT_ID, ARG_HOST, ARG_ID, ARG_PROVIDER, ARG_USER, COMMAND_CREDENTIAL,
    COMMAND_CREDENTIAL_DELETE, COMMAND_CREDENTIAL_LOGIN, COMMAND_CREDENTIAL_SET, PROVIDER_GITHUB,
    PROVIDER_GITLAB, VALUE_CLIENT_ID, VALUE_HOST, VALUE_ID, VALUE_PROVIDER, VALUE_USER,
};
use crate::error::CliError;

fn id_arg(help: String) -> Arg {
    Arg::new(ARG_ID)
        .value_name(VALUE_ID)
        .help(help)
        .required(true)
}

fn named(id: &'static str, value_name: &'static str, help: String) -> Arg {
    Arg::new(id).long(id).value_name(value_name).help(help)
}

/// The `credential` command with its three sub-commands.
pub fn command(say: &impl Fn(&str) -> Result<String, CliError>) -> Result<Command, CliError> {
    let set = Command::new(COMMAND_CREDENTIAL_SET)
        .about(say(CLI_CMD_CREDENTIAL_SET)?)
        .arg(id_arg(say(CLI_ARG_ID)?))
        .arg(named(ARG_USER, VALUE_USER, say(CLI_ARG_USER)?).required(true));
    let delete = Command::new(COMMAND_CREDENTIAL_DELETE)
        .about(say(CLI_CMD_CREDENTIAL_DELETE)?)
        .arg(id_arg(say(CLI_ARG_ID)?));
    let login = Command::new(COMMAND_CREDENTIAL_LOGIN)
        .about(say(CLI_CMD_CREDENTIAL_LOGIN)?)
        .arg(id_arg(say(CLI_ARG_ID)?))
        .arg(
            named(ARG_PROVIDER, VALUE_PROVIDER, say(CLI_ARG_PROVIDER)?)
                .value_parser([PROVIDER_GITHUB, PROVIDER_GITLAB])
                .required(true),
        )
        .arg(named(ARG_CLIENT_ID, VALUE_CLIENT_ID, say(CLI_ARG_CLIENT_ID)?).required(true))
        .arg(named(ARG_HOST, VALUE_HOST, say(CLI_ARG_HOST)?));
    Ok(Command::new(COMMAND_CREDENTIAL)
        .about(say(CLI_CMD_CREDENTIAL)?)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(set)
        .subcommand(delete)
        .subcommand(login))
}

fn text(matches: &ArgMatches, name: &str) -> String {
    matches.get_one::<String>(name).cloned().unwrap_or_default()
}

/// What the matched sub-command asks for.
pub fn action(matches: &ArgMatches) -> Result<CredentialAction, CliError> {
    match matches.subcommand() {
        Some((COMMAND_CREDENTIAL_SET, set)) => Ok(CredentialAction::Set {
            id: text(set, ARG_ID),
            user: text(set, ARG_USER),
        }),
        Some((COMMAND_CREDENTIAL_DELETE, delete)) => Ok(CredentialAction::Delete {
            id: text(delete, ARG_ID),
        }),
        Some((COMMAND_CREDENTIAL_LOGIN, login)) => Ok(CredentialAction::Login {
            id: text(login, ARG_ID),
            provider: if text(login, ARG_PROVIDER) == PROVIDER_GITLAB {
                ProviderChoice::GitLab
            } else {
                ProviderChoice::GitHub
            },
            client_id: text(login, ARG_CLIENT_ID),
            host: login.get_one::<String>(ARG_HOST).cloned(),
        }),
        _ => Err(CliError::Arguments(String::new())),
    }
}
