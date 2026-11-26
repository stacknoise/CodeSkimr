//! The command `scope` (ADR 0101): its definition with clap and the translation of its arguments.
//! What runs behind it are the commands `scope.get` and `scope.set` of the registry.

use clap::{Arg, ArgAction, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ARG_JSON, CLI_ARG_SCOPE_PROJECT, CLI_ARG_SCOPE_RESET, CLI_ARG_WITH_FOLDER,
    CLI_ARG_WITH_LANGUAGE, CLI_ARG_WITHOUT_FOLDER, CLI_ARG_WITHOUT_LANGUAGE, CLI_CMD_SCOPE,
};

use crate::args::ScopeArgs;
use crate::consts::{
    ARG_JSON, ARG_RESET, ARG_SCOPE_PROJECT, ARG_WITH_FOLDER, ARG_WITH_LANGUAGE, ARG_WITHOUT_FOLDER,
    ARG_WITHOUT_LANGUAGE, COMMAND_SCOPE, VALUE_EXPORT_PROJECT, VALUE_FOLDER, VALUE_LANGUAGE,
};
use crate::error::CliError;

fn repeated(id: &'static str, value: &'static str, help: String) -> Arg {
    Arg::new(id)
        .long(id)
        .value_name(value)
        .action(ArgAction::Append)
        .help(help)
}

fn flag(id: &'static str, help: String) -> Arg {
    Arg::new(id).long(id).action(ArgAction::SetTrue).help(help)
}

/// The `scope` command.
pub fn command(say: &impl Fn(&str) -> Result<String, CliError>) -> Result<Command, CliError> {
    let project = Arg::new(ARG_SCOPE_PROJECT)
        .value_name(VALUE_EXPORT_PROJECT)
        .help(say(CLI_ARG_SCOPE_PROJECT)?)
        .required(true);
    Ok(Command::new(COMMAND_SCOPE)
        .about(say(CLI_CMD_SCOPE)?)
        .arg(project)
        .arg(repeated(
            ARG_WITHOUT_LANGUAGE,
            VALUE_LANGUAGE,
            say(CLI_ARG_WITHOUT_LANGUAGE)?,
        ))
        .arg(repeated(
            ARG_WITH_LANGUAGE,
            VALUE_LANGUAGE,
            say(CLI_ARG_WITH_LANGUAGE)?,
        ))
        .arg(repeated(
            ARG_WITHOUT_FOLDER,
            VALUE_FOLDER,
            say(CLI_ARG_WITHOUT_FOLDER)?,
        ))
        .arg(repeated(
            ARG_WITH_FOLDER,
            VALUE_FOLDER,
            say(CLI_ARG_WITH_FOLDER)?,
        ))
        .arg(flag(ARG_RESET, say(CLI_ARG_SCOPE_RESET)?))
        .arg(flag(ARG_JSON, say(CLI_ARG_JSON)?)))
}

fn values(matches: &ArgMatches, id: &str) -> Vec<String> {
    matches
        .get_many::<String>(id)
        .into_iter()
        .flatten()
        .cloned()
        .collect()
}

/// The arguments of `scope` as the request that runs it.
pub fn args(matches: &ArgMatches) -> ScopeArgs {
    ScopeArgs {
        project: matches
            .get_one::<String>(ARG_SCOPE_PROJECT)
            .cloned()
            .unwrap_or_default(),
        without_languages: values(matches, ARG_WITHOUT_LANGUAGE),
        with_languages: values(matches, ARG_WITH_LANGUAGE),
        without_folders: values(matches, ARG_WITHOUT_FOLDER),
        with_folders: values(matches, ARG_WITH_FOLDER),
        reset: matches.get_flag(ARG_RESET),
        json: matches.get_flag(ARG_JSON),
    }
}
