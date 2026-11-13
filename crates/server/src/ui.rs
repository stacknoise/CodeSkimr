//! The route that tells the UI about its settings.

use axum::Json;
use axum::extract::State;

use crate::settings::UiSettings;
use crate::state::AppState;

/// `GET /api/v1/ui-config`.
pub async fn config(State(app): State<AppState>) -> Json<UiSettings> {
    Json(app.ui.as_ref().clone())
}
