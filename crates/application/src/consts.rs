//! Fixed texts of the application layer.

/// Label of `StageKind::Fetch`.
pub const STAGE_FETCH: &str = "fetch";
/// Label of `StageKind::Discover`.
pub const STAGE_DISCOVER: &str = "discover";
/// Label of `StageKind::Analyze`.
pub const STAGE_ANALYZE: &str = "analyze";
/// Label of `StageKind::Resolve`.
pub const STAGE_RESOLVE: &str = "resolve";
/// Label of `StageKind::Measure`.
pub const STAGE_MEASURE: &str = "measure";
/// Label of `StageKind::Persist`.
pub const STAGE_PERSIST: &str = "persist";

/// Separator of the parts of a project-relative path.
pub const PATH_SEPARATOR: char = '/';
/// The separator Windows users type; it is turned into [`PATH_SEPARATOR`].
pub const WINDOWS_SEPARATOR: char = '\\';
/// Name of the current folder; it selects the whole project.
pub const CURRENT_DIR: &str = ".";
/// Prefix that names the current folder and is dropped from a requested path.
pub const CURRENT_DIR_PREFIX: &str = "./";
/// First id handed out for the throw-away ids of a check; nothing is stored.
pub const CHECK_FIRST_ID: u64 = 1;
/// Snapshot id used for the throw-away analysis of a check; nothing is stored.
pub const CHECK_SNAPSHOT_ID: u64 = 0;
