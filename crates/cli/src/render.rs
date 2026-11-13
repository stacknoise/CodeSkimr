//! The texts the commands print: scan summary and error line.

use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_ERROR_LINE, CLI_SCAN_DETAILS, CLI_SCAN_FINISHED, CLI_SCAN_STARTED, CLI_WATCH_STARTED,
    CLI_WATCH_UPDATED,
};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde::Serialize;

use crate::consts::{LINE_BREAK, SECONDS_BASE, SECONDS_DECIMALS, arg};

/// The outcome of a scan as shown to the user and written by `--json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScanSummary {
    /// Name of the project.
    pub project: String,
    /// Id of the snapshot that was written.
    pub snapshot: u64,
    /// Number of scanned files.
    pub files: u32,
    /// Number of symbols found.
    pub symbols: u32,
    /// Number of references found.
    pub references: u32,
    /// Duration of the scan in seconds.
    pub seconds: f64,
}

/// The line that announces a scan.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn scan_started(
    messages: &Messages,
    language: &str,
    source: &str,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::PATH, source.to_owned());
    messages.format(language, CLI_SCAN_STARTED, Some(&args))
}

/// The two lines that close a scan.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn scan_finished(
    messages: &Messages,
    language: &str,
    summary: &ScanSummary,
) -> Result<String, I18nError> {
    let mut finished = FluentArgs::new();
    finished.set(arg::FILES, FluentValue::from(summary.files));
    finished.set(arg::SECONDS, FluentValue::from(rounded(summary.seconds)));
    let mut details = FluentArgs::new();
    details.set(arg::SYMBOLS, FluentValue::from(summary.symbols));
    details.set(arg::REFERENCES, FluentValue::from(summary.references));
    details.set(arg::SNAPSHOT, FluentValue::from(summary.snapshot));
    let first = messages.format(language, CLI_SCAN_FINISHED, Some(&finished))?;
    let second = messages.format(language, CLI_SCAN_DETAILS, Some(&details))?;
    Ok(format!("{first}{LINE_BREAK}{second}"))
}

/// The line that tells watch mode is on.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn watch_started(messages: &Messages, language: &str) -> Result<String, I18nError> {
    messages.format(language, CLI_WATCH_STARTED, None)
}

/// The line that reports one re-scan in watch mode: `changed` files in `millis` milliseconds.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn watch_updated(
    messages: &Messages,
    language: &str,
    changed: u32,
    millis: u64,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::CHANGED, FluentValue::from(changed));
    args.set(arg::MILLIS, FluentValue::from(millis));
    messages.format(language, CLI_WATCH_UPDATED, Some(&args))
}

/// The summary as one line of JSON.
///
/// # Errors
/// The serializer's error; a summary of plain numbers and text never fails.
pub fn scan_json(summary: &ScanSummary) -> Result<String, serde_json::Error> {
    serde_json::to_string(&ScanSummary {
        seconds: rounded(summary.seconds),
        ..summary.clone()
    })
}

/// The line `error[CODE]: message` for a failed command.
///
/// # Errors
/// `I18N-201` if the message of the code is missing.
pub fn error_line(
    messages: &Messages,
    language: &str,
    code: ErrorCode,
) -> Result<String, I18nError> {
    let message = messages.format(language, code.message_key(), None)?;
    let mut args = FluentArgs::new();
    args.set(arg::CODE, code.id());
    args.set(arg::MESSAGE, message);
    messages.format(language, CLI_ERROR_LINE, Some(&args))
}

fn rounded(seconds: f64) -> f64 {
    let factor = SECONDS_BASE.powi(SECONDS_DECIMALS);
    (seconds * factor).round() / factor
}
