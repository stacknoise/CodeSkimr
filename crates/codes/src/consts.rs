//! Fixed values of this crate: severity labels and the class digits of the code ids.

/// Label of `Severity::Error`.
pub const SEVERITY_ERROR: &str = "error";
/// Label of `Severity::Warning`.
pub const SEVERITY_WARNING: &str = "warning";
/// Label of `Severity::Info`.
pub const SEVERITY_INFO: &str = "info";

/// Class digit of `Class::InvalidInput`.
pub const CLASS_INVALID_INPUT: u8 = 1;
/// Class digit of `Class::NotFound`.
pub const CLASS_NOT_FOUND: u8 = 2;
/// Class digit of `Class::Conflict`.
pub const CLASS_CONFLICT: u8 = 3;
/// Class digit of `Class::Permission`.
pub const CLASS_PERMISSION: u8 = 4;
/// Class digit of `Class::Limit`.
pub const CLASS_LIMIT: u8 = 5;
/// Class digit of `Class::External`.
pub const CLASS_EXTERNAL: u8 = 6;
/// Class digit of `Class::Internal`.
pub const CLASS_INTERNAL: u8 = 9;
