use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};

use crate::types::ErrorResponse;

#[derive(Debug, thiserror::Error)]
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
    #[error("external service error: {0}")]
    External(String),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl ApiError {
    /// Stable machine-readable error code derived from the variant name.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Upstream(_) => "UPSTREAM_ERROR",
            Self::RateLimit(_) => "RATE_LIMITED",
            Self::AuthError(_) => "UNAUTHORIZED",
            Self::NotFound(_) => "NOT_FOUND",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::External(_) => "EXTERNAL_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::NotImplemented(_) => "NOT_IMPLEMENTED",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::ValidationError(_) | Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::Upstream(_) | Self::External(_) => StatusCode::BAD_GATEWAY,
            Self::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::InternalError(_) | Self::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
        }
    }

    fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: self.to_string(),
            code: Some(self.code().to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = self.to_error_response();
        (status, Json(body)).into_response()
    }
}
