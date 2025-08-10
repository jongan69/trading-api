use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde_json::json;

use crate::errors::ApiError;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

#[derive(Debug)]
struct RateLimitState {
    requests: Vec<Instant>,
    _last_cleanup: Instant,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            _last_cleanup: Instant::now(),
        }
    }

    fn cleanup_old_requests(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(60);
        self.requests.retain(|&time| time > cutoff);
    }

    fn can_make_request(&mut self, config: &RateLimitConfig) -> bool {
        self.cleanup_old_requests();
        
        if self.requests.len() >= config.requests_per_minute as usize {
            return false;
        }
        
        self.requests.push(Instant::now());
        true
    }
}

pub struct RateLimiter {
    clients: Arc<RwLock<HashMap<String, RateLimitState>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), ApiError> {
        let mut clients = self.clients.write().await;
        
        let state = clients.entry(client_id.to_string()).or_insert_with(RateLimitState::new);
        
        if state.can_make_request(&self.config) {
            Ok(())
        } else {
            Err(ApiError::RateLimit("Rate limit exceeded".to_string()))
        }
    }
}

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract client identifier (IP address or API key)
    let client_id = extract_client_id(&request);
    
    // Get rate limiter from extensions (you'll need to add this to your app state)
    // For now, we'll use a simple approach
    let rate_limiter = RateLimiter::new(RateLimitConfig::default());
    
    // Check rate limit
    rate_limiter.check_rate_limit(&client_id).await?;
    
    // Continue with the request
    Ok(next.run(request).await)
}

pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    
    // Log slow requests
    if duration > Duration::from_secs(5) {
        tracing::warn!("Slow request: {}ms", duration.as_millis());
    }
    
    // Log errors
    if response.status().is_client_error() || response.status().is_server_error() {
        tracing::error!(
            "Request failed: {} {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        );
    }
    
    response
}

pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add CORS headers
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Authorization"),
    );
    
    response
}

fn extract_client_id(request: &Request) -> String {
    // Try to get API key from headers first
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                return stripped.to_string();
            }
        }
    }
    
    // Fall back to IP address
    request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn retry_middleware<F, Fut, T, E>(
    mut f: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                
                if attempt < max_retries {
                    let delay = base_delay * 2_u32.pow(attempt);
                    tracing::warn!("Request failed, retrying in {:?}: {:?}", delay, last_error);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

pub fn create_error_response(status: StatusCode, message: &str) -> Response {
    let body = json!({
        "error": message,
        "status": status.as_u16(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}
