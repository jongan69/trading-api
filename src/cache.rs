use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub data: Value,
    pub expires_at: Instant,
}

impl CacheEntry {
    pub fn new(data: Value, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

#[derive(Clone)]
pub struct MemoryCache {
    storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        let storage = self.storage.read().await;
        if let Some(entry) = storage.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, data: Value, ttl: Duration) {
        let mut storage = self.storage.write().await;
        storage.insert(key, CacheEntry::new(data, ttl));
    }

    pub async fn cleanup_expired(&self) {
        let mut storage = self.storage.write().await;
        storage.retain(|_, entry| !entry.is_expired());
    }

    pub async fn clear(&self) {
        let mut storage = self.storage.write().await;
        storage.clear();
    }

    pub async fn size(&self) -> usize {
        let storage = self.storage.read().await;
        storage.len()
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cache_key(prefix: &str, params: &[(&str, &str)]) -> String {
    let mut key = prefix.to_string();
    for (k, v) in params {
        key.push(':');
        key.push_str(k);
        key.push('=');
        key.push_str(v);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = MemoryCache::new();
        let key = "test_key";
        let value = serde_json::json!({"message": "hello"});

        cache.set(key.to_string(), value.clone(), Duration::from_secs(60)).await;
        let cached_value = cache.get(key).await;
        assert_eq!(cached_value, Some(value));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MemoryCache::new();
        let key = "expiring_key";
        let value = serde_json::json!({"message": "expires soon"});

        cache.set(key.to_string(), value.clone(), Duration::from_millis(10)).await;
        sleep(Duration::from_millis(20)).await;
        
        let cached_value = cache.get(key).await;
        assert_eq!(cached_value, None);
    }

    #[test]
    fn test_cache_key_generation() {
        let key = cache_key("yahoo_prices", &[("symbol", "AAPL"), ("range", "1d")]);
        assert_eq!(key, "yahoo_prices:symbol=AAPL:range=1d");
    }
}