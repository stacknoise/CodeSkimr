//! Fixed values of the local server: routes, header names and limits.

use std::time::Duration;

/// Route that lists the commands the caller may run.
pub const COMMANDS_PATH: &str = "/api/v1/commands";
/// Route that runs one command; `{id}` is the command id such as `scan.start`.
pub const COMMAND_PATH: &str = "/api/v1/commands/{id}";
/// Route of the WebSocket that streams the domain events.
pub const EVENTS_PATH: &str = "/api/v1/events";

/// Prefix of every token the installation hands out (ADR 0019): the log redaction matches it.
pub const TOKEN_PREFIX: &str = "csk_";
/// Scheme word in front of the token in the `Authorization` header.
pub const BEARER_SCHEME: &str = "Bearer ";

/// Media type of request and response bodies.
pub const MEDIA_JSON: &str = "application/json";
/// Largest request body in bytes; a command input is small.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Largest message a WebSocket client may send in bytes. The stream is one-way, so this is small.
pub const MAX_SOCKET_MESSAGE_BYTES: usize = 1024;

/// Host name that resolves to the loopback address on every system.
pub const LOCALHOST_NAME: &str = "localhost";
/// Separates host name and port in a `Host` header.
pub const HOST_PORT_SEPARATOR: char = ':';
/// Scheme of an `Origin` header of this server; it speaks plain HTTP on the loopback address only.
pub const ORIGIN_SCHEME: &str = "http://";

/// Value of `Cache-Control` on every response: nothing is stored by caches or proxies.
pub const CACHE_CONTROL_VALUE: &str = "no-store";
/// Value of `X-Content-Type-Options` on every response.
pub const NO_SNIFF_VALUE: &str = "nosniff";
/// Name of the header that stops content sniffing.
pub const NO_SNIFF_HEADER: &str = "x-content-type-options";

/// How often the thread that carries events to a socket checks whether the socket is gone.
pub const EVENT_POLL: Duration = Duration::from_millis(200);
/// Name of the thread that carries events to a socket.
pub const EVENT_THREAD_NAME: &str = "codeskimr-events";

/// Reason of a stopped scan on the wire: the user cancelled it.
pub const REASON_CANCELLED: &str = "cancelled";
/// Reason of a stopped scan on the wire: a stage failed.
pub const REASON_FAILED: &str = "failed";
