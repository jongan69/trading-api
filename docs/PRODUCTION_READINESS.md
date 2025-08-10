# Production Readiness Checklist

This document outlines the critical changes required to make the Trading API production-ready. Issues are categorized by severity and priority.

## 🔴 Critical Issues (Must Fix Before Production)

### 1. Dependency Security Vulnerabilities

**Issue**: The codebase has security vulnerabilities in dependencies:
- `tungstenite 0.16.0` - High severity DoS vulnerability (RUSTSEC-2023-0065)
- Multiple unmaintained dependencies

**Impact**: Security vulnerabilities could be exploited to compromise the service.

**Actions Required**:
```bash
# Check current vulnerabilities
cargo audit

# Update Cargo.toml dependencies
# Replace dotenv with dotenvy
dotenv = "0.15.0"  # Remove this
dotenvy = "0.15.7"  # Add this

# Update krakenrs or find alternative that uses secure tungstenite version
# Current: krakenrs = "5.3.1" 
# Consider switching to a different Kraken client or forking/updating
```

**Code Changes**:
```rust
// In main.rs and other files, replace:
use dotenv::dotenv;
// With:
use dotenvy::dotenv;
```

### 2. Missing Authentication & Authorization

**Issue**: No authentication mechanism protects the API endpoints.

**Impact**: Unauthorized access, potential abuse, no user tracking.

**Actions Required**:

Create `src/middleware/auth.rs`:
```rust
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use crate::state::AppState;
use crate::errors::ApiError;

pub async fn api_key_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::AuthError("Missing API key".to_string()))?;

    if !state.config.is_valid_api_key(api_key) {
        return Err(ApiError::AuthError("Invalid API key".to_string()));
    }

    Ok(next.run(request).await)
}
```

Add to config:
```rust
// In src/config.rs
pub struct Config {
    // ... existing fields
    pub api_keys: Vec<String>,
}

impl Config {
    pub fn is_valid_api_key(&self, key: &str) -> bool {
        self.api_keys.contains(&key.to_string())
    }
}
```

### 3. Overly Permissive CORS Configuration

**Issue**: `Access-Control-Allow-Origin: *` allows requests from any domain.

**Impact**: Potential security vulnerability, enables unauthorized cross-origin requests.

**Actions Required**:

Update `src/middleware.rs`:
```rust
pub async fn cors_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Get allowed origins from config instead of wildcard
    let allowed_origins = state.config.cors.allowed_origins.join(", ");
    
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_str(&allowed_origins).unwrap_or(HeaderValue::from_static("none")),
    );
    // ... rest of headers
    
    response
}
```

Add to config:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}
```

## 🟡 High Priority (Should Fix)

### 4. Rate Limiting Not Applied

**Issue**: Rate limiting middleware exists but isn't applied to routes.

**Actions Required**:

Update `src/lib.rs`:
```rust
pub fn build_app(state: state::AppState) -> Router {
    let rate_limiter = RateLimiter::new(state.config.rate_limiting.clone());
    
    Router::new()
        // ... existing routes
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(from_fn(cors_middleware))
        .layer(TraceLayer::new_for_http())
}
```

### 5. Missing Request/Response Logging

**Issue**: No structured request logging with request IDs.

**Actions Required**:

Create `src/middleware/request_id.rs`:
```rust
use axum::{extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );
    
    let mut response = next.run(request).await;
    
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );
    
    response
}
```

### 6. Error Information Leakage

**Issue**: Internal errors may expose sensitive system information.

**Actions Required**:

Update `src/errors.rs`:
```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::InternalError(_) => {
                // Log the full error but don't expose it
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
            // ... handle other errors normally
            _ => (self.status_code(), self.to_string()),
        };
        
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
```

## 🟢 Medium Priority (Recommended)

### 7. Environment-Based Configuration

**Actions Required**:

Create `.env.production` template:
```bash
# Server Configuration
HOST=0.0.0.0
PORT=3000
RUST_ENV=production

# Security
API_KEYS=key1,key2,key3
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_BURST_SIZE=20

# External APIs
ALPACA_API_KEY_ID=your_production_key
ALPACA_API_SECRET_KEY=your_production_secret

# Logging
RUST_LOG=info,trading_api=debug,tower_http=info
ENABLE_REQUEST_LOGGING=true

# Timeouts (milliseconds)
HTTP_TIMEOUT_MS=30000
CONNECT_TIMEOUT_MS=10000
```

### 8. Health Check Improvements

**Actions Required**:

Update `src/routes/system.rs`:
```rust
#[utoipa::path(get, path = "/health/ready", tag = "system")]
pub async fn readiness_check(
    State(state): State<AppState>
) -> Result<impl IntoResponse, ApiError> {
    // Check external dependencies
    let mut checks = Vec::new();
    
    // Test Alpaca connectivity
    if let Err(e) = test_alpaca_connection(&state).await {
        checks.push(format!("Alpaca: {}", e));
    }
    
    // Test Yahoo Finance
    if let Err(e) = test_yahoo_connection(&state).await {
        checks.push(format!("Yahoo: {}", e));
    }
    
    if checks.is_empty() {
        Ok((StatusCode::OK, Json(json!({"status": "ready"}))))
    } else {
        Err(ApiError::InternalError(format!("Health checks failed: {}", checks.join(", "))))
    }
}

async fn test_alpaca_connection(state: &AppState) -> Result<(), String> {
    let timeout = Duration::from_secs(5);
    tokio::time::timeout(timeout, async {
        // Minimal API call to verify connectivity
        state.http.get("https://api.alpaca.markets/v2/clock").send().await
    })
    .await
    .map_err(|_| "Timeout".to_string())?
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

### 9. Graceful Shutdown

**Actions Required**:

Update `src/main.rs`:
```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing setup
    
    let app = trading_api::build_app(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("listening on http://{host}:{port}");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
        
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
```

### 10. Circuit Breakers

**Actions Required**:

Add to `Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies
tokio-util = { version = "0.7", features = ["time"] }
```

Create `src/middleware/circuit_breaker.rs`:
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
}

#[derive(Debug)]
struct CircuitState {
    failures: usize,
    last_failure: Option<Instant>,
    state: State,
}

#[derive(Debug, PartialEq)]
enum State {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitState {
                failures: 0,
                last_failure: None,
                state: State::Closed,
            })),
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Implementation of circuit breaker logic
        // ... (full implementation would be quite long)
        f.await
    }
}
```

## Deployment Checklist

Before deploying to production:

- [ ] Update all vulnerable dependencies
- [ ] Implement API key authentication
- [ ] Configure environment-specific CORS origins
- [ ] Apply rate limiting middleware
- [ ] Add request ID tracking
- [ ] Sanitize error responses
- [ ] Set up health check endpoints
- [ ] Configure graceful shutdown
- [ ] Test with production-like load
- [ ] Set up monitoring and alerting
- [ ] Configure log aggregation
- [ ] Set up SSL/TLS termination
- [ ] Configure reverse proxy (nginx/traefik)
- [ ] Set up backup and disaster recovery

## Environment Variables for Production

Create a comprehensive `.env.production` file based on the template in section 7 above, ensuring all sensitive values are properly secured and not committed to version control.

## Monitoring Recommendations

1. **Metrics**: Implement Prometheus metrics for request counts, response times, error rates
2. **Logging**: Use structured logging with correlation IDs
3. **Alerting**: Set up alerts for high error rates, slow responses, dependency failures
4. **Health Monitoring**: Regular health check monitoring with external service

## Security Considerations

1. **API Keys**: Use long, randomly generated API keys
2. **Rate Limiting**: Implement per-user and global rate limits
3. **Input Validation**: Validate all input parameters
4. **HTTPS Only**: Ensure all traffic uses HTTPS in production
5. **Security Headers**: Add security headers (HSTS, CSP, etc.)

---

For questions or clarifications on any of these recommendations, please refer to the individual issue descriptions above or consult the development team.