//! The checks every request passes before a route sees it (blueprint S-04): the `Host` and
//! `Origin` headers against DNS rebinding and cross-site requests, and the token for changes.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL, HOST, ORIGIN};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

use crate::consts::{
    BEARER_SCHEME, CACHE_CONTROL_VALUE, HOST_PORT_SEPARATOR, LOCALHOST_NAME, NO_SNIFF_HEADER,
    NO_SNIFF_VALUE, ORIGIN_SCHEME,
};
use crate::error::Failure;
use crate::token::InstallationToken;

/// The names under which this server may be reached, and the token that allows changes.
#[derive(Debug)]
pub struct Guard {
    hosts: Vec<String>,
    token: InstallationToken,
}

impl Guard {
    /// A guard for a server that listens on `bound`. It answers to the address itself and, on a
    /// loopback address, to `localhost` with the same port.
    #[must_use]
    pub fn new(bound: SocketAddr, token: InstallationToken) -> Self {
        let mut hosts = vec![bound.to_string()];
        if bound.ip().is_loopback() {
            let mut name = String::from(LOCALHOST_NAME);
            name.push(HOST_PORT_SEPARATOR);
            name.push_str(&bound.port().to_string());
            hosts.push(name);
        }
        Self { hosts, token }
    }

    fn knows(&self, host: &str) -> bool {
        self.hosts.iter().any(|own| own.eq_ignore_ascii_case(host))
    }

    /// Lets a request pass whose `Host` is one of ours and whose `Origin`, if it has one, is
    /// ours too. A browser always sends `Origin` on a cross-site request and on a WebSocket
    /// upgrade; clients without a browser send none.
    ///
    /// # Errors
    /// `API-402` if the `Host` is missing or foreign, or the `Origin` is foreign (including
    /// `null`).
    pub fn check_origin(&self, headers: &HeaderMap) -> Result<(), CodedError> {
        let host_ok = text(headers, &HOST).is_some_and(|host| self.knows(host));
        let origin_ok = headers.get(ORIGIN).is_none_or(|value| {
            value
                .to_str()
                .ok()
                .and_then(|origin| origin.strip_prefix(ORIGIN_SCHEME))
                .is_some_and(|host| self.knows(host))
        });
        if host_ok && origin_ok {
            Ok(())
        } else {
            Err(CodedError::new(ErrorCode::HostOrOriginRejected))
        }
    }

    /// Lets a request pass that carries the installation token as `Authorization: Bearer …`.
    ///
    /// # Errors
    /// `API-401` if the header is missing, malformed or holds another token. The answer is the
    /// same in each case.
    pub fn check_token(&self, headers: &HeaderMap) -> Result<(), CodedError> {
        let accepted = text(headers, &AUTHORIZATION)
            .and_then(|value| value.strip_prefix(BEARER_SCHEME))
            .is_some_and(|token| self.token.accepts(token));
        if accepted {
            Ok(())
        } else {
            Err(CodedError::new(ErrorCode::TokenMissingOrInvalid))
        }
    }
}

/// The value of a header that is present and plain text.
fn text<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// The layer around every route: refuses a foreign `Host` or `Origin`, and marks every answer as
/// not to be cached or sniffed.
pub async fn screen(State(guard): State<Arc<Guard>>, request: Request, next: Next) -> Response {
    let mut response = match guard.check_origin(request.headers()) {
        Ok(()) => next.run(request).await,
        Err(error) => Failure(error).into_response(),
    };
    let headers = response.headers_mut();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_VALUE));
    headers.insert(
        HeaderName::from_static(NO_SNIFF_HEADER),
        HeaderValue::from_static(NO_SNIFF_VALUE),
    );
    response
}
