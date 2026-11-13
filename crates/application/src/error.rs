//! Errors of the application layer.

use std::error::Error;
use std::fmt;

use codeskimr_codes::{Coded, ErrorCode};

/// A failure reported through a port or a command: one registered code plus the optional cause.
///
/// Adapters convert their own error enums with [`CodedError::caused_by`], so the code of the
/// adapter error survives and no adapter detail leaks into the application layer.
#[derive(Debug)]
pub struct CodedError {
    code: ErrorCode,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl CodedError {
    /// A failure with the given code and no cause.
    #[must_use]
    pub const fn new(code: ErrorCode) -> Self {
        Self { code, cause: None }
    }

    /// Wraps an adapter error, keeping its code and keeping it as the source.
    pub fn caused_by<E>(cause: E) -> Self
    where
        E: Coded + Error + Send + Sync + 'static,
    {
        Self {
            code: cause.code(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl Coded for CodedError {
    fn code(&self) -> ErrorCode {
        self.code
    }
}

impl fmt::Display for CodedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code.description())
    }
}

impl Error for CodedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}
