//! The REST routes: the list of commands, the call of one command, and the answers for
//! everything else.

use axum::Json;
use axum::body::{Body, Bytes, to_bytes};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use codeskimr_application::access;
use codeskimr_codes::ErrorCode;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::commands::CommandInfo;
use crate::consts::{API_PREFIX, MAX_BODY_BYTES, MEDIA_JSON, MEDIA_PARAMETER_SEPARATOR};
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
    let input = read_input(&headers, body, MAX_BODY_BYTES).await?;
    let output = app.api.run(app.principal, &id, input).await?;
    Ok(Json(output))
}

/// The JSON value of the body; no body is `null`, which is the input of a command without input.
async fn read_input(headers: &HeaderMap, body: Body, limit: usize) -> Result<Value, Failure> {
    let bytes = read_body(headers, body, limit).await?;
    if bytes.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_slice(&bytes).map_err(|_| Failure::code(ErrorCode::RequestInvalid))
}

/// The body as a value of type `T`; a body that is missing or not JSON is refused.
pub async fn read_json<T: DeserializeOwned>(
    headers: &HeaderMap,
    body: Body,
    limit: usize,
) -> Result<T, Failure> {
    let bytes = read_body(headers, body, limit).await?;
    serde_json::from_slice(&bytes).map_err(|_| Failure::code(ErrorCode::RequestInvalid))
}

/// The bytes of a JSON body of at most `limit` bytes.
///
/// Anything but `application/json` is refused: a form or a plain-text request from another site
/// needs no CORS preflight, a JSON one does, and CORS is closed (S-04).
async fn read_body(headers: &HeaderMap, body: Body, limit: usize) -> Result<Bytes, Failure> {
    let bytes = to_bytes(body, limit)
        .await
        .map_err(|_| Failure::code(ErrorCode::RequestTooLarge))?;
    if bytes.is_empty() {
        return Ok(bytes);
    }
    let is_json = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(MEDIA_PARAMETER_SEPARATOR).next())
        .is_some_and(|media| media.trim().eq_ignore_ascii_case(MEDIA_JSON));
    if is_json {
        Ok(bytes)
    } else {
        Err(Failure::code(ErrorCode::RequestInvalid))
    }
}

/// Any other path: a file of the UI for a `GET`, if the server has a UI, else `API-201`.
pub async fn not_found(State(app): State<AppState>, method: Method, uri: Uri) -> Response {
    let path = uri.path();
    match (
        &app.assets,
        method == Method::GET,
        path.starts_with(API_PREFIX),
    ) {
        (Some(assets), true, false) => assets.get(path).await,
        _ => Failure::code(ErrorCode::CommandNotFound).into_response(),
    }
}

/// A known path with a method it does not offer.
pub async fn wrong_method() -> Response {
    let mut response = Failure::code(ErrorCode::RequestInvalid).into_response();
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    response
}
