//! The lines of a scan summary for the warnings of the scan, one per code with a count.

use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{CLI_SCAN_WARNING_AMBIGUOUS_REFERENCE, CLI_SCAN_WARNING_OTHER};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde::Serialize;

use crate::consts::arg;

/// A warning of a scan: how often the code with `code` was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WarningLine {
    /// The error code, for example `RES-301`.
    pub code: String,
    /// How many times it happened in the scan.
    pub count: u64,
}

impl WarningLine {
    /// The line for the user.
    ///
    /// # Errors
    /// `I18N-201` if a message is missing.
    pub fn render(&self, messages: &Messages, language: &str) -> Result<String, I18nError> {
        let mut args = FluentArgs::new();
        args.set(arg::COUNT, FluentValue::from(self.count));
        let key = if self.code == ErrorCode::AmbiguousReference.id() {
            CLI_SCAN_WARNING_AMBIGUOUS_REFERENCE
        } else {
            args.set(arg::CODE, self.code.clone());
            CLI_SCAN_WARNING_OTHER
        };
        messages.format(language, key, Some(&args))
    }
}
