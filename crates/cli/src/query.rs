//! The query commands `symbols`, `usages`, `impact`, `context`, `unused` and `preview-rename`
//! (blueprint 10.2): their
//! definition with clap and the translation of their arguments. They ask the same questions as
//! the MCP tools of the same names, so scripts, CI and agents without MCP get the same answers.

use clap::{Arg, ArgAction, ArgMatches, Command};
use codeskimr_codes::i18n::{
    CLI_ARG_ASSESSMENT, CLI_ARG_BUDGET, CLI_ARG_COMPARE_BASE, CLI_ARG_COMPARE_PROJECT,
    CLI_ARG_COMPARE_TARGET, CLI_ARG_DEPTH, CLI_ARG_DIRECTION, CLI_ARG_INCLUDE_TESTS, CLI_ARG_JSON,
    CLI_ARG_KIND_FILTER, CLI_ARG_LANGUAGE_FILTER, CLI_ARG_LIMIT, CLI_ARG_MARKDOWN,
    CLI_ARG_QUERY_NEW_NAME, CLI_ARG_QUERY_PROJECT, CLI_ARG_QUERY_SYMBOL, CLI_ARG_QUERY_TEXT,
    CLI_ARG_UNUSED_KIND, CLI_CMD_CHANGE, CLI_CMD_COMPARE, CLI_CMD_CONTEXT, CLI_CMD_IMPACT,
    CLI_CMD_PREVIEW_RENAME, CLI_CMD_RULES, CLI_CMD_SCANS, CLI_CMD_SYMBOLS, CLI_CMD_UNUSED,
    CLI_CMD_USAGES,
};

use crate::args::{
    CompareQuery, ContextQuery, ImpactQuery, QueryAction, QueryTarget, RenameQuery, RulesQuery,
    ScansQuery, SymbolsQuery, UnusedQuery, UsagesQuery,
};
use crate::consts::{
    ARG_ASSESSMENT, ARG_BASE, ARG_BUDGET, ARG_DEPTH, ARG_DIRECTION, ARG_INCLUDE_TESTS, ARG_JSON,
    ARG_KIND, ARG_LANGUAGE, ARG_MARKDOWN, ARG_PROJECT, ARG_QUERY_NEW_NAME, ARG_QUERY_PROJECT,
    ARG_QUERY_SYMBOL, ARG_QUERY_TEXT, ARG_ROWS, ARG_SCAN_TARGET, ASSESSMENTS, BUDGETS,
    COMMAND_CHANGE, COMMAND_COMPARE, COMMAND_CONTEXT, COMMAND_IMPACT, COMMAND_PREVIEW_RENAME,
    COMMAND_RULES, COMMAND_SCANS, COMMAND_SYMBOLS, COMMAND_UNUSED, COMMAND_USAGES, DIRECTIONS,
    UNUSED_KINDS, VALUE_ASSESSMENT, VALUE_BUDGET, VALUE_DIRECTION, VALUE_KIND, VALUE_LANGUAGE,
    VALUE_QUERY_NEW_NAME, VALUE_QUERY_PROJECT, VALUE_QUERY_SYMBOL, VALUE_QUERY_TEXT, VALUE_ROWS,
    VALUE_SNAPSHOT,
};
use crate::error::CliError;

type Say<'a> = &'a dyn Fn(&str) -> Result<String, CliError>;

fn project_and_json(command: Command, say: Say<'_>) -> Result<Command, CliError> {
    Ok(command
        .arg(
            Arg::new(ARG_QUERY_PROJECT)
                .long(ARG_PROJECT)
                .value_name(VALUE_QUERY_PROJECT)
                .help(say(CLI_ARG_QUERY_PROJECT)?),
        )
        .arg(
            Arg::new(ARG_JSON)
                .long(ARG_JSON)
                .action(ArgAction::SetTrue)
                .help(say(CLI_ARG_JSON)?),
        ))
}

fn limit(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_ROWS)
        .long(ARG_ROWS)
        .value_name(VALUE_ROWS)
        .value_parser(clap::value_parser!(u32))
        .help(say(CLI_ARG_LIMIT)?))
}

fn include_tests(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_INCLUDE_TESTS)
        .long(ARG_INCLUDE_TESTS)
        .action(ArgAction::SetTrue)
        .help(say(CLI_ARG_INCLUDE_TESTS)?))
}

fn symbol(say: Say<'_>) -> Result<Arg, CliError> {
    Ok(Arg::new(ARG_QUERY_SYMBOL)
        .value_name(VALUE_QUERY_SYMBOL)
        .help(say(CLI_ARG_QUERY_SYMBOL)?)
        .required(true))
}

fn symbols_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_SYMBOLS)
        .about(say(CLI_CMD_SYMBOLS)?)
        .arg(
            Arg::new(ARG_QUERY_TEXT)
                .value_name(VALUE_QUERY_TEXT)
                .help(say(CLI_ARG_QUERY_TEXT)?)
                .required(true),
        )
        .arg(
            Arg::new(ARG_KIND)
                .long(ARG_KIND)
                .value_name(VALUE_KIND)
                .help(say(CLI_ARG_KIND_FILTER)?),
        )
        .arg(limit(say)?);
    project_and_json(command, say)
}

fn usages_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_USAGES)
        .about(say(CLI_CMD_USAGES)?)
        .arg(symbol(say)?)
        .arg(
            Arg::new(ARG_DIRECTION)
                .long(ARG_DIRECTION)
                .value_name(VALUE_DIRECTION)
                .value_parser(DIRECTIONS)
                .help(say(CLI_ARG_DIRECTION)?),
        )
        .arg(
            Arg::new(ARG_DEPTH)
                .long(ARG_DEPTH)
                .value_name(VALUE_ROWS)
                .value_parser(clap::value_parser!(u32))
                .help(say(CLI_ARG_DEPTH)?),
        )
        .arg(include_tests(say)?)
        .arg(limit(say)?);
    project_and_json(command, say)
}

fn impact_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_IMPACT)
        .about(say(CLI_CMD_IMPACT)?)
        .arg(symbol(say)?)
        .arg(limit(say)?);
    project_and_json(command, say)
}

fn context_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_CONTEXT)
        .about(say(CLI_CMD_CONTEXT)?)
        .arg(symbol(say)?)
        .arg(
            Arg::new(ARG_BUDGET)
                .long(ARG_BUDGET)
                .value_name(VALUE_BUDGET)
                .value_parser(BUDGETS)
                .help(say(CLI_ARG_BUDGET)?),
        );
    project_and_json(command, say)
}

fn unused_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_UNUSED)
        .about(say(CLI_CMD_UNUSED)?)
        .arg(
            Arg::new(ARG_KIND)
                .long(ARG_KIND)
                .value_name(VALUE_KIND)
                .value_parser(UNUSED_KINDS)
                .help(say(CLI_ARG_UNUSED_KIND)?),
        )
        .arg(
            Arg::new(ARG_LANGUAGE)
                .long(ARG_LANGUAGE)
                .value_name(VALUE_LANGUAGE)
                .help(say(CLI_ARG_LANGUAGE_FILTER)?),
        )
        .arg(
            Arg::new(ARG_ASSESSMENT)
                .long(ARG_ASSESSMENT)
                .value_name(VALUE_ASSESSMENT)
                .value_parser(ASSESSMENTS)
                .help(say(CLI_ARG_ASSESSMENT)?),
        )
        .arg(include_tests(say)?)
        .arg(limit(say)?);
    project_and_json(command, say)
}

fn rules_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_RULES)
        .about(say(CLI_CMD_RULES)?)
        .arg(limit(say)?);
    project_and_json(command, say)
}

fn rename_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_PREVIEW_RENAME)
        .about(say(CLI_CMD_PREVIEW_RENAME)?)
        .arg(symbol(say)?)
        .arg(
            Arg::new(ARG_QUERY_NEW_NAME)
                .value_name(VALUE_QUERY_NEW_NAME)
                .help(say(CLI_ARG_QUERY_NEW_NAME)?)
                .required(true),
        )
        .arg(limit(say)?);
    project_and_json(command, say)
}

/// The project of `scans`, `compare` and `change` is the first argument, not an option.
fn project_and_output(command: Command, say: Say<'_>, markdown: bool) -> Result<Command, CliError> {
    let command = command
        .arg(
            Arg::new(ARG_QUERY_PROJECT)
                .value_name(VALUE_QUERY_PROJECT)
                .help(say(CLI_ARG_COMPARE_PROJECT)?),
        )
        .arg(
            Arg::new(ARG_JSON)
                .long(ARG_JSON)
                .action(ArgAction::SetTrue)
                .help(say(CLI_ARG_JSON)?),
        );
    if !markdown {
        return Ok(command);
    }
    Ok(command.arg(
        Arg::new(ARG_MARKDOWN)
            .long(ARG_MARKDOWN)
            .action(ArgAction::SetTrue)
            .conflicts_with(ARG_JSON)
            .help(say(CLI_ARG_MARKDOWN)?),
    ))
}

fn scan_id(name: &'static str, help: String) -> Arg {
    Arg::new(name)
        .long(name)
        .value_name(VALUE_SNAPSHOT)
        .value_parser(clap::value_parser!(u64))
        .help(help)
}

fn scans_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = Command::new(COMMAND_SCANS).about(say(CLI_CMD_SCANS)?);
    project_and_output(command, say, false)
}

fn two_scans(command: Command, say: Say<'_>) -> Result<Command, CliError> {
    Ok(command
        .arg(scan_id(ARG_BASE, say(CLI_ARG_COMPARE_BASE)?))
        .arg(scan_id(ARG_SCAN_TARGET, say(CLI_ARG_COMPARE_TARGET)?)))
}

fn compare_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = two_scans(
        Command::new(COMMAND_COMPARE).about(say(CLI_CMD_COMPARE)?),
        say,
    )?;
    project_and_output(command, say, false)
}

fn change_command(say: Say<'_>) -> Result<Command, CliError> {
    let command = two_scans(
        Command::new(COMMAND_CHANGE).about(say(CLI_CMD_CHANGE)?),
        say,
    )?;
    project_and_output(command, say, true)
}

/// The ten query commands.
///
/// # Errors
/// `I18N-201` if a help text is missing.
pub fn commands(say: Say<'_>) -> Result<Vec<Command>, CliError> {
    Ok(vec![
        symbols_command(say)?,
        usages_command(say)?,
        impact_command(say)?,
        context_command(say)?,
        unused_command(say)?,
        rules_command(say)?,
        rename_command(say)?,
        scans_command(say)?,
        compare_command(say)?,
        change_command(say)?,
    ])
}

/// Whether `name` is one of the query commands.
pub fn is_query(name: &str) -> bool {
    [
        COMMAND_SYMBOLS,
        COMMAND_USAGES,
        COMMAND_IMPACT,
        COMMAND_CONTEXT,
        COMMAND_UNUSED,
        COMMAND_RULES,
        COMMAND_PREVIEW_RENAME,
        COMMAND_SCANS,
        COMMAND_COMPARE,
        COMMAND_CHANGE,
    ]
    .contains(&name)
}

fn text(matches: &ArgMatches, id: &str) -> String {
    matches.get_one::<String>(id).cloned().unwrap_or_default()
}

fn target(matches: &ArgMatches) -> QueryTarget {
    QueryTarget {
        project: matches.get_one::<String>(ARG_QUERY_PROJECT).cloned(),
        json: matches.get_flag(ARG_JSON),
    }
}

fn unused_action(matches: &ArgMatches) -> QueryAction {
    QueryAction::Unused(UnusedQuery {
        kind: matches.get_one::<String>(ARG_KIND).cloned(),
        language: matches.get_one::<String>(ARG_LANGUAGE).cloned(),
        assessment: matches.get_one::<String>(ARG_ASSESSMENT).cloned(),
        include_tests: matches.get_flag(ARG_INCLUDE_TESTS),
        limit: matches.get_one::<u32>(ARG_ROWS).copied(),
        target: target(matches),
    })
}

fn rename_action(matches: &ArgMatches) -> QueryAction {
    QueryAction::Rename(RenameQuery {
        symbol: text(matches, ARG_QUERY_SYMBOL),
        new_name: text(matches, ARG_QUERY_NEW_NAME),
        limit: matches.get_one::<u32>(ARG_ROWS).copied(),
        target: target(matches),
    })
}

fn compare_query(matches: &ArgMatches) -> CompareQuery {
    CompareQuery {
        base: matches.get_one::<u64>(ARG_BASE).copied(),
        newer: matches.get_one::<u64>(ARG_SCAN_TARGET).copied(),
        markdown: matches
            .try_get_one::<bool>(ARG_MARKDOWN)
            .ok()
            .flatten()
            .copied()
            == Some(true),
        target: target(matches),
    }
}

/// The request of the query command `name`; `name` is one for which [`is_query`] is true.
pub fn action(name: &str, matches: &ArgMatches) -> QueryAction {
    let limit = || matches.get_one::<u32>(ARG_ROWS).copied();
    match name {
        COMMAND_SYMBOLS => QueryAction::Symbols(SymbolsQuery {
            query: text(matches, ARG_QUERY_TEXT),
            kind: matches.get_one::<String>(ARG_KIND).cloned(),
            limit: limit(),
            target: target(matches),
        }),
        COMMAND_USAGES => QueryAction::Usages(UsagesQuery {
            symbol: text(matches, ARG_QUERY_SYMBOL),
            direction: matches.get_one::<String>(ARG_DIRECTION).cloned(),
            depth: matches.get_one::<u32>(ARG_DEPTH).copied(),
            include_tests: matches.get_flag(ARG_INCLUDE_TESTS),
            limit: limit(),
            target: target(matches),
        }),
        COMMAND_IMPACT => QueryAction::Impact(ImpactQuery {
            symbol: text(matches, ARG_QUERY_SYMBOL),
            limit: limit(),
            target: target(matches),
        }),
        COMMAND_UNUSED => unused_action(matches),
        COMMAND_RULES => QueryAction::Rules(RulesQuery {
            limit: limit(),
            target: target(matches),
        }),
        COMMAND_PREVIEW_RENAME => rename_action(matches),
        COMMAND_SCANS => QueryAction::Scans(ScansQuery {
            target: target(matches),
        }),
        COMMAND_COMPARE => QueryAction::Compare(compare_query(matches)),
        COMMAND_CHANGE => QueryAction::Change(compare_query(matches)),
        _ => QueryAction::Context(ContextQuery {
            symbol: text(matches, ARG_QUERY_SYMBOL),
            budget: matches.get_one::<String>(ARG_BUDGET).cloned(),
            target: target(matches),
        }),
    }
}
