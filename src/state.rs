use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Semaphore;
use yahoo_finance_api::YahooConnector;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub http: Client,
    pub yahoo: Arc<YahooConnector>,
    pub concurrency_options: Arc<Semaphore>,
    pub config: Arc<Config>,
}


