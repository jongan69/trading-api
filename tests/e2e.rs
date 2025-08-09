use trading_api::{build_app, state::AppState};
use tokio::task::JoinHandle;
use yahoo_finance_api::YahooConnector;

async fn spawn_app() -> (String, JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = AppState {
        http: reqwest::Client::new(),
        yahoo: std::sync::Arc::new(YahooConnector::new().unwrap()),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
    };
    let app = build_app(state).into_make_service();
    let h = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{}", addr), h)
}

#[tokio::test]
async fn health_ok() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{}/health", base)).await.unwrap();
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
        let res = reqwest::get(format!("{}{}", base, path)).await.unwrap();
        assert!(res.status().is_success(), "failed: {}", path);
    }
}

#[tokio::test]
async fn yahoo_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{}/metrics/yahoo?symbols=AAPL&range=1mo&interval=1d", base)).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{}/rank/yahoo?symbols=AAPL,MSFT&range=1mo&interval=1d", base)).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{}/recommendations/yahoo?symbols=AAPL,MSFT&range=1mo&interval=1d", base)).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn finviz_endpoints() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{}/screener/candidates?signal=TopGainers&order=Price&screener=Performance&limit=5", base)).await.unwrap();
    assert!(res.status().is_success());
    let res = reqwest::get(format!("{}/recommendations/finviz?signal=TopGainers&order=Price&screener=Performance&limit=10&range=1mo&interval=1d", base)).await.unwrap();
    assert!(res.status().is_success());
}

#[tokio::test]
async fn options_endpoint_yahoo_fallback() {
    if std::env::var("RUN_E2E").is_err() { return; }
    let (base, _h) = spawn_app().await;
    let res = reqwest::get(format!("{}/options/recommendations?symbols=AAPL&side=call&min_dte=7&max_dte=30&range=1mo&interval=1d&limit=5", base)).await.unwrap();
    assert!(res.status().is_success());
}


