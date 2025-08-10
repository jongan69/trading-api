use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Semaphore;
use yahoo_finance_api::YahooConnector;
use crate::config::Config;
use crate::cache::MemoryCache;
use crate::middleware::RateLimiter;
use crate::optimized_client::OptimizedApiClient;

#[derive(Clone)]
pub struct AppState {
    pub http: Client,
    pub yahoo: Arc<YahooConnector>,
    pub concurrency_options: Arc<Semaphore>,
    pub config: Arc<Config>,
    pub cache: Arc<MemoryCache>,
    pub rate_limiter: Arc<RateLimiter>,
    pub optimized_client: OptimizedApiClient,
}


