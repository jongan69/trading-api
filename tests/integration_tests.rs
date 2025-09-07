use trading_api::{build_app, state::AppState};
use tokio::task::JoinHandle;
use yahoo_finance_api::YahooConnector;
use reqwest::StatusCode;
use serde_json::Value;

async fn spawn_app() -> (String, JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let cache = std::sync::Arc::new(trading_api::cache::MemoryCache::new());
    let state = AppState {
        http: reqwest::Client::new(),
        yahoo: std::sync::Arc::new(YahooConnector::new().unwrap()),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
        config: std::sync::Arc::new(trading_api::config::Config::default()),
        cache: cache.clone(),
        rate_limiter: std::sync::Arc::new(trading_api::middleware::RateLimiter::new(trading_api::middleware::RateLimitConfig::default())),
        optimized_client: trading_api::optimized_client::OptimizedApiClient::new(cache).unwrap(),
    };
    let app = build_app(state).into_make_service();
    let h = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), h)
}

#[tokio::test]
async fn test_api_health_and_basic_functionality() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test health endpoint
    let response = client.get(&format!("{base}/health")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let json: Value = response.json().await.unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_error_handling() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test non-existent endpoint
    let response = client.get(&format!("{base}/nonexistent")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_yahoo_metrics_validation() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test with invalid symbol
    let response = client.get(&format!("{base}/metrics/yahoo?symbols=INVALID_SYMBOL_123")).send().await.unwrap();
    // Should handle gracefully - either return error or empty results
    assert!(response.status().is_success() || response.status().is_client_error());
    
    // Test with valid symbol
    let response = client.get(&format!("{base}/metrics/yahoo?symbols=AAPL&range=1mo&interval=1d")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let json: Value = response.json().await.unwrap();
    assert!(json.get("symbol").is_some());
    assert!(json.get("metrics").is_some());
}

#[tokio::test]
async fn test_coingecko_rate_limiting() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test CoinGecko endpoints can handle rate limiting gracefully
    let response = client.get(&format!("{base}/coingecko/top?limit=5")).send().await.unwrap();
    
    // Should either succeed or return proper rate limit error
    assert!(response.status().is_success() || response.status() == StatusCode::TOO_MANY_REQUESTS || response.status().is_server_error());
    
    if response.status().is_success() {
        let json: Value = response.json().await.unwrap();
        assert!(json.get("success").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("timestamp").is_some());
    }
}

#[tokio::test]
async fn test_kraken_endpoints_comprehensive() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test basic endpoints that should always work
    for endpoint in ["/kraken/status", "/kraken/time"] {
        let response = client.get(&format!("{base}{endpoint}")).send().await.unwrap();
        assert!(response.status().is_success(), "Failed endpoint: {endpoint}");
        
        let json: Value = response.json().await.unwrap();
        assert!(json.get("success").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("timestamp").is_some());
    }
    
    // Test data endpoints that might have network issues
    for endpoint in ["/kraken/ticker", "/kraken/assets", "/kraken/pairs"] {
        let response = client.get(&format!("{base}{endpoint}")).send().await.unwrap();
        // Allow success or server errors (API might be temporarily unavailable)
        assert!(response.status().is_success() || response.status().is_server_error(), 
                "Unexpected status for {endpoint}: {}", response.status());
    }
}

#[tokio::test]
async fn test_options_endpoint_validation() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test options endpoint with valid parameters
    let response = client.get(&format!("{base}/options/recommendations?symbols=AAPL&side=call&min_dte=7&max_dte=30&range=1mo&interval=1d&limit=5")).send().await.unwrap();
    assert!(response.status().is_success() || response.status().is_server_error());
    
    if response.status().is_success() {
        let json: Value = response.json().await.unwrap();
        if let Some(data) = json.as_array() {
            for item in data {
                assert!(item.get("symbol").is_some());
                assert!(item.get("contract").is_some());
                assert!(item.get("side").is_some());
            }
        }
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test multiple concurrent requests
    let mut handles = vec![];
    
    for _ in 0..5 {
        let base_clone = base.clone();
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let response = client_clone.get(&format!("{base_clone}/health")).send().await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_large_limit_parameters() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test with reasonable large limits
    let response = client.get(&format!("{base}/coingecko/top?limit=50")).send().await.unwrap();
    assert!(response.status().is_success() || response.status() == StatusCode::TOO_MANY_REQUESTS || response.status().is_server_error());
    
    // Test with very large limit (should be handled gracefully)
    let response = client.get(&format!("{base}/news?limit=1000")).send().await.unwrap();
    assert!(response.status().is_success() || response.status().is_server_error());
}

#[tokio::test]
async fn test_malformed_parameters() {
    if std::env::var("RUN_INTEGRATION").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Test with malformed parameters
    let response = client.get(&format!("{base}/metrics/yahoo?symbols=&range=invalid&interval=bad")).send().await.unwrap();
    assert!(response.status().is_client_error() || response.status().is_server_error());
    
    // Test with missing required parameters
    let response = client.get(&format!("{base}/metrics/yahoo")).send().await.unwrap();
    assert!(response.status().is_client_error() || response.status().is_server_error());
}
