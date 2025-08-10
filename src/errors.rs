use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

use crate::types::ErrorResponse;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("rate limit exceeded: {0}")]
    RateLimit(String),
    #[error("authentication failed: {0}")]
    AuthError(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("internal server error: {0}")]
    InternalError(String),
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
            ApiError::RateLimit(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
            ApiError::AuthError(msg) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
            ApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
            ApiError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
            ApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: msg }),
            )
                .into_response(),
        }
    }
}


