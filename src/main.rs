
use dotenvy::dotenv;
use yahoo_finance_api::YahooConnector;

pub use trading_api::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // load .env file

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = trading_api::config::Config::from_env()
        .map_err(|e| format!("Configuration error: {e}"))?;

    let cache = std::sync::Arc::new(trading_api::cache::MemoryCache::new());
    let rate_limiter = std::sync::Arc::new(trading_api::middleware::RateLimiter::new(
        trading_api::middleware::RateLimitConfig::default()
    ));

    let state = AppState {
        http: trading_api::http::shared_client().clone(),
        yahoo: std::sync::Arc::new(YahooConnector::new()?),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(16)),
        config: std::sync::Arc::new(config),
        cache: cache.clone(),
        rate_limiter,
    };

    let cache_cleanup = cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            cache_cleanup.cleanup_expired().await;
        }
    });

    let port = state.config.server.port;
    let host = state.config.server.host.clone();
    
    let app = trading_api::build_app(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{host}:{port}");
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}
// OpenAPI moved to library build_app