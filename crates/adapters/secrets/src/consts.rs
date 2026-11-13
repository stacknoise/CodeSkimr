//! Constants of the secrets adapter.

/// Name under which all entries are filed in the operating system's keystore.
pub const KEYSTORE_SERVICE: &str = "codeskimr";

/// Byte between the user name and the secret in a stored entry. A user name cannot contain it.
pub const FIELD_SEPARATOR: u8 = 0;
