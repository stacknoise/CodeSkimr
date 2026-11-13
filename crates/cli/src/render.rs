//! The texts the commands print: scan summary and error line.

use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_CREDENTIAL_DELETED, CLI_CREDENTIAL_DEVICE_PROMPT, CLI_CREDENTIAL_STORED, CLI_ERROR_LINE,
    CLI_EXPORT_WRITTEN, CLI_OPEN_READY, CLI_SCAN_DETAILS, CLI_SCAN_FINISHED, CLI_SCAN_STARTED,
    CLI_SECRET_PROMPT, CLI_SERVE_LISTENING, CLI_WATCH_STARTED, CLI_WATCH_UPDATED,
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

/// Formats `key` with text arguments.
fn format_with(
    messages: &Messages,
    language: &str,
    key: &str,
    values: &[(&str, &str)],
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, (*value).to_owned());
    }
    messages.format(language, key, Some(&args))
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
    format_with(messages, language, CLI_SCAN_STARTED, &[(arg::PATH, source)])
}

/// The line that tells where the local server listens.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn serve_started(messages: &Messages, language: &str, url: &str) -> Result<String, I18nError> {
    format_with(messages, language, CLI_SERVE_LISTENING, &[(arg::URL, url)])
}

/// The line that tells where the launch file is and how long it is good for.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn open_ready(
    messages: &Messages,
    language: &str,
    path: &str,
    seconds: u64,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::PATH, path.to_owned());
    args.set(arg::LAUNCH_SECONDS, seconds);
    messages.format(language, CLI_OPEN_READY, Some(&args))
}

/// The line that says where the export went.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn export_written(
    messages: &Messages,
    language: &str,
    path: &str,
    symbols: usize,
    kilobytes: usize,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::PATH, path.to_owned());
    args.set(arg::SYMBOLS, symbols);
    args.set(arg::KILOBYTES, kilobytes);
    messages.format(language, CLI_EXPORT_WRITTEN, Some(&args))
}

/// The line that tells that a credential was stored or replaced. It names the credential, never
/// the secret.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn credential_stored(
    messages: &Messages,
    language: &str,
    id: &str,
) -> Result<String, I18nError> {
    format_with(messages, language, CLI_CREDENTIAL_STORED, &[(arg::ID, id)])
}

/// The line that tells that a credential was removed.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn credential_deleted(
    messages: &Messages,
    language: &str,
    id: &str,
) -> Result<String, I18nError> {
    format_with(messages, language, CLI_CREDENTIAL_DELETED, &[(arg::ID, id)])
}

/// The prompt in front of the hidden input of a secret.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn secret_prompt(messages: &Messages, language: &str) -> Result<String, I18nError> {
    messages.format(language, CLI_SECRET_PROMPT, None)
}

/// What the user does to approve a device sign-in: open the page and enter the code.
///
/// # Errors
/// `I18N-201` if the message is missing.
pub fn device_prompt(
    messages: &Messages,
    language: &str,
    uri: &str,
    user_code: &str,
) -> Result<String, I18nError> {
    format_with(
        messages,
        language,
        CLI_CREDENTIAL_DEVICE_PROMPT,
        &[(arg::URI, uri), (arg::USER_CODE, user_code)],
    )
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
