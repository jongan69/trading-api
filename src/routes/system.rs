use axum::{routing::get, Json, Router};
use axum::response::IntoResponse;
use axum::http::StatusCode;

use crate::types::HealthResponse;
use crate::state::AppState;

pub fn router(_state: AppState) -> Router {
    Router::new().route("/health", get(health))
}

#[utoipa::path(get, path = "/health", tag = "system", responses((status = 200, description = "Service health", body = HealthResponse)))]
pub async fn health() -> impl IntoResponse {
    let body = HealthResponse { status: "ok" };
    (StatusCode::OK, Json(body))
}


