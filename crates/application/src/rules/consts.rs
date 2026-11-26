//! Fixed texts of the architecture rules.

/// Name of the file that holds the held state, next to the project file (ADR 0104).
pub const BASELINE_FILE: &str = ".codeskimr-baseline.toml";
/// Id of the kind `forbid` in the id of a rule.
pub const ID_FORBID: &str = "forbid";
/// Id of the kind `public_api` in the id of a rule.
pub const ID_PUBLIC_API: &str = "public_api";
/// Separates the kind from the names in the id of a rule.
pub const ID_SEPARATOR: &str = ":";
/// Separates the two names of a `forbid` rule in its id.
pub const ID_ARROW: &str = ">";
/// Id of the layer rule (ADR 0054), the one rule that the project file does not name.
pub const ID_LAYERS: &str = "layers";
