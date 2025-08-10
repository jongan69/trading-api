use crate::sources::alpaca_data::get_alpaca_news;
use crate::sources::finviz_data::fetch_finviz_news;
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

    fn clear_expired(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.data.retain(|_, (_, timestamp)| {
            current_time - *timestamp < self.ttl
        });
    }
}

// Global cache instance
lazy_static::lazy_static! {
    static ref NEWS_CACHE: Arc<Mutex<NewsCache>> = Arc::new(Mutex::new(NewsCache::new(300))); // 5 minute TTL
}

pub async fn get_news() -> Result<Value, String> {
    // Configure timeouts per service (some services might be slower than others)
    let finviz_timeout = Duration::from_secs(10);  // Finviz can be slow due to scraping
    let reddit_timeout = Duration::from_secs(12);  // Reddit API can be slow
    let alpaca_timeout = Duration::from_secs(8);   // Alpaca is usually fast
    
    // Execute all three API calls in parallel with individual timeouts
    let (finviz_result, reddit_result, alpaca_result) = tokio::try_join!(
        timeout(finviz_timeout, fetch_finviz_news(None)),
        timeout(reddit_timeout, get_reddit_news()),
        timeout(alpaca_timeout, get_alpaca_news())
    ).map_err(|e| format!("Failed to execute news requests: {}", e))?;

    // Handle individual results with specific error handling
    let finviz_news = match finviz_result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Finviz news error: {}", e);
            Value::Null
        }
    };

    let reddit_news = match reddit_result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Reddit news error: {}", e);
            Value::Null
        }
    };

    let alpaca_news = match alpaca_result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Alpaca news error: {}", e);
            Value::Null
        }
    };

    Ok(json!({ 
        "finviz": finviz_news, 
        "reddit": reddit_news, 
        "alpaca": alpaca_news 
    }))
}

// Cached version that avoids redundant API calls
pub async fn get_news_cached() -> Result<Value, String> {
    let mut cache = NEWS_CACHE.lock().await;
    
    // Check if we have cached data
    if let Some(cached_data) = cache.get("combined_news") {
        return Ok(cached_data);
    }
    
    // Clear expired entries
    cache.clear_expired();
    
    // Fetch fresh data
    let fresh_data = get_news().await?;
    
    // Cache the result
    cache.set("combined_news".to_string(), fresh_data.clone());
    
    Ok(fresh_data)
}

// Alternative implementation with more granular error handling and retry logic
pub async fn get_news_with_retry(max_retries: u32) -> Result<Value, String> {
    let mut attempts = 0;
    
    while attempts < max_retries {
        match get_news().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(format!("Failed to fetch news after {} attempts: {}", max_retries, e));
                }
                // Exponential backoff
                let delay = Duration::from_millis(100 * 2_u64.pow(attempts - 1));
                tokio::time::sleep(delay).await;
            }
        }
    }
    
    Err("Unexpected error in retry loop".to_string())
}

// Force refresh cache (useful for testing or manual cache invalidation)
pub async fn refresh_news_cache() -> Result<Value, String> {
    let mut cache = NEWS_CACHE.lock().await;
    cache.data.remove("combined_news");
    drop(cache); // Release lock before calling get_news_cached
    
    get_news_cached().await
}

// Benchmark function to compare performance
pub async fn benchmark_news_performance() -> Result<String, String> {
    use std::time::Instant;
    
    // Test sequential approach (original)
    let start = Instant::now();
    let _sequential_result = get_news_sequential().await?;
    let sequential_duration = start.elapsed();
    
    // Test parallel approach (optimized)
    let start = Instant::now();
    let _parallel_result = get_news().await?;
    let parallel_duration = start.elapsed();
    
    // Test cached approach
    let start = Instant::now();
    let _cached_result = get_news_cached().await?;
    let cached_duration = start.elapsed();
    
    Ok(format!(
        "Performance Benchmark Results:\n\
         Sequential: {:?}\n\
         Parallel:   {:?}\n\
         Cached:     {:?}\n\
         Speedup:    {:.2}x faster with parallel execution\n\
         Cache hit:  {:.2}x faster with caching",
        sequential_duration,
        parallel_duration,
        cached_duration,
        sequential_duration.as_millis() as f64 / parallel_duration.as_millis() as f64,
        sequential_duration.as_millis() as f64 / cached_duration.as_millis() as f64
    ))
}

// Original sequential implementation for benchmarking
async fn get_news_sequential() -> Result<Value, String> {
    // Get Finviz News (no-arg lib function)
    let finviz_news = fetch_finviz_news(None).await.unwrap_or(Value::Null);
    // Get Reddit News
    let reddit_news = get_reddit_news().await.unwrap_or(Value::Null);
    // Get Alpaca News
    let alpaca_news = get_alpaca_news().await.unwrap_or(Value::Null);
    Ok(json!({ "finviz": finviz_news, "reddit": reddit_news, "alpaca": alpaca_news }))
}
