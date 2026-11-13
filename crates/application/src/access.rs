//! The one place that decides whether a caller may do something (ADR 0019).
//!
//! The command registry runs [`require`] before every command; use cases that are not registered
//! commands yet call it themselves with the permission they need. Adapters never decide.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal};

use crate::error::CodedError;

/// Lets the call pass if `principal` holds `permission`.
///
/// # Errors
/// `AUTH-401` if the permission is missing. The error names neither the permission nor a project,
/// so a denial reveals nothing about what exists.
pub fn require(principal: &Principal, permission: Permission) -> Result<(), CodedError> {
    if principal.holds(permission) {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::PermissionDenied))
    }
}
