//! The REST routes: the list of commands, the call of one command, and the answers for
//! everything else.

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use codeskimr_application::access;
use codeskimr_codes::ErrorCode;
use serde_json::Value;

use crate::commands::CommandInfo;
use crate::consts::{MAX_BODY_BYTES, MEDIA_JSON};
use crate::error::Failure;
use crate::state::AppState;

/// `GET /api/v1/commands`: the commands the caller may run.
pub async fn list(State(app): State<AppState>) -> Json<Vec<CommandInfo>> {
    Json(app.api.list(&app.principal))
}

/// `POST /api/v1/commands/{id}`: runs a command with the JSON body as its input and answers with
/// its output.
///
/// The order is fixed and tested: the command must exist; a command that changes something needs
/// the token; the caller needs the permission; only then the body is read. A denied caller sees
/// the same `AUTH-401` whatever it sends.
pub async fn run(
    State(app): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<Value>, Failure> {
    let descriptor = app
        .api
        .descriptor(&id)
        .ok_or_else(|| Failure::code(ErrorCode::CommandNotFound))?;
    if !descriptor.read_only {
        app.guard.check_token(&headers)?;
    }
    access::require(&app.principal, descriptor.permission)?;
    let input = read_input(&headers, body).await?;
    let output = app.api.run(app.principal, &id, input).await?;
    Ok(Json(output))
}

/// The JSON value of the body; no body is `null`, which is the input of a command without input.
///
/// Anything but `application/json` is refused: a form or a plain-text request from another site
/// needs no CORS preflight, a JSON one does, and CORS is closed (S-04).
async fn read_input(headers: &HeaderMap, body: Body) -> Result<Value, Failure> {
    let bytes = to_bytes(body, MAX_BODY_BYTES)
        .await
        .map_err(|_| Failure::code(ErrorCode::RequestTooLarge))?;
    if bytes.is_empty() {
        return Ok(Value::Null);
    }
    let is_json = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with(MEDIA_JSON));
    if !is_json {
        return Err(Failure::code(ErrorCode::RequestInvalid));
    }
    serde_json::from_slice(&bytes).map_err(|_| Failure::code(ErrorCode::RequestInvalid))
}

/// Any other path.
pub async fn not_found() -> Response {
    Failure::code(ErrorCode::CommandNotFound).into_response()
}

/// A known path with a method it does not offer.
pub async fn wrong_method() -> Response {
    let mut response = Failure::code(ErrorCode::RequestInvalid).into_response();
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    response
}
