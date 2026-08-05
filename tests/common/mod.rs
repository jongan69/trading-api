use std::sync::Arc;
use tokio::task::JoinHandle;
use trading_api::{build_app, state::AppState};
use yahoo_finance_api::YahooConnector;

/// Launch the app on an ephemeral port with ConnectInfo for per-IP rate limiting.
pub async fn spawn_app() -> (String, JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let cache = Arc::new(trading_api::cache::MemoryCache::new());
    let config = Arc::new(trading_api::config::Config {
        rate_limiting: trading_api::config::RateLimitConfig {
            requests_per_minute: 1000,
            burst_size: 200,
            enabled: false, // disabled in tests by default
        },
        ..trading_api::config::Config::default()
    });
    let state = AppState {
        http: trading_api::http::shared_client().clone(),
        yahoo: Arc::new(YahooConnector::new().unwrap()),
        concurrency_options: Arc::new(tokio::sync::Semaphore::new(8)),
        config: config.clone(),
        cache: cache.clone(),
        rate_limiter: Arc::new(trading_api::middleware::RateLimiter::new(
            trading_api::middleware::RateLimitConfig::default(),
        )),
    };
    let app = build_app(state).into_make_service_with_connect_info::<std::net::SocketAddr>();
    let h = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), h)
}
