
use dotenv::dotenv;
use reqwest::Client;
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
    let optimized_client = trading_api::optimized_client::OptimizedApiClient::new(cache.clone())
        .map_err(|e| format!("Failed to create optimized client: {e}"))?;
    
    let state = AppState {
        http: Client::new(),
        yahoo: std::sync::Arc::new(YahooConnector::new()?),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(16)),
        config: std::sync::Arc::new(config),
        cache: cache.clone(),
        rate_limiter,
        optimized_client,
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
    println!("listening on http://{host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
// OpenAPI moved to library build_app