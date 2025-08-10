use std::time::Duration;
use reqwest::{Client, ClientBuilder};
use crate::config::Config;

/// A wrapper around reqwest::Client with common configuration and error handling
pub struct ApiClient {
    client: Client,
    config: Arc<Config>,
}

impl ApiClient {
    /// Create a new API client with proper configuration
    pub fn new(config: Arc<Config>) -> Result<Self, String> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(30))
            .http2_prior_knowledge()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self { client, config })
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get Alpaca API headers
    pub fn alpaca_headers(&self) -> (String, String) {
        self.config.alpaca_headers()
    }

    /// Make a GET request with retry logic
    pub async fn get_with_retry<T>(&self, url: &str, headers: Option<Vec<(String, String)>>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut request = self.client.get(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {} {}", response.status(), response.status().canonical_reason().unwrap_or("")));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {e}"))
    }

    /// Make a GET request and return raw JSON
    pub async fn get_json(&self, url: &str, headers: Option<Vec<(String, String)>>) -> Result<serde_json::Value, String> {
        let mut request = self.client.get(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 429 {
                return Err("Rate limit exceeded".to_string());
            }
            return Err(format!("HTTP error: {} {}", response.status(), response.status().canonical_reason().unwrap_or("")));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {e}"))
    }
}

use std::sync::Arc;
