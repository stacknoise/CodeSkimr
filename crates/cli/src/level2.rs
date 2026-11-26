//! The lines of a scan summary that tell what level 2 did for each language (ADR 0033).

use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_LEVEL2_REASON_CANCELLED, CLI_LEVEL2_REASON_FAILED, CLI_LEVEL2_REASON_NO_PROJECT_FILE,
    CLI_LEVEL2_REASON_NOT_CONFIRMED, CLI_LEVEL2_REASON_NOT_PERMITTED, CLI_LEVEL2_REASON_STALE,
    CLI_LEVEL2_REASON_UNAVAILABLE, CLI_SCAN_LEVEL2_CHECKED, CLI_SCAN_LEVEL2_FAILED_WORKSPACES,
    CLI_SCAN_LEVEL2_OUTSIDE_SCAN, CLI_SCAN_LEVEL2_PARTIAL, CLI_SCAN_LEVEL2_SKIPPED,
};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde::Serialize;

use crate::consts::{LINE_BREAK, arg};

/// Why level 2 did not check a language. Level 1 stays in every case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level2Reason {
    /// The caller may not start indexers.
    NotPermitted,
    /// Level 2 is not enabled and confirmed for the project.
    NotConfirmed,
    /// The indexer or its toolchain is missing or too old.
    Unavailable,
    /// The project folder holds none of the files the indexer needs.
    NoProjectFile,
    /// Every file changed since the scan.
    Stale,
    /// The scan was cancelled.
    Cancelled,
    /// The run, the import or the storage failed with this error code.
    Failed(String),
}

/// What level 2 did for one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Level2Line {
    /// Name of the language.
    pub language: String,
    /// Files whose references now come from the index.
    pub files: u64,
    /// Files that changed after the scan and stay at level 1.
    pub stale: u64,
    /// Uses of code in folders the scan skips (ADR 0061): the index knows the symbols, the
    /// snapshot does not, so these uses have no target and are not counted as unmapped.
    pub outside_scan: u64,
    /// Workspaces of the indexer that failed while others ran through (ADR 0100); their files
    /// stay at level 1.
    pub failed_workspaces: u64,
    /// Why it was skipped; `None` if it was checked.
    pub reason: Option<Level2Reason>,
}

impl Level2Line {
    /// The line for the user.
    ///
    /// # Errors
    /// `I18N-201` if a message is missing.
    pub fn render(&self, messages: &Messages, language: &str) -> Result<String, I18nError> {
        let mut args = FluentArgs::new();
        args.set(arg::LANGUAGE, self.language.clone());
        let Some(reason) = &self.reason else {
            args.set(arg::FILES, FluentValue::from(self.files));
            args.set(arg::STALE, FluentValue::from(self.stale));
            let key = if self.stale == 0 {
                CLI_SCAN_LEVEL2_CHECKED
            } else {
                CLI_SCAN_LEVEL2_PARTIAL
            };
            let checked = messages.format(language, key, Some(&args))?;
            return self.with_notes(messages, language, checked);
        };
        args.set(arg::REASON, reason_text(messages, language, reason)?);
        messages.format(language, CLI_SCAN_LEVEL2_SKIPPED, Some(&args))
    }

    /// The line of a checked language, and a line each if some workspaces failed or some uses lie
    /// in skipped folders.
    fn with_notes(
        &self,
        messages: &Messages,
        language: &str,
        checked: String,
    ) -> Result<String, I18nError> {
        let notes = [
            (self.failed_workspaces, CLI_SCAN_LEVEL2_FAILED_WORKSPACES),
            (self.outside_scan, CLI_SCAN_LEVEL2_OUTSIDE_SCAN),
        ];
        let mut lines = vec![checked];
        for (count, key) in notes {
            if count == u64::default() {
                continue;
            }
            let mut args = FluentArgs::new();
            args.set(arg::LANGUAGE, self.language.clone());
            args.set(arg::COUNT, FluentValue::from(count));
            lines.push(messages.format(language, key, Some(&args))?);
        }
        Ok(lines.join(LINE_BREAK))
    }
}

fn reason_text(
    messages: &Messages,
    language: &str,
    reason: &Level2Reason,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    let key = match reason {
        Level2Reason::NotPermitted => CLI_LEVEL2_REASON_NOT_PERMITTED,
        Level2Reason::NotConfirmed => CLI_LEVEL2_REASON_NOT_CONFIRMED,
        Level2Reason::Unavailable => CLI_LEVEL2_REASON_UNAVAILABLE,
        Level2Reason::NoProjectFile => CLI_LEVEL2_REASON_NO_PROJECT_FILE,
        Level2Reason::Stale => CLI_LEVEL2_REASON_STALE,
        Level2Reason::Cancelled => CLI_LEVEL2_REASON_CANCELLED,
        Level2Reason::Failed(code) => {
            args.set(arg::CODE, code.clone());
            args.set(arg::MESSAGE, error_text(messages, language, code)?);
            CLI_LEVEL2_REASON_FAILED
        }
    };
    messages.format(language, key, Some(&args))
}

/// The text of the registered error code `code`; empty for a code this version does not know.
fn error_text(messages: &Messages, language: &str, code: &str) -> Result<String, I18nError> {
    ErrorCode::ALL
        .iter()
        .find(|known| known.id() == code)
        .map_or_else(
            || Ok(String::new()),
            |known| messages.format(language, known.message_key(), None),
        )
}
