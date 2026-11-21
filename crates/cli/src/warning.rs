//! The lines of a scan summary for the warnings of the scan, one per code with a count.

use codeskimr_application::ports::ScanWarning;
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_SCAN_WARNING_AMBIGUOUS_REFERENCE, CLI_SCAN_WARNING_FILES_LEFT_OUT, CLI_SCAN_WARNING_OTHER,
};
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
    /// Whether the warning counts files that the scan left out.
    fn is_file_left_out(&self) -> bool {
        ScanWarning::FILE_LEFT_OUT
            .iter()
            .any(|code| code.id() == self.code)
    }

    /// The line for the user.
    ///
    /// # Errors
    /// `I18N-201` if a message is missing.
    pub fn render(&self, messages: &Messages, language: &str) -> Result<String, I18nError> {
        let mut args = FluentArgs::new();
        args.set(arg::COUNT, FluentValue::from(self.count));
        let key = if self.code == ErrorCode::AmbiguousReference.id() {
            CLI_SCAN_WARNING_AMBIGUOUS_REFERENCE
        } else if self.is_file_left_out() {
            args.set(arg::CODE, self.code.clone());
            CLI_SCAN_WARNING_FILES_LEFT_OUT
        } else {
            args.set(arg::CODE, self.code.clone());
            CLI_SCAN_WARNING_OTHER
        };
        messages.format(language, key, Some(&args))
    }
}
