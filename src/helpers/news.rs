use crate::sources::alpaca_data::get_alpaca_news;
use crate::sources::reddit_data::get_reddit_news;
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Simple cache structure for news data
struct NewsCache {
    data: HashMap<String, (Value, u64)>, // (data, timestamp)
    ttl: u64, // Time to live in seconds
}

impl NewsCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: ttl_seconds,
        }
    }

    fn get(&self, key: &str) -> Option<Value> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if let Some((data, timestamp)) = self.data.get(key) {
            if current_time - timestamp < self.ttl {
                return Some(data.clone());
            }
        }
        None
    }

    fn set(&mut self, key: String, value: Value) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.data.insert(key, (value, current_time));
    }

}

// Global cache instance
static NEWS_CACHE: std::sync::LazyLock<Arc<Mutex<NewsCache>>> = std::sync::LazyLock::new(|| {
    Arc::new(Mutex::new(NewsCache::new(300))) // 5 minute TTL
});

pub async fn get_news() -> Result<Value, String> {
    let reddit_timeout = Duration::from_secs(12);
    let alpaca_timeout = Duration::from_secs(8);

    let (reddit_result, alpaca_result) = tokio::try_join!(
        timeout(reddit_timeout, get_reddit_news()),
        timeout(alpaca_timeout, get_alpaca_news())
    ).map_err(|e| format!("Failed to execute news requests: {e}"))?;

    let reddit_news = match reddit_result {
        Ok(result) => result,
        Err(_) => Value::Null,
    };

    let alpaca_news = match alpaca_result {
        Ok(result) => result,
        Err(_) => Value::Null,
    };

    Ok(json!({ "reddit": reddit_news, "alpaca": alpaca_news }))
}

// Cached version that avoids redundant API calls.
// Uses a double-checked pattern: peek at cache → fetch → re-lock and insert
// so the lock is never held across network I/O.
pub async fn get_news_cached() -> Result<Value, String> {
    // Fast path: check cache with minimal lock time
    {
        let cache = NEWS_CACHE.lock().await;
        if let Some(cached_data) = cache.get("combined_news") {
            return Ok(cached_data);
        }
    } // lock dropped here — network calls happen unlocked

    // Fetch fresh data without holding the cache lock
    let fresh_data = get_news().await?;

    // Re-lock and insert (concurrent fetches are acceptable — last writer wins)
    {
        let mut cache = NEWS_CACHE.lock().await;
        cache.set("combined_news".to_string(), fresh_data.clone());
    }

    Ok(fresh_data)
}
