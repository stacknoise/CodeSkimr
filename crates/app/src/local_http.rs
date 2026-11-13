//! A minimal HTTP/1.1 client for the local server, on the loopback address only.
//!
//! `codeskimr open` needs two calls and nothing else, so no HTTP library is pulled in for it. The
//! installation token goes into the `Authorization` header of the request and nowhere else.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use serde_json::Value;

use crate::consts::{
    CONNECT_TIMEOUT_MS, HTTP_BEARER_HEADER, HTTP_FIXED_HEADERS, HTTP_HEAD_END, HTTP_HOST_HEADER,
    HTTP_LINE_BREAK, HTTP_OK, HTTP_REQUEST_SEPARATOR, HTTP_STATUS_FIELD, HTTP_UNAUTHORIZED,
    HTTP_VERSION, LOOPBACK_HOST, MAX_REPLY_BYTES, PORT_SEPARATOR, REPLY_TIMEOUT_MS,
};

/// A call to the local server.
pub struct Call<'a> {
    /// Port of the server.
    pub port: u16,
    /// `GET` or `POST`.
    pub method: &'a str,
    /// Path of the route.
    pub path: &'a str,
    /// The installation token for the `Authorization` header.
    pub token: &'a str,
}

/// The JSON body of a successful answer.
///
/// # Errors
/// - `API-601` if the server cannot be reached or does not answer as the server does.
/// - `API-401` if the server does not accept the token.
pub fn call(request: &Call<'_>) -> Result<Value, CodedError> {
    let unreachable = || CodedError::new(ErrorCode::ServerNotReachable);
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, request.port));
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_millis(CONNECT_TIMEOUT_MS))
            .map_err(|_| unreachable())?;
    let reply_timeout = Some(Duration::from_millis(REPLY_TIMEOUT_MS));
    stream
        .set_read_timeout(reply_timeout)
        .and(stream.set_write_timeout(reply_timeout))
        .map_err(|_| unreachable())?;
    stream
        .write_all(message(request).as_bytes())
        .map_err(|_| unreachable())?;
    let mut raw = String::new();
    stream
        .take(MAX_REPLY_BYTES)
        .read_to_string(&mut raw)
        .map_err(|_| unreachable())?;
    let (status, body) = split(&raw).ok_or_else(unreachable)?;
    match status {
        HTTP_OK => serde_json::from_str(body).map_err(|_| unreachable()),
        HTTP_UNAUTHORIZED => Err(CodedError::new(ErrorCode::TokenMissingOrInvalid)),
        _ => Err(unreachable()),
    }
}

fn message(request: &Call<'_>) -> String {
    let mut lines = vec![
        [request.method, request.path, HTTP_VERSION].join(HTTP_REQUEST_SEPARATOR),
        [
            HTTP_HOST_HEADER,
            LOOPBACK_HOST,
            PORT_SEPARATOR,
            &request.port.to_string(),
        ]
        .concat(),
        [HTTP_BEARER_HEADER, request.token].concat(),
    ];
    lines.extend(HTTP_FIXED_HEADERS.iter().map(|line| (*line).to_owned()));
    format!("{}{HTTP_HEAD_END}", lines.join(HTTP_LINE_BREAK))
}

/// The status and the body of a raw answer.
fn split(raw: &str) -> Option<(u16, &str)> {
    let (head, body) = raw.split_once(HTTP_HEAD_END)?;
    let status = head
        .lines()
        .next()?
        .split_whitespace()
        .nth(HTTP_STATUS_FIELD)?
        .parse()
        .ok()?;
    Some((status, body))
}
