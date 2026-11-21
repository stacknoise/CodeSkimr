//! The lines of a scan summary for the warnings of the scan, one per code with a count.

use codeskimr_application::ports::LeftOut;
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_SCAN_LEFT_OUT_FILE, CLI_SCAN_LEFT_OUT_FILE_KEPT, CLI_SCAN_LEFT_OUT_MORE,
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
        LeftOut::CODES.iter().any(|code| code.id() == self.code)
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

/// A file the scan left out because it alone could not be analysed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LeftOutLine {
    /// The file, relative to the root of the project.
    pub path: String,
    /// The error code that says why, for example `PRS-101`.
    pub code: String,
    /// Whether the result still shows the file as the previous scan saw it.
    pub kept: bool,
}

/// The files a scan left out: the first ones by name, the others as a number.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct LeftOutView {
    /// The files that are named.
    pub files: Vec<LeftOutLine>,
    /// How many more were left out.
    pub more: u64,
}

impl LeftOutView {
    /// The view of what a scan reports, with at most the files the scan names.
    #[must_use]
    pub fn of(left_out: &[LeftOut], more: u64) -> Self {
        Self {
            files: left_out
                .iter()
                .map(|left| LeftOutLine {
                    path: left.path.as_str().to_owned(),
                    code: left.code.id().to_owned(),
                    kept: left.kept,
                })
                .collect(),
            more,
        }
    }

    /// One line per named file and one for the others, for the user.
    ///
    /// # Errors
    /// `I18N-201` if a message is missing.
    pub fn render(&self, messages: &Messages, language: &str) -> Result<Vec<String>, I18nError> {
        let mut lines = Vec::new();
        for file in &self.files {
            let mut args = FluentArgs::new();
            args.set(arg::PATH, file.path.clone());
            args.set(arg::CODE, file.code.clone());
            let key = if file.kept {
                CLI_SCAN_LEFT_OUT_FILE_KEPT
            } else {
                CLI_SCAN_LEFT_OUT_FILE
            };
            lines.push(messages.format(language, key, Some(&args))?);
        }
        if self.more > 0 {
            let mut args = FluentArgs::new();
            args.set(arg::COUNT, FluentValue::from(self.more));
            lines.push(messages.format(language, CLI_SCAN_LEFT_OUT_MORE, Some(&args))?);
        }
        Ok(lines)
    }
}
