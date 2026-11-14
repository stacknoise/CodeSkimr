//! Constants of the secrets adapter.

/// Name under which all entries are filed in the operating system's keystore.
pub const KEYSTORE_SERVICE: &str = "codeskimr";

/// Byte between the user name and the secret in a stored entry. A user name cannot contain it.
pub const FIELD_SEPARATOR: u8 = 0;

/// Largest answer of an OAuth server that is read, in bytes.
pub const MAX_RESPONSE_BYTES: u64 = 65_536;

/// Seconds after which a request to an OAuth server is given up.
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Seconds between two polls if the server names no interval (RFC 8628, section 3.2).
pub const DEFAULT_INTERVAL_SECS: u64 = 5;

/// Seconds added to the interval when the server answers `slow_down` (RFC 8628, section 3.5).
pub const SLOW_DOWN_STEP_SECS: u64 = 5;

/// Start of an address that is allowed for OAuth endpoints.
pub const HTTPS_PREFIX: &str = "https://";

/// Start of an address that may use plain HTTP, but only to [`LOOPBACK_HOST`] (the tests).
pub const LOOPBACK_SCHEME: &str = "http://";

/// The only host that may be reached without TLS.
pub const LOOPBACK_HOST: &str = "127.0.0.1";

/// Separates host and port in the authority of a URL.
pub const PORT_SEPARATOR: char = ':';

/// Precedes the host when a URL carries user info.
pub const USERINFO_MARKER: char = '@';

/// Request header and value that ask for JSON; GitHub answers with a form without it.
pub const ACCEPT_HEADER: &str = "accept";
/// Value of [`ACCEPT_HEADER`].
pub const ACCEPT_JSON: &str = "application/json";
/// Content type of a form body.
pub const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";

/// Form field of the application id.
pub const FIELD_CLIENT_ID: &str = "client_id";
/// Form field of the requested scopes.
pub const FIELD_SCOPE: &str = "scope";
/// Form field of the device code.
pub const FIELD_DEVICE_CODE: &str = "device_code";
/// Form field of the grant type.
pub const FIELD_GRANT_TYPE: &str = "grant_type";
/// Value of the grant type of the device flow (RFC 8628, section 3.4).
pub const GRANT_DEVICE_CODE: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// Bytes of one escaped byte in a form body (`%` and two hex digits).
pub const ESCAPED_WIDTH: usize = 3;
/// Bytes a form body needs besides its escaped texts: the `=` of a field and the `&` after it.
pub const FORM_OVERHEAD: usize = 2;
/// Form field of the refresh token.
pub const FIELD_REFRESH_TOKEN: &str = "refresh_token";
/// Value of the grant type that trades a refresh token for a new access token (RFC 6749, 6).
pub const GRANT_REFRESH_TOKEN: &str = "refresh_token";

/// Error of the token endpoint while the user has not approved yet.
pub const ERROR_PENDING: &str = "authorization_pending";
/// Error of the token endpoint that asks for a longer interval.
pub const ERROR_SLOW_DOWN: &str = "slow_down";

/// Bytes that are sent as they are in a form value (RFC 3986, unreserved).
pub const FORM_SAFE: &[u8] = b"-._~";

/// Host of GitHub.
pub const GITHUB_HOST: &str = "https://github.com";
/// Path of the device code endpoint of GitHub.
pub const GITHUB_DEVICE_PATH: &str = "/login/device/code";
/// Path of the token endpoint of GitHub.
pub const GITHUB_TOKEN_PATH: &str = "/login/oauth/access_token";
/// The only scope that lets GitHub clone a private repository.
pub const GITHUB_SCOPE: &str = "repo";
/// User name that goes with a GitHub token in a Git login.
pub const GITHUB_USER: &str = "x-access-token";

/// Short name of GitHub in a stored refresh link.
pub const GITHUB_NAME: &str = "github";
/// Short name of GitLab in a stored refresh link.
pub const GITLAB_NAME: &str = "gitlab";

/// Host of GitLab.
pub const GITLAB_HOST: &str = "https://gitlab.com";
/// Path of the device code endpoint of GitLab.
pub const GITLAB_DEVICE_PATH: &str = "/oauth/authorize_device";
/// Path of the token endpoint of GitLab.
pub const GITLAB_TOKEN_PATH: &str = "/oauth/token";
/// Scope of GitLab that only reads repositories.
pub const GITLAB_SCOPE: &str = "read_repository";
/// User name that goes with a GitLab token in a Git login.
pub const GITLAB_USER: &str = "oauth2";

/// Ends the host of a URL before the path that is appended to it.
pub const PATH_SEPARATOR: char = '/';

/// Shortest pause between two polls, in seconds, whatever the server says.
pub const MIN_INTERVAL_SECS: u64 = 1;

/// Separates the fields of a form body.
pub const FORM_JOIN: char = '&';
/// Separates name and value of a form field.
pub const FORM_ASSIGN: char = '=';
/// Starts a percent-escaped byte.
pub const ESCAPE_MARK: char = '%';
/// Bits of a hexadecimal digit.
pub const NIBBLE_BITS: u8 = 4;
/// Selects the low hexadecimal digit of a byte.
pub const NIBBLE_MASK: u8 = 0x0f;
/// Radix of the escape digits.
pub const HEX_RADIX: u32 = 16;

/// Longest pause between two polls, in seconds, whatever the server says.
pub const MAX_INTERVAL_SECS: u64 = 60;

/// Longest time that a device code is waited for, in seconds (RFC 8628 suggests 15 minutes).
pub const MAX_LIFETIME_SECS: u64 = 1_800;
