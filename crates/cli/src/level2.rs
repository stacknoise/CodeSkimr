//! The lines of a scan summary that tell what level 2 did for each language (ADR 0033).

use codeskimr_codes::i18n::{
    CLI_LEVEL2_REASON_CANCELLED, CLI_LEVEL2_REASON_FAILED, CLI_LEVEL2_REASON_NOT_CONFIRMED,
    CLI_LEVEL2_REASON_NOT_PERMITTED, CLI_LEVEL2_REASON_STALE, CLI_LEVEL2_REASON_UNAVAILABLE,
    CLI_SCAN_LEVEL2_CHECKED, CLI_SCAN_LEVEL2_PARTIAL, CLI_SCAN_LEVEL2_SKIPPED,
};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde::Serialize;

use crate::consts::arg;

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
            return messages.format(language, key, Some(&args));
        };
        args.set(arg::REASON, reason_text(messages, language, reason)?);
        messages.format(language, CLI_SCAN_LEVEL2_SKIPPED, Some(&args))
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
        Level2Reason::Stale => CLI_LEVEL2_REASON_STALE,
        Level2Reason::Cancelled => CLI_LEVEL2_REASON_CANCELLED,
        Level2Reason::Failed(code) => {
            args.set(arg::CODE, code.clone());
            CLI_LEVEL2_REASON_FAILED
        }
    };
    messages.format(language, key, Some(&args))
}
