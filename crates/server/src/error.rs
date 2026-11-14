//! The error answer of every route: a registered code and its English description.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use codeskimr_application::error::CodedError;
use codeskimr_codes::{Class, Coded, ErrorCode};
use serde::Serialize;

/// The body of an error response (blueprint 11.5). It names a code and nothing else: neither the
/// permission that was missing nor the cause, so a denial is no oracle (blueprint 10.1, S-06).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorBody {
    /// Registered code such as `API-401`.
    pub code: &'static str,
    /// English developer description of the code.
    pub description: &'static str,
}

/// A failed request. It turns into the status of the code's class and an [`ErrorBody`].
#[derive(Debug)]
pub struct Failure(pub CodedError);

impl Failure {
    /// A failure with `code` and no cause.
    #[must_use]
    pub const fn code(code: ErrorCode) -> Self {
        Self(CodedError::new(code))
    }
}

impl From<CodedError> for Failure {
    fn from(error: CodedError) -> Self {
        Self(error)
    }
}

/// The HTTP status that belongs to `code`.
#[must_use]
pub fn status_of(code: ErrorCode) -> StatusCode {
    match code {
        ErrorCode::PermissionDenied | ErrorCode::HostOrOriginRejected => StatusCode::FORBIDDEN,
        ErrorCode::RequestTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ErrorCode::RequestTimedOut => StatusCode::REQUEST_TIMEOUT,
        _ => match code.class() {
            Class::InvalidInput => StatusCode::BAD_REQUEST,
            Class::NotFound => StatusCode::NOT_FOUND,
            Class::Conflict => StatusCode::CONFLICT,
            Class::Permission => StatusCode::UNAUTHORIZED,
            Class::Limit => StatusCode::SERVICE_UNAVAILABLE,
            Class::External => StatusCode::BAD_GATEWAY,
            Class::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        let code = self.0.code();
        let body = ErrorBody {
            code: code.id(),
            description: code.description(),
        };
        (status_of(code), Json(body)).into_response()
    }
}
