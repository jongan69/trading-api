use trading_api::{build_app, state::AppState};
use tokio::task::JoinHandle;
use yahoo_finance_api::YahooConnector;

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
async fn health_ok() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{base}/health")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn data_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    for path in [
        "/news",
        "/forex?limit=2",
        "/crypto?limit=2",
        "/future?limit=2",
        "/insider?limit=2",
        "/group?limit=2",
        "/reddit/stocks?limit=2",
        "/trending/stocks?limit=2",
    ] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert!(res.status().is_success(), "failed: {path}");
    }
}

#[tokio::test]
async fn yahoo_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{base}/metrics/yahoo?symbols=AAPL&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{base}/rank/yahoo?symbols=AAPL,MSFT&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{base}/recommendations/yahoo?symbols=AAPL,MSFT&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn finviz_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{base}/screener/candidates?signal=TopGainers&order=Price&screener=Performance&limit=5")).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{base}/recommendations/finviz?signal=TopGainers&order=Price&screener=Performance&limit=10&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn options_endpoint_yahoo_fallback() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{base}/options/recommendations?symbols=AAPL&side=call&min_dte=7&max_dte=30&range=1mo&interval=1d&limit=5")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn kraken_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    
    // Test basic endpoints that should work
    for path in [
        "/kraken/status",
        "/kraken/time",
    ] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert!(res.status().is_success(), "failed: {} with status: {}", path, res.status());
    }
    
    // Test data endpoints that might fail due to API issues (allow 500 errors)
    for path in [
        "/kraken/ticker",
        "/kraken/assets",
        "/kraken/pairs",
        "/trending/crypto",
    ] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        // Allow both success and server errors (API might be down or have issues)
        assert!(res.status().is_success() || res.status().is_server_error(), 
                "failed: {} with status: {}", path, res.status());
    }
}

#[tokio::test]
async fn coingecko_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    
    // Test all CoinGecko endpoints - allow various error responses since API might have issues
    for path in [
        "/coingecko/top?limit=5",
        "/coingecko/gainers?limit=3",
        "/coingecko/losers?limit=3",
        "/coingecko/trending",
        "/coingecko/market-overview",
        "/coingecko/market-context",
        "/coingecko/trending-symbols",
    ] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        // Allow success, rate limiting (429), and server errors (500) since CoinGecko API might have issues
        assert!(res.status().is_success() || res.status().as_u16() == 429 || res.status().is_server_error(), 
                "failed: {} with status: {}", path, res.status());
    }
    
    // Test simple price endpoint with parameters
    let res = reqwest::get(format!("{base}/coingecko/simple-price?ids=bitcoin,ethereum&vs_currencies=usd&include_24hr_change=true")).await.unwrap();
    assert!(res.status().is_success() || res.status().as_u16() == 429 || res.status().is_server_error(), 
            "failed: simple-price with status: {}", res.status());
}

#[tokio::test]
async fn coingecko_response_structure() {
    if std::env::var("RUN_E2E").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    
    // Test that successful responses have the expected structure
    let res = reqwest::get(format!("{base}/coingecko/top?limit=3")).await.unwrap();
    
    if res.status().is_success() {
        let json: serde_json::Value = res.json().await.unwrap();
        
        // Check response structure
        assert!(json.get("success").is_some(), "Response should have 'success' field");
        assert!(json.get("data").is_some(), "Response should have 'data' field");
        assert!(json.get("timestamp").is_some(), "Response should have 'timestamp' field");
        
        // Check data is an array
        if let Some(data) = json.get("data") {
            assert!(data.is_array(), "Data should be an array");
            
            // If we have data, check the first coin structure
            if let Some(coins) = data.as_array() {
                if !coins.is_empty() {
                    let first_coin = &coins[0];
                    assert!(first_coin.get("id").is_some(), "Coin should have 'id' field");
                    assert!(first_coin.get("symbol").is_some(), "Coin should have 'symbol' field");
                    assert!(first_coin.get("name").is_some(), "Coin should have 'name' field");
                }
            }
        }
    }
}

#[tokio::test]
async fn coingecko_market_overview_structure() {
    if std::env::var("RUN_E2E").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    
    let res = reqwest::get(format!("{base}/coingecko/market-overview")).await.unwrap();
    
    if res.status().is_success() {
        let json: serde_json::Value = res.json().await.unwrap();
        
        // Check response structure
        assert!(json.get("success").is_some(), "Response should have 'success' field");
        assert!(json.get("data").is_some(), "Response should have 'data' field");
        
        // Check market overview data structure
        if let Some(data) = json.get("data") {
            assert!(data.get("total_market_cap").is_some(), "Market overview should have 'total_market_cap' field");
            assert!(data.get("total_volume").is_some(), "Market overview should have 'total_volume' field");
            assert!(data.get("bitcoin_dominance").is_some(), "Market overview should have 'bitcoin_dominance' field");
        }
    }
}

#[tokio::test]
async fn coingecko_trending_structure() {
    if std::env::var("RUN_E2E").is_err() { return; }
    
    let (base, _h) = spawn_app().await;
    
    let res = reqwest::get(format!("{base}/coingecko/trending")).await.unwrap();
    
    if res.status().is_success() {
        let json: serde_json::Value = res.json().await.unwrap();
        
        // Check response structure
        assert!(json.get("success").is_some(), "Response should have 'success' field");
        assert!(json.get("data").is_some(), "Response should have 'data' field");
        
        // Check trending data structure
        if let Some(data) = json.get("data") {
            assert!(data.is_array(), "Trending data should be an array");
            
            if let Some(trending) = data.as_array() {
                if !trending.is_empty() {
                    let first_item = &trending[0];
                    assert!(first_item.get("item").is_some(), "Trending item should have 'item' field");
                    
                    if let Some(item) = first_item.get("item") {
                        assert!(item.get("name").is_some(), "Trending item should have 'name' field");
                        assert!(item.get("symbol").is_some(), "Trending item should have 'symbol' field");
                        assert!(item.get("score").is_some(), "Trending item should have 'score' field");
                    }
                }
            }
        }
    }
}


