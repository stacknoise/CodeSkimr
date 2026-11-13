//! Constants of the source adapter. Tunable values live in `config/default.toml`; what is here is
//! fixed by the formats the adapter speaks (paths, Git, hashing).

/// Separator of a `RelativePath`, independent of the operating system.
pub const RELATIVE_PATH_SEPARATOR: char = '/';

/// Read buffer size when hashing a file, in bytes.
pub const HASH_BUFFER_BYTES: usize = 64 * 1024;

/// Depth of a shallow clone: only the newest commit is needed.
pub const SHALLOW_CLONE_DEPTH: u32 = 1;

/// How often a running clone checks whether the scan was cancelled, in milliseconds.
pub const CANCEL_POLL_MS: u64 = 50;

/// Prefix of a branch name in a full reference name.
pub const BRANCH_REF_PREFIX: &str = "refs/heads/";

/// Suffix of the folder a new clone is prepared in, before it replaces the old one.
pub const STAGING_SUFFIX: &str = ".staging";

/// Marker that separates the scheme of a URL from the rest.
pub const URL_SCHEME_MARKER: &str = "://";

/// Character that ends the credentials part of a URL.
pub const URL_USERINFO_END: char = '@';

/// Characters that end the authority part of a URL.
pub const URL_AUTHORITY_END: [char; 3] = ['/', '?', '#'];

/// Words in a transport error that show the remote asked for credentials (lower case).
pub const AUTH_FAILURE_HINTS: [&str; 4] = ["authentication", "credentials", "401", "403"];

/// Words in a fetch error that show the requested branch does not exist (lower case).
pub const MISSING_BRANCH_HINTS: [&str; 2] = ["any ref that matched", "no matching remote"];

/// Separator between the messages of an error chain.
pub const CHAIN_SEPARATOR: &str = ": ";
