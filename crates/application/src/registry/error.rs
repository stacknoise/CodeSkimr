//! Errors of the registry.

use std::fmt;

use codeskimr_codes::{Coded, ErrorCode};

use super::CommandId;

/// Registering a command failed. A duplicate id is a programming error in the composition root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryError {
    /// Another command already uses this id.
    DuplicateId(CommandId),
}

impl Coded for RegistryError {
    fn code(&self) -> ErrorCode {
        ErrorCode::UnexpectedInternalError
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code().description())
    }
}

impl std::error::Error for RegistryError {}
