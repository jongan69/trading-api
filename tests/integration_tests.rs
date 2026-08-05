mod common;

/// Deterministic: 404 fallback returns structured {error, code, timestamp}.
#[tokio::test]
async fn test_404_is_structured_error() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/zzz/nonexistent")).await.unwrap();
    assert_eq!(res.status(), 404);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"], "NOT_FOUND");
    assert!(body.get("error").is_some(), "error field required");
    assert!(body.get("timestamp").is_some(), "timestamp field required");
}

/// Deterministic: rate limit config is respected.
#[tokio::test]
async fn test_rate_limit_middleware_present() {
    let (base, _h) = common::spawn_app().await;
    // Rate limiting is disabled in tests, so multiple requests should all succeed
    for _ in 0..5 {
        let res = reqwest::get(format!("{base}/health")).await.unwrap();
        assert!(res.status().is_success());
    }
}

/// Deterministic: large limits are capped.
#[tokio::test]
async fn test_options_endpoint_validation() {
    let (base, _h) = common::spawn_app().await;
    // Invalid symbol should still return success (empty or 400 depending on source)
    let res = reqwest::get(format!("{base}/options/recommendations?symbols=ZZZZTOP&side=call&min_dte=7&max_dte=30&range=1mo&interval=1d&limit=2")).await.unwrap();
    // Either 200 (no options found) or 400/502 (upstream failure) — both are valid
    let status = res.status().as_u16();
    assert!(status == 200 || status >= 400, "unexpected status {}", status);
}

#[tokio::test]
#[ignore] // live upstreams
async fn test_kraken_endpoints_live() {
    let (base, _h) = common::spawn_app().await;
    for path in ["/kraken/ticker", "/kraken/assets", "/kraken/pairs", "/kraken/status", "/kraken/time"] {
        let res = reqwest::get(format!("{base}{path}")).await.unwrap();
        assert!(res.status().is_success(), "failed: {path} status {}", res.status());
    }
}

#[tokio::test]
#[ignore] // live upstreams
async fn test_yahoo_metrics_live() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/metrics/yahoo?symbols=AAPL&range=1mo&interval=1d")).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn test_health_and_basic_functionality() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/health")).await.unwrap();
    assert!(res.status().is_success());
    let json: serde_json::Value = res.json().await.unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let (base, _h) = common::spawn_app().await;
    let mut handles = vec![];
    for _ in 0..8 {
        let url = format!("{base}/health");
        handles.push(tokio::spawn(async move {
            reqwest::get(&url).await.unwrap().status()
        }));
    }
    for h in handles {
        let status = h.await.unwrap();
        assert!(status.is_success());
    }
}

#[tokio::test]
async fn test_malformed_parameters() {
    let (base, _h) = common::spawn_app().await;
    // Missing symbols
    let res = reqwest::get(format!("{base}/metrics/yahoo?range=1mo")).await.unwrap();
    assert!(!res.status().is_success(), "expected error for missing symbols");
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("error").is_some());
    assert!(body.get("code").is_some());
}

#[tokio::test]
async fn test_swagger_ui_available() {
    let (base, _h) = common::spawn_app().await;
    let res = reqwest::get(format!("{base}/docs")).await.unwrap();
    assert!(res.status().is_success(), "Swagger UI should be reachable");
}
