//! The program: reads the command line, runs the command and reports the outcome.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use codeskimr_application::error::CodedError;
use codeskimr_cli::{
    Action, CliError, Invocation, Parsed, error_line, language_from, parse, scan_finished,
    scan_json, scan_started,
};
use codeskimr_codes::i18n::CLI_CREDENTIALS_HINT;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::Messages;

use crate::consts::{
    ENV_LANGUAGE_TAGS, EXIT_FAILURE, EXIT_USAGE, LINE_BREAK, PLAIN_ERROR_SEPARATOR,
};
use crate::locations::{Locations, Lookup};
use crate::source_arg::SourceError;
use crate::{mcp, scan};

/// Why a command did not finish: a registered code, plus the parser's text for usage errors.
struct Failure {
    code: ErrorCode,
    detail: Option<String>,
}

impl From<CodedError> for Failure {
    fn from(error: CodedError) -> Self {
        Self {
            code: error.code(),
            detail: None,
        }
    }
}

impl From<CliError> for Failure {
    fn from(error: CliError) -> Self {
        let detail = match &error {
            CliError::Arguments(text) => Some(text.clone()),
            CliError::Messages(_) => None,
        };
        Self {
            code: error.code(),
            detail,
        }
    }
}

/// Runs the program with `arguments` (the first is the program name) and returns its exit code.
pub fn execute<I>(arguments: I, env: Lookup<'_>) -> ExitCode
where
    I: IntoIterator<Item = OsString>,
{
    let tags: Vec<String> = ENV_LANGUAGE_TAGS
        .iter()
        .filter_map(|name| env(name))
        .collect();
    let language = language_from(tags.iter().map(String::as_str));
    let Ok(messages) = Messages::load() else {
        return report_plain(ErrorCode::UnexpectedInternalError);
    };
    match dispatch(arguments, env, &messages, language) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => report(&messages, language, &failure),
    }
}

fn dispatch<I>(
    arguments: I,
    env: Lookup<'_>,
    messages: &Messages,
    language: &str,
) -> Result<(), Failure>
where
    I: IntoIterator<Item = OsString>,
{
    match parse(arguments, messages, language)? {
        Parsed::Display(text) => {
            write_out(&text);
            Ok(())
        }
        Parsed::Run(invocation) => run_invocation(&invocation, env, messages, language),
    }
}

fn run_invocation(
    invocation: &Invocation,
    env: Lookup<'_>,
    messages: &Messages,
    language: &str,
) -> Result<(), Failure> {
    let current_dir =
        std::env::current_dir().map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from));
    let locations = Locations::resolve(
        &invocation.global,
        env,
        executable_dir.as_deref(),
        &current_dir,
    )?;
    match &invocation.action {
        Action::Scan(args) => {
            let spec =
                scan::prepare(args).map_err(|error| source_failure(error, messages, language))?;
            if !args.json {
                write_err(
                    &scan_started(messages, language, spec.location()).map_err(CliError::from)?,
                );
            }
            let summary = scan::run(spec, &invocation.global, &locations)?;
            let text = if args.json {
                scan_json(&summary)
                    .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
            } else {
                scan_finished(messages, language, &summary).map_err(CliError::from)?
            };
            write_out(&text);
            Ok(())
        }
        Action::Mcp(args) => Ok(mcp::run(
            args,
            &invocation.global,
            &locations,
            &current_dir,
        )?),
    }
}

fn report(messages: &Messages, language: &str, failure: &Failure) -> ExitCode {
    let Ok(line) = error_line(messages, language, failure.code) else {
        return report_plain(failure.code);
    };
    write_err(&line);
    if let Some(detail) = &failure.detail {
        write_err(detail);
    }
    ExitCode::from(exit_status(failure.code))
}

/// The report when not even the message texts are available: code and English description.
fn report_plain(code: ErrorCode) -> ExitCode {
    let separator = PLAIN_ERROR_SEPARATOR;
    write_err(&format!("{}{separator}{}", code.id(), code.description()));
    ExitCode::from(exit_status(code))
}

fn exit_status(code: ErrorCode) -> u8 {
    if code == ErrorCode::CliArgumentsInvalid {
        EXIT_USAGE
    } else {
        EXIT_FAILURE
    }
}

fn write_out(text: &str) {
    // A closed pipe is the reader's decision; there is nobody left to tell.
    emit(io::stdout(), text);
}

fn write_err(text: &str) {
    emit(io::stderr(), text);
}

/// Writes `text` and a line break.
fn emit(mut sink: impl Write, text: &str) {
    for part in [text, LINE_BREAK] {
        if sink.write_all(part.as_bytes()).is_err() {
            return;
        }
    }
}

/// `PRJ-101`, with the reason for secrets in a URL, which the code alone does not say.
fn source_failure(error: SourceError, messages: &Messages, language: &str) -> Failure {
    let detail = match error {
        SourceError::HasCredentials => messages.format(language, CLI_CREDENTIALS_HINT, None).ok(),
        SourceError::NotASource => None,
    };
    Failure {
        code: ErrorCode::ProjectSourceInvalid,
        detail,
    }
}
