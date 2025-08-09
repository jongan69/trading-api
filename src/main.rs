
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

    let state = AppState {
        http: Client::new(),
        yahoo: std::sync::Arc::new(YahooConnector::new()?),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
    };

    let app = trading_api::build_app(state);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
// OpenAPI moved to library build_app