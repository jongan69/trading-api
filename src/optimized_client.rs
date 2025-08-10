use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use crate::cache::{MemoryCache, cache_key};

/// Type alias for cache parameter key-value pairs
pub type CacheParams<'a> = Vec<(&'a str, &'a str)>;

/// Type alias for HTTP headers
pub type HttpHeaders = Vec<(String, String)>;

/// Type alias for a single cached request configuration
pub type CachedRequest<'a> = (&'a str, &'a str, CacheParams<'a>);

#[derive(Clone)]
pub struct OptimizedApiClient {
    client: Client,
    cache: Arc<MemoryCache>,
}

impl OptimizedApiClient {
    pub fn new(cache: Arc<MemoryCache>) -> Result<Self, String> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(3))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(50)
            .tcp_keepalive(Duration::from_secs(30))
            .http2_prior_knowledge()
            .user_agent("TradingAPI/1.0")
            .build()
            .map_err(|e| format!("Failed to create optimized client: {e}"))?;

        Ok(Self { client, cache })
    }

    pub async fn get_cached<T>(&self, 
        url: &str, 
        cache_prefix: &str, 
        cache_params: &CacheParams<'_>,
        cache_ttl: Duration,
        headers: Option<HttpHeaders>
    ) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let cache_key = cache_key(cache_prefix, cache_params.as_slice());
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(result) = serde_json::from_value::<T>(cached) {
                return Ok(result);
            }
        }

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

        let json_value: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {e}"))?;

        let result: T = serde_json::from_value(json_value.clone())
            .map_err(|e| format!("Failed to deserialize response: {e}"))?;

        self.cache.set(cache_key, json_value, cache_ttl).await;
        Ok(result)
    }

    pub async fn get_json_cached(&self, 
        url: &str,
        cache_prefix: &str,
        cache_params: &CacheParams<'_>,
        cache_ttl: Duration,
        headers: Option<HttpHeaders>
    ) -> Result<Value, String> {
        self.get_cached::<Value>(url, cache_prefix, cache_params, cache_ttl, headers).await
    }

    pub async fn batch_get_cached<T>(&self,
        requests: Vec<CachedRequest<'_>>, // (url, cache_prefix, cache_params)
        cache_ttl: Duration,
        headers: Option<HttpHeaders>
    ) -> Vec<Result<T, String>>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let futures = requests.into_iter().map(|(url, cache_prefix, cache_params)| {
            let client = self.clone();
            let headers = headers.clone();
            async move {
                client.get_cached::<T>(url, cache_prefix, &cache_params, cache_ttl, headers).await
            }
        });

        futures::future::join_all(futures).await
    }
}