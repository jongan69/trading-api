use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

use crate::types::ErrorResponse;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
            ApiError::Upstream(msg) => (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
        }
    }
}


