//! `POST /api/v1/level2/confirm-window`: asks the server to open the console window in which the
//! owner confirms level 2 for a project (ADR 0071, decision 4).
//!
//! This is the one route in the product that starts a process, and it lies outside the command
//! registry, like the routes of the launch code. It has no logic of its own: it runs the guard in
//! a fixed order and calls `level2::open_confirmation`. Nothing it receives can carry a consent;
//! the owner confirms in the window, by typing the name of the project.

use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::RETRY_AFTER;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::level2::{ConfirmationDesk, Launch, Opened, open_confirmation};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, ProjectId};
use serde::{Deserialize, Serialize};

use crate::consts::{
    LOG_CONFIRM_WINDOW_OPENED, LOG_CONFIRM_WINDOW_REFUSED, MAX_CONFIRM_WINDOW_BYTES,
};
use crate::error::Failure;
use crate::guard::Guard;
use crate::peer::PeerAddr;
use crate::routes::{is_json, read_json};
use crate::state::AppState;

/// Body of the request: the id of the project the owner wants to confirm.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowRequest {
    project_id: u64,
}

/// Answer of the route.
#[derive(Debug, Serialize)]
pub struct WindowAnswer {
    /// `window` if a console window is open for the confirmation, `copy` if the screen shows the
    /// command.
    launch: Launch,
    /// The command to copy, only with `launch: copy`.
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
}

impl From<Opened> for WindowAnswer {
    fn from(opened: Opened) -> Self {
        match opened {
            Opened::Window => Self {
                launch: Launch::Window,
                command: None,
            },
            Opened::Copy { command } => Self {
                launch: Launch::Copy,
                command: Some(command),
            },
        }
    }
}

/// Runs the checks that need no body, in the order of ADR 0071: `Sec-Fetch-Site`, the JSON content
/// type, the token, the loopback peer and the right. `Host` and `Origin` were checked by the layer
/// around every route.
fn guard(app: &AppState, headers: &HeaderMap, peer: Option<PeerAddr>) -> Result<(), CodedError> {
    check_caller(&app.guard, headers, peer)?;
    access::require(&app.principal, Permission::OpenConfirmation)
}

/// The checks of [`guard`] that look at the request and the connection, not at the right.
fn check_caller(
    guard: &Guard,
    headers: &HeaderMap,
    peer: Option<PeerAddr>,
) -> Result<(), CodedError> {
    Guard::check_fetch_site(headers)?;
    if !is_json(headers) {
        return Err(CodedError::new(ErrorCode::RequestInvalid));
    }
    guard.check_token(headers)?;
    if peer.is_some_and(PeerAddr::is_local) {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::IndexerNotPermitted))
    }
}

/// The route. Without a desk (a server that was not wired for it) there is no such route.
pub async fn open(
    State(app): State<AppState>,
    peer: Option<Extension<PeerAddr>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, Failure> {
    let desk = app
        .confirmation
        .clone()
        .ok_or_else(|| Failure::code(ErrorCode::CommandNotFound))?;
    guard(&app, &headers, peer.map(|Extension(peer)| peer))?;
    let request: WindowRequest = read_json(&headers, body, MAX_CONFIRM_WINDOW_BYTES).await?;
    let principal = app.principal;
    let worker = Arc::clone(&desk);
    let id = ProjectId::new(request.project_id);
    // Starting a process is blocking work; it never runs on the thread of the server.
    let result = tokio::task::spawn_blocking(move || open_confirmation(&principal, &worker, id))
        .await
        .map_err(|_| Failure::code(ErrorCode::UnexpectedInternalError))?;
    Ok(answer(result, &desk))
}

/// The answer, and one line in the log that names the outcome and never the project.
fn answer(result: Result<Opened, CodedError>, desk: &ConfirmationDesk) -> Response {
    match result {
        Ok(opened) => {
            tracing::info!(event = LOG_CONFIRM_WINDOW_OPENED);
            Json(WindowAnswer::from(opened)).into_response()
        }
        Err(error) => {
            let code = error.code();
            tracing::warn!(event = LOG_CONFIRM_WINDOW_REFUSED, code = code.id());
            let mut response = Failure(error).into_response();
            if code == ErrorCode::ConfirmWindowRateLimited {
                response
                    .headers_mut()
                    .insert(RETRY_AFTER, retry_after(desk));
            }
            response
        }
    }
}

/// Whole seconds until the pause ends, at least one.
fn retry_after(desk: &ConfirmationDesk) -> HeaderValue {
    let left = desk.retry_after();
    let seconds = left.as_secs() + u64::from(left.subsec_nanos() > 0);
    HeaderValue::from(seconds.max(1))
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};

    use super::*;
    use crate::guard::Guard;
    use crate::token::InstallationToken;

    const TOKEN: &str = "csk_unit-test-token";

    fn guard_of_the_server() -> Guard {
        let address: SocketAddr = ([127, 0, 0, 1], 7421).into();
        Guard::new(address, InstallationToken::from_token(TOKEN))
    }

    fn headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers
    }

    fn code(result: Result<(), CodedError>) -> ErrorCode {
        result.unwrap_err().code()
    }

    #[test]
    fn only_a_peer_on_this_machine_passes_and_a_missing_peer_does_not() {
        let guard = guard_of_the_server();
        let local = |text: &str| Some(PeerAddr(text.parse().unwrap()));
        assert!(check_caller(&guard, &headers(TOKEN), local("127.0.0.1:50000")).is_ok());
        assert!(check_caller(&guard, &headers(TOKEN), local("[::1]:50000")).is_ok());
        for peer in [
            local("10.0.0.5:50000"),
            local("192.168.1.2:1"),
            local("[2001:db8::1]:1"),
            None,
        ] {
            assert_eq!(
                code(check_caller(&guard, &headers(TOKEN), peer)),
                ErrorCode::IndexerNotPermitted
            );
        }
    }

    #[test]
    fn the_token_is_checked_before_the_peer() {
        let guard = guard_of_the_server();
        let stranger = Some(PeerAddr("10.0.0.5:1".parse().unwrap()));
        assert_eq!(
            code(check_caller(&guard, &headers("csk_wrong"), stranger)),
            ErrorCode::TokenMissingOrInvalid
        );
    }
}
