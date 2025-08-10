use axum::{routing::get, Json, Router};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize};
use std::collections::HashMap;
use serde_json::json;

use crate::types::HealthResponse;
use crate::state::AppState;
use crate::errors::ApiError;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SystemStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime: u64,
    pub version: String,
    pub environment: String,
    pub services: HashMap<String, ServiceStatus>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServiceStatus {
    pub status: String,
    pub last_check: u64,
    pub error_count: u64,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MetricsResponse {
    pub total_requests: u64,
    pub error_rate: f64,
    pub average_response_time: f64,
    pub active_connections: u64,
    pub memory_usage: MemoryUsage,
    pub cache_stats: CacheStats,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CacheStats {
    pub size: usize,
    pub hit_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MemoryUsage {
    pub used_mb: u64,
    pub total_mb: u64,
    pub percentage: f64,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(system_status))
        .route("/metrics", get(metrics))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        .with_state(state)
}

#[utoipa::path(get, path = "/health", tag = "system", responses((status = 200, description = "Service health", body = HealthResponse)))]
pub async fn health() -> impl IntoResponse {
    let body = HealthResponse { status: "ok" };
    (StatusCode::OK, Json(body))
}

#[utoipa::path(get, path = "/status", tag = "system", responses((status = 200, description = "Detailed system status", body = SystemStatus)))]
pub async fn system_status(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<impl IntoResponse, ApiError> {
    let start_time = SystemTime::now();
    let timestamp = start_time.duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::InternalError("Failed to get system time".to_string()))?
        .as_secs();

    // Check external services
    let mut services = HashMap::new();
    
    // Check Alpaca API
    let alpaca_status = check_alpaca_service(&state).await;
    services.insert("alpaca".to_string(), alpaca_status);
    
    // Check Yahoo Finance API
    let yahoo_status = check_yahoo_service(&state).await;
    services.insert("yahoo_finance".to_string(), yahoo_status);
    
    // Check Reddit API (if configured)
    if state.config.reddit.is_some() {
        let reddit_status = check_reddit_service(&state).await;
        services.insert("reddit".to_string(), reddit_status);
    }

    let overall_status = if services.values().all(|s| s.status == "healthy") {
        "healthy"
    } else if services.values().any(|s| s.status == "unhealthy") {
        "degraded"
    } else {
        "healthy"
    };

    let body = SystemStatus {
        status: overall_status.to_string(),
        timestamp,
        uptime: timestamp, // Simplified - you might want to track actual uptime
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        services,
    };

    Ok((StatusCode::OK, Json(body)))
}

#[utoipa::path(get, path = "/metrics", tag = "system", responses((status = 200, description = "System metrics", body = MetricsResponse)))]
pub async fn metrics(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<impl IntoResponse, ApiError> {
    let cache_size = state.cache.size().await;
    
    let body = MetricsResponse {
        total_requests: 0, // TODO: Implement request counting
        error_rate: 0.0,   // TODO: Implement error rate calculation
        average_response_time: 0.0, // TODO: Implement response time tracking
        active_connections: 0, // TODO: Implement connection tracking
        memory_usage: get_memory_usage(),
        cache_stats: CacheStats {
            size: cache_size,
            hit_rate: 0.0,     // TODO: Implement hit rate tracking
            total_hits: 0,     // TODO: Implement hit counting
            total_misses: 0,   // TODO: Implement miss counting
        },
    };

    Ok((StatusCode::OK, Json(body)))
}

#[utoipa::path(get, path = "/ready", tag = "system", responses((status = 200, description = "Readiness check")))]
pub async fn readiness_check(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<impl IntoResponse, ApiError> {
    // Check if the service is ready to handle requests
    // This includes checking external dependencies
    
    let mut checks = Vec::new();
    
    // Check Alpaca API
    let alpaca_status = check_alpaca_service(&state).await;
    checks.push(("alpaca", alpaca_status.status == "healthy"));
    
    // Check Yahoo Finance API
    let yahoo_status = check_yahoo_service(&state).await;
    checks.push(("yahoo_finance", yahoo_status.status == "healthy"));
    
    // Check if all critical services are healthy
    let all_healthy = checks.iter().all(|(_, healthy)| *healthy);
    
    if all_healthy {
        Ok((StatusCode::OK, Json(json!({"status": "ready"}))))
    } else {
        Err(ApiError::InternalError("Service not ready".to_string()))
    }
}

#[utoipa::path(get, path = "/live", tag = "system", responses((status = 200, description = "Liveness check")))]
pub async fn liveness_check() -> impl IntoResponse {
    // Simple liveness check - just verify the service is running
    (StatusCode::OK, Json(json!({"status": "alive"})))
}

async fn check_alpaca_service(state: &AppState) -> ServiceStatus {
    let start_time = std::time::Instant::now();
    
    // Try to make a simple API call to Alpaca
    let client = &state.http;
    let (api_key, api_secret) = state.config.alpaca_headers();
    
    let response = client
        .get("https://api.alpaca.markets/v2/clock")
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", api_secret)
        .send()
        .await;
    
    let response_time = start_time.elapsed().as_millis() as u64;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                ServiceStatus {
                    status: "healthy".to_string(),
                    last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    error_count: 0,
                    response_time_ms: Some(response_time),
                }
            } else {
                ServiceStatus {
                    status: "degraded".to_string(),
                    last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    error_count: 1,
                    response_time_ms: Some(response_time),
                }
            }
        }
        Err(_) => ServiceStatus {
            status: "unhealthy".to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            error_count: 1,
            response_time_ms: Some(response_time),
        }
    }
}

async fn check_yahoo_service(state: &AppState) -> ServiceStatus {
    let start_time = std::time::Instant::now();
    
    // Try to search for a simple ticker
    let result = state.yahoo.search_ticker("AAPL").await;
    let response_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(_) => ServiceStatus {
            status: "healthy".to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            error_count: 0,
            response_time_ms: Some(response_time),
        },
        Err(_) => ServiceStatus {
            status: "degraded".to_string(),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            error_count: 1,
            response_time_ms: Some(response_time),
        }
    }
}

async fn check_reddit_service(state: &AppState) -> ServiceStatus {
    let start_time = std::time::Instant::now();
    
    // Simple check - just verify we have Reddit config
    let has_config = state.config.reddit.is_some();
    let response_time = start_time.elapsed().as_millis() as u64;
    
    ServiceStatus {
        status: if has_config { "healthy".to_string() } else { "unhealthy".to_string() },
        last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        error_count: if has_config { 0 } else { 1 },
        response_time_ms: Some(response_time),
    }
}

fn get_memory_usage() -> MemoryUsage {
    // This is a simplified memory usage calculation
    // In a real application, you'd want to use a proper system monitoring library
    
    // For now, return dummy values
    MemoryUsage {
        used_mb: 128,
        total_mb: 1024,
        percentage: 12.5,
    }
}


