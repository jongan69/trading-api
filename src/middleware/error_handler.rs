use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{error, warn};

/// Enhanced error handler middleware
pub async fn handle_error(err: axum::BoxError) -> Response {
    if err.is::<tower::timeout::error::Elapsed>() {
        warn!("Request timeout");
        return (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "error": "Request timeout",
                "code": "TIMEOUT",
                "timestamp": chrono::Utc::now().timestamp()
            })),
        ).into_response();
    }

    if err.is::<tower::load_shed::error::Overloaded>() {
        warn!("Service overloaded");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service temporarily overloaded",
                "code": "OVERLOADED",
                "timestamp": chrono::Utc::now().timestamp()
            })),
        ).into_response();
    }

    error!("Unhandled error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "Internal server error",
            "code": "INTERNAL_ERROR",
            "timestamp": chrono::Utc::now().timestamp()
        })),
    ).into_response()
}

/// Request logging middleware
pub async fn log_request<B>(
    request: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    match status.as_u16() {
        200..=299 => tracing::info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            "Request completed successfully"
        ),
        400..=499 => tracing::warn!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            "Client error"
        ),
        500..=599 => tracing::error!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            "Server error"
        ),
        _ => tracing::info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            "Request completed"
        ),
    }
    
    response
}
