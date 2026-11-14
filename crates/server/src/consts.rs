//! Fixed values of the local server: routes, header names and limits.

use std::time::Duration;

/// Route that lists the commands the caller may run.
pub const COMMANDS_PATH: &str = "/api/v1/commands";
/// Route that runs one command; `{id}` is the command id such as `scan.start`.
pub const COMMAND_PATH: &str = "/api/v1/commands/{id}";
/// Route of the WebSocket that streams the domain events.
pub const EVENTS_PATH: &str = "/api/v1/events";
/// Route that creates (`POST`) and reports (`GET`) the one-time launch code (ADR 0030).
pub const LAUNCH_CODE_PATH: &str = "/api/v1/session/launch-code";
/// Route that exchanges the launch code for the installation token (ADR 0028).
pub const EXCHANGE_PATH: &str = "/api/v1/session/exchange";
/// Path of the MCP endpoint (streamable HTTP, blueprint 10.4).
pub const MCP_PATH: &str = "/mcp";
/// Route with the settings the UI needs before it shows anything.
pub const UI_CONFIG_PATH: &str = "/api/v1/ui-config";
/// Start of every API path; what is below it and unknown is an API error, not a page.
pub const API_PREFIX: &str = "/api/";
/// The page every unknown path of the UI falls back to.
pub const INDEX_FILE: &str = "index.html";

/// Random bytes of a launch code: 128 bits (ADR 0028).
pub const LAUNCH_CODE_BYTES: usize = 16;
/// Hexadecimal digits of a launch code.
pub const LAUNCH_CODE_HEX_LEN: usize = 32;
/// Bits in a hexadecimal digit.
pub const HEX_DIGIT_BITS: u32 = 4;
/// Mask of the low hexadecimal digit of a byte.
pub const HEX_LOW_MASK: u8 = 0x0F;
/// Radix of hexadecimal digits.
pub const HEX_RADIX: u32 = 16;
/// Largest body of the exchange request: a small JSON object with the code.
pub const MAX_EXCHANGE_BYTES: usize = 256;
/// Event name in the log when a launch code was created. The code itself is never logged.
pub const LOG_LAUNCH_CODE_ISSUED: &str = "launch_code_issued";
/// Event name in the log when a launch code was exchanged.
pub const LOG_LAUNCH_CODE_USED: &str = "launch_code_used";
/// Event name in the log when an exchange failed.
pub const LOG_LAUNCH_CODE_REFUSED: &str = "launch_code_refused";

/// Name of the `Content-Security-Policy` header.
pub const CSP_HEADER: &str = "content-security-policy";
/// Directives of the policy that do not depend on the address (ADR 0028): everything from this
/// origin only, no inline script or style, no framing, no base tag, no form, no plug-in.
pub const CSP_FIXED: &str = "default-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'; object-src 'none'";
/// Separates the sources of a directive.
pub const CSP_SEPARATOR: char = ' ';
/// Start of the `connect-src` directive; the WebSocket addresses follow (Safari does not let
/// `'self'` cover `ws:`).
pub const CSP_CONNECT: &str = "; connect-src 'self'";
/// Scheme of the WebSocket addresses in `connect-src`.
pub const WEBSOCKET_SCHEME: &str = "ws://";
/// Name of the `Referrer-Policy` header and its value.
pub const REFERRER_HEADER: &str = "referrer-policy";
/// Value of the `Referrer-Policy` header.
pub const REFERRER_VALUE: &str = "no-referrer";
/// Name of the `Cross-Origin-Opener-Policy` header.
pub const OPENER_HEADER: &str = "cross-origin-opener-policy";
/// Value of the `Cross-Origin-Opener-Policy` header.
pub const OPENER_VALUE: &str = "same-origin";

/// Name of the `Permissions-Policy` header.
pub const PERMISSIONS_HEADER: &str = "permissions-policy";
/// Value of the `Permissions-Policy` header: the UI uses none of the browser features that ask
/// for a permission.
pub const PERMISSIONS_VALUE: &str = "camera=(), microphone=(), geolocation=(), payment=(), usb=()";

/// Media types of the files the UI consists of, by extension.
pub const MEDIA_TYPES: [(&str, &str); 11] = [
    ("html", "text/html; charset=utf-8"),
    ("js", "text/javascript; charset=utf-8"),
    ("mjs", "text/javascript; charset=utf-8"),
    ("css", "text/css; charset=utf-8"),
    ("json", "application/json"),
    ("svg", "image/svg+xml"),
    ("png", "image/png"),
    ("ico", "image/x-icon"),
    ("woff2", "font/woff2"),
    ("txt", "text/plain; charset=utf-8"),
    ("map", "application/json"),
];
/// Media type of a file with an unknown extension.
pub const MEDIA_UNKNOWN: &str = "application/octet-stream";

/// Prefix of every token the installation hands out (ADR 0019): the log redaction matches it.
pub const TOKEN_PREFIX: &str = "csk_";
/// Scheme word in front of the token in the `Authorization` header.
pub const BEARER_SCHEME: &str = "Bearer ";

/// Media type of request and response bodies.
pub const MEDIA_JSON: &str = "application/json";
/// Separates the media type from its parameters in a `Content-Type` value.
pub const MEDIA_PARAMETER_SEPARATOR: char = ';';
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

/// Most connections the server holds at once. Further clients wait in the backlog of the
/// system until a slot is free; a connection that sends no request head gives its slot back
/// after `HEADER_READ_TIMEOUT`.
pub const MAX_CONNECTIONS: usize = 256;
/// Most event streams (web sockets) at once; one more is refused with `API-502`.
pub const MAX_EVENT_STREAMS: usize = 16;
/// Events that wait for a slow client. A client that falls further behind is disconnected
/// (close code "try again later") and connects again, so memory per stream stays bounded.
pub const EVENT_BUFFER: usize = 256;
/// How long the server waits for open connections after it was told to stop.
pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
/// How long a client may take to send the head of a request before the connection is closed.
pub const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(10);
/// Seconds a client is told to wait before it asks again after `API-502` (`Retry-After`).
pub const RETRY_AFTER_SECONDS: &str = "5";
/// How long a client may take to send the body of a request.
pub const BODY_READ_TIMEOUT: Duration = Duration::from_secs(30);
/// How long one message to a WebSocket client may take before the client is dropped.
pub const SOCKET_SEND_TIMEOUT: Duration = Duration::from_secs(10);
/// How often an idle event stream sends a ping, so that a client that left is noticed.
pub const SOCKET_PING_INTERVAL: Duration = Duration::from_secs(20);
/// How long an event stream may hear nothing from the client, not even an answer to a ping.
pub const SOCKET_IDLE_LIMIT: Duration = Duration::from_secs(90);
/// Reason text of the close frame of a stream that was cut off.
pub const SOCKET_CLOSE_REASON: &str = "stream ended";
/// Pause after a failed `accept` so that a full file table does not spin the loop.
pub const ACCEPT_RETRY: Duration = Duration::from_millis(100);

/// How often the thread that carries events to a socket checks whether the socket is gone.
pub const EVENT_POLL: Duration = Duration::from_millis(200);
/// Name of the thread that carries events to a socket.
pub const EVENT_THREAD_NAME: &str = "codeskimr-events";

/// Reason of a stopped scan on the wire: the user cancelled it.
pub const REASON_CANCELLED: &str = "cancelled";
/// Reason of a stopped scan on the wire: a stage failed.
pub const REASON_FAILED: &str = "failed";

/// Separates the segments of a request path.
pub const SEGMENT_SEPARATOR: char = '/';
/// Separates a file name from its extension.
pub const EXTENSION_SEPARATOR: char = '.';
/// Characters besides letters and digits that a path segment may hold.
pub const SEGMENT_EXTRA: [char; 3] = ['.', '_', '-'];
