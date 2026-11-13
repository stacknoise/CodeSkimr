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
    BEARER_SCHEME, CACHE_CONTROL_VALUE, CSP_CONNECT, CSP_FIXED, CSP_HEADER, CSP_SEPARATOR,
    HOST_PORT_SEPARATOR, LOCALHOST_NAME, NO_SNIFF_HEADER, NO_SNIFF_VALUE, OPENER_HEADER,
    OPENER_VALUE, ORIGIN_SCHEME, REFERRER_HEADER, REFERRER_VALUE, WEBSOCKET_SCHEME,
};
use crate::error::Failure;
use crate::token::InstallationToken;

/// The names under which this server may be reached, and the token that allows changes.
#[derive(Debug)]
pub struct Guard {
    hosts: Vec<String>,
    token: InstallationToken,
    policy: HeaderValue,
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
        let policy = content_security_policy(&hosts);
        Self {
            hosts,
            token,
            policy,
        }
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
        self.bearer(headers).map(|_| ())
    }

    /// The installation token a request carries, after it was found to be the right one. The
    /// launch code store keeps it for the browser (ADR 0030).
    ///
    /// # Errors
    /// `API-401`, as for [`Guard::check_token`].
    pub fn bearer<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, CodedError> {
        text(headers, &AUTHORIZATION)
            .and_then(|value| value.strip_prefix(BEARER_SCHEME))
            .filter(|token| self.token.accepts(token))
            .ok_or_else(|| CodedError::new(ErrorCode::TokenMissingOrInvalid))
    }
}

/// The policy of every answer (ADR 0028): scripts and styles from this origin only, no inline
/// code, no framing. The WebSocket addresses are named because not every browser lets `'self'`
/// cover them.
fn content_security_policy(hosts: &[String]) -> HeaderValue {
    let mut policy = String::from(CSP_FIXED);
    policy.push_str(CSP_CONNECT);
    for host in hosts {
        policy.push(CSP_SEPARATOR);
        policy.push_str(WEBSOCKET_SCHEME);
        policy.push_str(host);
    }
    HeaderValue::from_str(&policy).unwrap_or_else(|_| HeaderValue::from_static(CSP_FIXED))
}

/// The value of a header that is present and plain text.
fn text<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// The layer around every route: refuses a foreign `Host` or `Origin`, and marks every answer as
/// not to be cached or sniffed, with the content security policy.
pub async fn screen(State(guard): State<Arc<Guard>>, request: Request, next: Next) -> Response {
    let mut response = match guard.check_origin(request.headers()) {
        Ok(()) => next.run(request).await,
        Err(error) => Failure(error).into_response(),
    };
    harden(response.headers_mut(), &guard.policy);
    response
}

/// The layer of a route that is never open: refuses a request without the installation token
/// before the route sees it.
pub async fn require_token(
    State(guard): State<Arc<Guard>>,
    request: Request,
    next: Next,
) -> Response {
    match guard.check_token(request.headers()) {
        Ok(()) => next.run(request).await,
        Err(error) => Failure(error).into_response(),
    }
}

/// Sets the headers that every answer carries.
fn harden(headers: &mut HeaderMap, policy: &HeaderValue) {
    let fixed = [
        (CACHE_CONTROL.as_str(), CACHE_CONTROL_VALUE),
        (NO_SNIFF_HEADER, NO_SNIFF_VALUE),
        (REFERRER_HEADER, REFERRER_VALUE),
        (OPENER_HEADER, OPENER_VALUE),
    ];
    for (name, value) in fixed {
        headers.insert(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
    headers.insert(HeaderName::from_static(CSP_HEADER), policy.clone());
}
