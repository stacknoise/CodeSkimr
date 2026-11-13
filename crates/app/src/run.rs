//! The program: reads the command line, runs the command and reports the outcome.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use codeskimr_application::conventions::CheckReport;
use codeskimr_application::error::CodedError;
use codeskimr_cli::{
    Action, CliError, Invocation, OpenArgs, Parsed, ScanArgs, ScanSummary, check_json, check_text,
    error_line, language_from, open_ready, parse, scan_finished, scan_json, scan_started,
    serve_started, watch_started, watch_updated,
};
use codeskimr_codes::i18n::CLI_CREDENTIALS_HINT;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::Messages;

use crate::consts::{
    ENV_LANGUAGE_TAGS, EXIT_FAILURE, EXIT_USAGE, LINE_BREAK, PLAIN_ERROR_SEPARATOR,
};
use crate::locations::{Locations, Lookup};
use crate::log_writer::RedactingWriter;
use crate::logging;
use crate::source_arg::SourceError;
use crate::watch::{self, Report};
use crate::{check, mcp, open, scan, serve};

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

/// How a command that ran ended: findings are a result, not a failure, but CI needs them in the
/// exit code.
enum Outcome {
    Done,
    FindingsReported,
}

/// Runs the program with `arguments` (the first is the program name) and returns its exit code.
pub fn execute<I>(arguments: I, env: Lookup<'_>) -> ExitCode
where
    I: IntoIterator<Item = OsString>,
{
    logging::init(env);
    let tags: Vec<String> = ENV_LANGUAGE_TAGS
        .iter()
        .filter_map(|name| env(name))
        .collect();
    let language = language_from(tags.iter().map(String::as_str));
    let Ok(messages) = Messages::load() else {
        return report_plain(ErrorCode::UnexpectedInternalError);
    };
    match dispatch(arguments, env, &messages, language) {
        Ok(Outcome::Done) => ExitCode::SUCCESS,
        Ok(Outcome::FindingsReported) => ExitCode::from(EXIT_FAILURE),
        Err(failure) => report(&messages, language, &failure),
    }
}

fn dispatch<I>(
    arguments: I,
    env: Lookup<'_>,
    messages: &Messages,
    language: &str,
) -> Result<Outcome, Failure>
where
    I: IntoIterator<Item = OsString>,
{
    match parse(arguments, messages, language)? {
        Parsed::Display(text) => {
            write_out(&text);
            Ok(Outcome::Done)
        }
        Parsed::Run(invocation) => run_invocation(&invocation, env, messages, language),
    }
}

fn run_invocation(
    invocation: &Invocation,
    env: Lookup<'_>,
    messages: &Messages,
    language: &str,
) -> Result<Outcome, Failure> {
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
        Action::Scan(args) => run_scan(args, invocation, &locations, messages, language),
        Action::Mcp(args) => {
            mcp::run(args, &invocation.global, &locations, &current_dir)?;
            Ok(Outcome::Done)
        }
        Action::Open(args) => run_open(*args, invocation, &locations, env, (messages, language)),
        Action::Serve(_) => {
            serve::run(&invocation.global, &locations, |url| {
                if let Ok(line) = serve_started(messages, language, url) {
                    write_out(&line);
                }
            })?;
            Ok(Outcome::Done)
        }
        Action::Check(args) => {
            let report = check::run(args, &invocation.global, &locations, &current_dir)?;
            print_check(args.json, &report, messages, language)
        }
    }
}

/// `codeskimr open`: tells where the launch file is, then waits until the browser has used it.
fn run_open(
    args: OpenArgs,
    invocation: &Invocation,
    locations: &Locations,
    env: Lookup<'_>,
    (messages, language): (&Messages, &str),
) -> Result<Outcome, Failure> {
    let announce = |ready: &open::Ready<'_>| {
        let path = ready.path.display().to_string();
        if let Ok(line) = open_ready(messages, language, &path, ready.seconds) {
            write_out(&line);
        }
    };
    open::run(args, &invocation.global, locations, env, announce)?;
    Ok(Outcome::Done)
}

/// `codeskimr scan`, once or, with `--watch`, until the user stops it.
fn run_scan(
    args: &ScanArgs,
    invocation: &Invocation,
    locations: &Locations,
    messages: &Messages,
    language: &str,
) -> Result<Outcome, Failure> {
    let spec = scan::prepare(args).map_err(|error| source_failure(error, messages, language))?;
    if args.watch {
        watch::require_folder(&spec)?;
    }
    if !args.json {
        write_err(&scan_started(messages, language, spec.location()).map_err(CliError::from)?);
    }
    if args.watch {
        let session = scan::open(spec, &invocation.global, locations)?;
        let mut show = |report| show_report(report, args.json, messages, language);
        watch::run(&session, &mut show)?;
        return Ok(Outcome::Done);
    }
    let summary = scan::run(spec, &invocation.global, locations)?;
    write_out(&summary_text(&summary, args.json, messages, language)?);
    Ok(Outcome::Done)
}

/// Prints the report of `check` and tells whether it found anything.
fn print_check(
    json: bool,
    report: &CheckReport,
    messages: &Messages,
    language: &str,
) -> Result<Outcome, Failure> {
    let text = if json {
        check_json(report).map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
    } else {
        check_text(messages, language, report).map_err(CliError::from)?
    };
    write_out(&text);
    Ok(
        if report.findings.is_empty() && report.unmatched.is_empty() {
            Outcome::Done
        } else {
            Outcome::FindingsReported
        },
    )
}

fn summary_text(
    summary: &ScanSummary,
    json: bool,
    messages: &Messages,
    language: &str,
) -> Result<String, Failure> {
    if json {
        Ok(scan_json(summary).map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?)
    } else {
        Ok(scan_finished(messages, language, summary).map_err(CliError::from)?)
    }
}

/// Prints what watch mode reports: the first summary, then one line per re-scan. A failure ends
/// nothing here; it is printed and watching goes on.
fn show_report(report: Report, json: bool, messages: &Messages, language: &str) {
    let show = |text: Result<String, Failure>| match text {
        Ok(text) => write_out(&text),
        Err(failure) => {
            let _ = self::report(messages, language, &failure);
        }
    };
    let cli = |error| Failure::from(CliError::from(error));
    match report {
        Report::Initial(summary) => {
            show(summary_text(&summary, json, messages, language));
            if !json {
                show(watch_started(messages, language).map_err(cli));
            }
        }
        Report::Updated {
            summary,
            changed,
            millis,
        } => show(if json {
            summary_text(&summary, json, messages, language)
        } else {
            watch_updated(messages, language, changed, millis).map_err(cli)
        }),
        Report::Failed(error) => show(Err(Failure::from(error))),
    }
}

fn report(messages: &Messages, language: &str, failure: &Failure) -> ExitCode {
    tracing::debug!(code = failure.code.id(), detail = ?failure.detail);
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

/// Error output passes through the redaction layer like the log: an error text may echo an
/// argument that holds a secret (blueprint S-01, S-02).
fn write_err(text: &str) {
    emit(RedactingWriter::new(io::stderr()), text);
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
