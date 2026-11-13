//! Errors of the command line (error group `CLI`, blueprint section 11).

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::I18nError;

/// Why the command line could not be turned into a request.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The arguments are invalid (`CLI-101`); the text is what the parser reported.
    #[error("{}", ErrorCode::CliArgumentsInvalid.description())]
    Arguments(String),
    /// A message text is missing.
    #[error(transparent)]
    Messages(#[from] I18nError),
}

impl Coded for CliError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Arguments(_) => ErrorCode::CliArgumentsInvalid,
            Self::Messages(error) => error.code(),
        }
    }
}
