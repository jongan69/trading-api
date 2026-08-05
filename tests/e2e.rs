mod common;

/// Deterministic: /health always returns 200.
#[tokio::test]
async fn health_ok() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/health")).await.unwrap();
    assert!(res.status().is_success());
}

/// Deterministic: 404 returns structured error.
#[tokio::test]
async fn not_found_is_structured_error() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/nonexistent")).await.unwrap();
    assert_eq!(res.status(), 404);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"], "NOT_FOUND");
}

/// Deterministic: pumpfun endpoints return 501.
#[tokio::test]
async fn pumpfun_returns_not_implemented() {
    let (base, _h) = common::spawn_app().await;
    for path in ["/pumpfun/trending", "/pumpfun/market-summary", "/pumpfun/search"] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert_eq!(res.status(), 501, "expected 501 for {path}");
    }
}

#[tokio::test]
#[ignore] // live upstreams
async fn data_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    for path in [
        "/news", "/forex?limit=2", "/crypto?limit=2", "/future?limit=2",
        "/insider?limit=2", "/group?limit=2", "/reddit/stocks?limit=2",
        "/trending/stocks?limit=2",
    ] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert!(res.status().is_success(), "failed: {path}");
    }
}

#[tokio::test]
#[ignore] // live upstreams
async fn yahoo_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/metrics/yahoo?symbols=AAPL&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{base}/rank/yahoo?symbols=AAPL,MSFT&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
#[ignore] // live upstreams
async fn finviz_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/screener/candidates?signal=TopGainers&order=Price&screener=Performance&limit=5")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
#[ignore] // live upstreams
async fn options_endpoint_live() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/options/recommendations?symbols=AAPL&side=call&min_dte=7&max_dte=30&range=1mo&interval=1d&limit=5")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
#[ignore] // live upstreams
async fn kraken_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    for path in ["/kraken/status", "/kraken/time", "/kraken/ticker", "/kraken/assets", "/kraken/pairs"] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert!(res.status().is_success(), "failed: {path} status {}", res.status());
    }
}

#[tokio::test]
#[ignore] // live upstreams
async fn coingecko_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/coingecko/top?limit=3")).await.unwrap();
    let json: serde_json::Value = res.json().await.unwrap();
    assert!(json.get("success").is_some());
    assert!(json.get("data").is_some());
}
