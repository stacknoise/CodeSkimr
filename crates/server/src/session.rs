//! The routes that give the browser the installation token (ADR 0028, ADR 0030): creating the
//! one-time launch code, asking whether it is still waiting, and exchanging it for the token.
//!
//! These routes and the settings route are the only ones outside the command registry. They hold
//! no logic beyond the code store.

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderMap;
use codeskimr_codes::ErrorCode;
use serde::{Deserialize, Serialize};

use crate::consts::{
    LOG_LAUNCH_CODE_ISSUED, LOG_LAUNCH_CODE_REFUSED, LOG_LAUNCH_CODE_USED, MAX_EXCHANGE_BYTES,
};
use crate::error::Failure;
use crate::launch::reveal;
use crate::routes::read_json;
use crate::state::AppState;

/// Answer of `POST /api/v1/session/launch-code`.
#[derive(Serialize)]
pub struct IssuedCode {
    /// The code. It is shown once, to the caller that has the token.
    code: String,
    /// How long the code is valid, in seconds.
    valid_seconds: u64,
}

/// Answer of `GET /api/v1/session/launch-code`.
#[derive(Debug, Serialize)]
pub struct CodeState {
    /// Whether a code is waiting to be used.
    pending: bool,
}

/// Body of `POST /api/v1/session/exchange`.
#[derive(Deserialize)]
pub struct ExchangeRequest {
    code: String,
}

/// Answer of `POST /api/v1/session/exchange`.
#[derive(Serialize)]
pub struct Exchanged {
    /// The installation token. The UI keeps it in `sessionStorage` and sends it as a bearer.
    token: String,
}

/// `POST /api/v1/session/launch-code`: a new code for the caller, who must hold the token.
/// The code is for the browser the caller is about to start; a code created earlier is void.
pub async fn issue(
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<IssuedCode>, Failure> {
    let token = app.guard.bearer(&headers)?;
    let code = app
        .launch
        .issue(token)
        .ok_or_else(|| Failure::code(ErrorCode::UnexpectedInternalError))?;
    tracing::info!(event = LOG_LAUNCH_CODE_ISSUED);
    Ok(Json(IssuedCode {
        code: reveal(&code).to_owned(),
        valid_seconds: app.launch.lifetime().as_secs(),
    }))
}

/// `GET /api/v1/session/launch-code`: whether the code is still waiting. The launcher asks, to
/// remove its file as soon as the browser has taken the code.
pub async fn state(
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CodeState>, Failure> {
    app.guard.check_token(&headers)?;
    Ok(Json(CodeState {
        pending: app.launch.is_pending(),
    }))
}

/// `POST /api/v1/session/exchange`: the token for a valid code. `Host` and `Origin` were checked
/// by the layer around every route; the code takes the place of the token check (S-04).
pub async fn exchange(
    State(app): State<AppState>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<Exchanged>, Failure> {
    let request: ExchangeRequest = read_json(&headers, body, MAX_EXCHANGE_BYTES).await?;
    let Some(token) = app.launch.redeem(&request.code) else {
        tracing::warn!(event = LOG_LAUNCH_CODE_REFUSED);
        return Err(Failure::code(ErrorCode::LaunchCodeInvalid));
    };
    tracing::info!(event = LOG_LAUNCH_CODE_USED);
    Ok(Json(Exchanged {
        token: reveal(&token).to_owned(),
    }))
}
