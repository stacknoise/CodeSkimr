//! The commands `annotations` and `alerts` (ADR 0068): their definition with clap and the
//! translation of their arguments. The commands that run behind them are the registry's.

use clap::{Arg, ArgAction, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ARG_ALERT, CLI_ARG_ALERT_KINDS, CLI_ARG_ALERT_STATE, CLI_ARG_ANNOTATION, CLI_ARG_ASSIGNEE,
    CLI_ARG_CLEAR_ASSIGNEE, CLI_ARG_CLEAR_DUE, CLI_ARG_CLEAR_TEXT, CLI_ARG_DUE, CLI_ARG_JSON,
    CLI_ARG_KIND, CLI_ARG_LIMIT, CLI_ARG_NOTE_PROJECT, CLI_ARG_OFFSET, CLI_ARG_PARAM,
    CLI_ARG_STATUS, CLI_ARG_SYMBOL, CLI_ARG_SYMBOL_KIND, CLI_ARG_TEXT, CLI_ARG_TITLE, CLI_ARG_YES,
    CLI_CMD_ALERTS, CLI_CMD_ALERTS_DISMISS, CLI_CMD_ALERTS_DISMISS_ALL, CLI_CMD_ALERTS_LIST,
    CLI_CMD_ANNOTATIONS, CLI_CMD_ANNOTATIONS_ADD, CLI_CMD_ANNOTATIONS_LIST,
    CLI_CMD_ANNOTATIONS_REMOVE, CLI_CMD_ANNOTATIONS_UPDATE,
};

use crate::args::{
    AddAnnotation, AlertsAction, AnnotationFields, AnnotationsAction, ById, Clears, DismissAlerts,
    ListAlerts, ListAnnotations, PageArgs, SymbolArg, UpdateAnnotation,
};
use crate::consts::{
    ALERT_KINDS, ALERT_STATES, ANNOTATION_KINDS, ARG_ALERT, ARG_ANNOTATION, ARG_ASSIGNEE,
    ARG_CLEAR_ASSIGNEE, ARG_CLEAR_DUE, ARG_CLEAR_TEXT, ARG_DUE, ARG_JSON, ARG_KIND,
    ARG_NOTE_PROJECT, ARG_OFFSET, ARG_PARAM, ARG_ROWS, ARG_STATE, ARG_STATUS, ARG_SYMBOL,
    ARG_SYMBOL_KIND, ARG_TEXT, ARG_TITLE, ARG_YES, COMMAND_ADD, COMMAND_ALERTS,
    COMMAND_ANNOTATIONS, COMMAND_DISMISS, COMMAND_DISMISS_ALL, COMMAND_LIST, COMMAND_REMOVE,
    COMMAND_UPDATE, TASK_STATUSES, VALUE_ASSIGNEE, VALUE_DUE, VALUE_KIND, VALUE_NOTE_PROJECT,
    VALUE_NUMBER, VALUE_PARAM, VALUE_ROWS, VALUE_STATE, VALUE_STATUS, VALUE_SYMBOL,
    VALUE_SYMBOL_KIND, VALUE_TEXT, VALUE_TITLE,
};
use crate::error::CliError;

type Say<'a> = &'a dyn Fn(&str) -> Result<String, CliError>;

fn project_arg(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_NOTE_PROJECT)
        .value_name(VALUE_NOTE_PROJECT)
        .help(say(CLI_ARG_NOTE_PROJECT)?)
        .required(true))
}

fn option(id: &'static str, value: &'static str, help: String) -> Arg {
    Arg::new(id).long(id).value_name(value).help(help)
}

fn flag(id: &'static str, help: String) -> Arg {
    Arg::new(id).long(id).action(ArgAction::SetTrue).help(help)
}

fn json_flag(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(flag(ARG_JSON, say(CLI_ARG_JSON)?))
}

fn number(id: &'static str, value: &'static str, help: String) -> Arg {
    option(id, value, help).value_parser(clap::value_parser!(u32))
}

fn paging(command: Command, say: Say<'_>) -> Result<Command, CliError> {
    Ok(command
        .arg(number(ARG_ROWS, VALUE_ROWS, say(CLI_ARG_LIMIT)?))
        .arg(number(ARG_OFFSET, VALUE_ROWS, say(CLI_ARG_OFFSET)?)))
}

/// The options that name a symbol.
fn symbol_args(command: Command, say: Say<'_>) -> Result<Command, CliError> {
    Ok(command
        .arg(option(ARG_SYMBOL, VALUE_SYMBOL, say(CLI_ARG_SYMBOL)?))
        .arg(option(
            ARG_SYMBOL_KIND,
            VALUE_SYMBOL_KIND,
            say(CLI_ARG_SYMBOL_KIND)?,
        ))
        .arg(option(ARG_PARAM, VALUE_PARAM, say(CLI_ARG_PARAM)?).action(ArgAction::Append)))
}

/// The options that set the fields of an annotation.
fn field_args(command: Command, say: Say<'_>) -> Result<Command, CliError> {
    Ok(command
        .arg(option(ARG_TITLE, VALUE_TITLE, say(CLI_ARG_TITLE)?))
        .arg(option(ARG_TEXT, VALUE_TEXT, say(CLI_ARG_TEXT)?))
        .arg(option(ARG_STATUS, VALUE_STATUS, say(CLI_ARG_STATUS)?).value_parser(TASK_STATUSES))
        .arg(option(ARG_DUE, VALUE_DUE, say(CLI_ARG_DUE)?).value_parser(clap::value_parser!(i64)))
        .arg(option(ARG_ASSIGNEE, VALUE_ASSIGNEE, say(CLI_ARG_ASSIGNEE)?)))
}

fn annotation_id(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_ANNOTATION)
        .value_name(VALUE_NUMBER)
        .help(say(CLI_ARG_ANNOTATION)?)
        .value_parser(clap::value_parser!(u64))
        .required(true))
}

fn annotations_list(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_LIST)
        .about(say(CLI_CMD_ANNOTATIONS_LIST)?)
        .arg(project_arg(say)?)
        .arg(json_flag(say)?)
        .arg(option(ARG_KIND, VALUE_KIND, say(CLI_ARG_KIND)?).value_parser(ANNOTATION_KINDS))
        .arg(option(ARG_STATUS, VALUE_STATUS, say(CLI_ARG_STATUS)?).value_parser(TASK_STATUSES));
    paging(symbol_args(command, say)?, say)
}

fn annotations_add(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_ADD)
        .about(say(CLI_CMD_ANNOTATIONS_ADD)?)
        .arg(project_arg(say)?)
        .arg(json_flag(say)?)
        .arg(
            option(ARG_KIND, VALUE_KIND, say(CLI_ARG_KIND)?)
                .value_parser(ANNOTATION_KINDS)
                .required(true),
        );
    let command = symbol_args(command, say)?
        .mut_arg(ARG_SYMBOL, |arg| arg.required(true))
        .mut_arg(ARG_SYMBOL_KIND, |arg| arg.required(true));
    field_args(command, say)
}

fn annotations_update(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_UPDATE)
        .about(say(CLI_CMD_ANNOTATIONS_UPDATE)?)
        .arg(project_arg(say)?)
        .arg(annotation_id(say)?)
        .arg(json_flag(say)?)
        .arg(flag(ARG_CLEAR_TEXT, say(CLI_ARG_CLEAR_TEXT)?))
        .arg(flag(ARG_CLEAR_DUE, say(CLI_ARG_CLEAR_DUE)?))
        .arg(flag(ARG_CLEAR_ASSIGNEE, say(CLI_ARG_CLEAR_ASSIGNEE)?));
    field_args(symbol_args(command, say)?, say)
}

fn annotations_remove(say: Say<'_>) -> Result<Command, CliError> {
    Ok(Command::new(COMMAND_REMOVE)
        .about(say(CLI_CMD_ANNOTATIONS_REMOVE)?)
        .arg(project_arg(say)?)
        .arg(annotation_id(say)?)
        .arg(json_flag(say)?))
}

/// The `annotations` command with its sub-commands.
///
/// # Errors
/// `I18N-201` if a help text is missing.
pub fn annotations_command(say: Say<'_>) -> Result<Command, CliError> {
    Ok(Command::new(COMMAND_ANNOTATIONS)
        .about(say(CLI_CMD_ANNOTATIONS)?)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(annotations_list(say)?)
        .subcommand(annotations_add(say)?)
        .subcommand(annotations_update(say)?)
        .subcommand(annotations_remove(say)?))
}

fn kind_filter(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(option(ARG_KIND, VALUE_KIND, say(CLI_ARG_ALERT_KINDS)?)
        .value_parser(ALERT_KINDS)
        .action(ArgAction::Append))
}

fn alert_id(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_ALERT)
        .value_name(VALUE_NUMBER)
        .help(say(CLI_ARG_ALERT)?)
        .value_parser(clap::value_parser!(u64))
        .required(true))
}

// codeskimr: allow CHK-102 declares the arguments of one sub-command, like the list of annotations
fn alerts_list(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_LIST)
        .about(say(CLI_CMD_ALERTS_LIST)?)
        .arg(project_arg(say)?)
        .arg(json_flag(say)?)
        .arg(kind_filter(say)?)
        .arg(option(ARG_STATE, VALUE_STATE, say(CLI_ARG_ALERT_STATE)?).value_parser(ALERT_STATES));
    paging(command, say)
}

/// The `alerts` command with its sub-commands.
///
/// # Errors
/// `I18N-201` if a help text is missing.
pub fn alerts_command(say: Say<'_>) -> Result<Command, CliError> {
    Ok(Command::new(COMMAND_ALERTS)
        .about(say(CLI_CMD_ALERTS)?)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(alerts_list(say)?)
        .subcommand(
            Command::new(COMMAND_DISMISS)
                .about(say(CLI_CMD_ALERTS_DISMISS)?)
                .arg(project_arg(say)?)
                .arg(alert_id(say)?)
                .arg(json_flag(say)?),
        )
        .subcommand(
            Command::new(COMMAND_DISMISS_ALL)
                .about(say(CLI_CMD_ALERTS_DISMISS_ALL)?)
                .arg(project_arg(say)?)
                .arg(kind_filter(say)?)
                .arg(flag(ARG_YES, say(CLI_ARG_YES)?))
                .arg(json_flag(say)?),
        ))
}

fn text(matches: &ArgMatches, name: &str) -> Option<String> {
    matches.get_one::<String>(name).cloned()
}

fn project(matches: &ArgMatches) -> String {
    text(matches, ARG_NOTE_PROJECT).unwrap_or_default()
}

fn many(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .into_iter()
        .flatten()
        .cloned()
        .collect()
}

fn symbol(matches: &ArgMatches) -> Option<SymbolArg> {
    Some(SymbolArg {
        kind: text(matches, ARG_SYMBOL_KIND)?,
        name: text(matches, ARG_SYMBOL)?,
        parameters: many(matches, ARG_PARAM),
    })
}

fn page(matches: &ArgMatches) -> PageArgs {
    PageArgs {
        offset: matches.get_one::<u32>(ARG_OFFSET).copied(),
        limit: matches.get_one::<u32>(ARG_ROWS).copied(),
    }
}

fn fields(matches: &ArgMatches) -> AnnotationFields {
    AnnotationFields {
        title: text(matches, ARG_TITLE),
        text: text(matches, ARG_TEXT),
        status: text(matches, ARG_STATUS),
        due: matches.get_one::<i64>(ARG_DUE).copied(),
        assignee: text(matches, ARG_ASSIGNEE),
    }
}

fn list_annotations(list: &ArgMatches) -> ListAnnotations {
    ListAnnotations {
        project: project(list),
        symbol: symbol(list),
        kind: text(list, ARG_KIND),
        status: text(list, ARG_STATUS),
        page: page(list),
        json: list.get_flag(ARG_JSON),
    }
}

fn by_id(matches: &ArgMatches, name: &str) -> ById {
    ById {
        project: project(matches),
        id: matches.get_one::<u64>(name).copied().unwrap_or_default(),
        json: matches.get_flag(ARG_JSON),
    }
}

fn update_annotation(update: &ArgMatches) -> UpdateAnnotation {
    UpdateAnnotation {
        project: project(update),
        id: update
            .get_one::<u64>(ARG_ANNOTATION)
            .copied()
            .unwrap_or_default(),
        symbol: symbol(update),
        fields: fields(update),
        clears: Clears {
            text: update.get_flag(ARG_CLEAR_TEXT),
            due: update.get_flag(ARG_CLEAR_DUE),
            assignee: update.get_flag(ARG_CLEAR_ASSIGNEE),
        },
        json: update.get_flag(ARG_JSON),
    }
}

/// What the matched sub-command of `annotations` asks for.
///
/// # Errors
/// `CLI-101` for a missing sub-command or a symbol that has a name but no kind, or the other way.
pub fn annotations_action(matches: &ArgMatches) -> Result<AnnotationsAction, CliError> {
    match matches.subcommand() {
        Some((COMMAND_LIST, list)) => Ok(AnnotationsAction::List(list_annotations(list))),
        Some((COMMAND_ADD, add)) => Ok(AnnotationsAction::Add(AddAnnotation {
            project: project(add),
            symbol: symbol(add).ok_or_else(|| CliError::Arguments(String::new()))?,
            kind: text(add, ARG_KIND).unwrap_or_default(),
            fields: fields(add),
            json: add.get_flag(ARG_JSON),
        })),
        Some((COMMAND_UPDATE, update)) => Ok(AnnotationsAction::Update(update_annotation(update))),
        Some((COMMAND_REMOVE, remove)) => {
            Ok(AnnotationsAction::Remove(by_id(remove, ARG_ANNOTATION)))
        }
        _ => Err(CliError::Arguments(String::new())),
    }
}

/// What the matched sub-command of `alerts` asks for.
///
/// # Errors
/// `CLI-101` for a missing sub-command.
pub fn alerts_action(matches: &ArgMatches) -> Result<AlertsAction, CliError> {
    match matches.subcommand() {
        Some((COMMAND_LIST, list)) => Ok(AlertsAction::List(ListAlerts {
            project: project(list),
            state: text(list, ARG_STATE),
            kinds: many(list, ARG_KIND),
            page: page(list),
            json: list.get_flag(ARG_JSON),
        })),
        Some((COMMAND_DISMISS, dismiss)) => Ok(AlertsAction::Dismiss(by_id(dismiss, ARG_ALERT))),
        Some((COMMAND_DISMISS_ALL, all)) => Ok(AlertsAction::DismissAll(DismissAlerts {
            project: project(all),
            kinds: many(all, ARG_KIND),
            confirm: all.get_flag(ARG_YES),
            json: all.get_flag(ARG_JSON),
        })),
        _ => Err(CliError::Arguments(String::new())),
    }
}
