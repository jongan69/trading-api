mod metrics;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use finviz_rs::{
    common::Scrape,
    crypto::Crypto,
    forex::Forex,
    future::Future,
    group::{Group, GroupBy, GroupType, OrderBy, Ordering},
    insider::Insider,
    news::News,
    order_type::OrderType,
    screener::Screener,
    screener_type::ScreenerType,
    signal_type::SignalType,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use time::{Duration, OffsetDateTime};
use yahoo_finance_api::YahooConnector;
use futures::future::join_all;
use reqwest::Client;
use utoipa::{OpenApi, ToSchema, IntoParams};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
struct AppState;

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState;

    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .route("/health", get(health))
        .route("/news", get(get_news))
        .route("/forex", get(get_forex))
        .route("/crypto", get(get_crypto))
        .route("/future", get(get_future))
        .route("/insider", get(get_insider))
        .route("/group", get(get_group))
        .route("/metrics/yahoo", get(get_metrics_yahoo))
        .route("/rank/yahoo", get(get_rank_yahoo))
        .route("/recommendations/yahoo", get(get_recommendations_yahoo))
        .route("/screener/candidates", get(get_screener_candidates))
        .route("/recommendations/finviz", get(get_recommendations_finviz))
        .route("/options/recommendations", get(get_options_recommendations))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
// removed unused POST body types to keep GET-only API clean

#[derive(Deserialize, ToSchema, IntoParams)]
struct YahooQuery {
    symbols: Option<String>, // comma-separated symbols
    range: Option<String>,   // e.g., 1mo, 3mo, 6mo, 1y
    interval: Option<String>, // e.g., 1d, 1wk, 1mo
    rf_annual: Option<f64>,
    target_return_annual: Option<f64>,
    periods_per_year: Option<usize>,
}

async fn fetch_prices_for_symbol(symbol: &str, range: &str, _interval: &str) -> Result<Vec<f64>, String> {
    let now = OffsetDateTime::now_utc();
    let start = match range {
        "1mo" => now - Duration::days(30),
        "3mo" => now - Duration::days(90),
        "6mo" => now - Duration::days(180),
        "1y" => now - Duration::days(365),
        "2y" => now - Duration::days(730),
        "5y" => now - Duration::days(365 * 5),
        _ => now - Duration::days(30),
    };
    let provider = YahooConnector::new().map_err(|e| format!("yahoo connector error: {e}"))?;
    let resp = provider
        .get_quote_history(symbol, start, now)
        .await
        .map_err(|e| format!("yahoo history error: {e}"))?;
    let quotes = resp.quotes().map_err(|e| format!("quotes parse error: {e}"))?;
    if quotes.len() < 2 {
        return Err("not enough quotes".to_string());
    }
    Ok(quotes.into_iter().map(|q| q.close).collect())
}

#[utoipa::path(get, path = "/recommendations/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Recommendations for symbols")))]
async fn get_recommendations_yahoo(Query(q): Query<YahooQuery>) -> impl IntoResponse {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q.periods_per_year.unwrap_or(match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    });

    let symbols: Vec<String> = q
        .symbols
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    if symbols.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "symbols query param required".to_string() }),
        )
            .into_response();
    }

    let futures_vec = symbols.iter().map(|sym| async move {
        match fetch_prices_for_symbol(sym, period_label, interval).await {
            Ok(prices) => {
                let returns = metrics::compute_returns_from_prices(&prices);
                let m = metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, None);
                json!({ "symbol": sym, "metrics": m })
            }
            Err(err) => json!({ "symbol": sym, "error": err }),
        }
    });

    let mut results: Vec<Value> = join_all(futures_vec).await;
    results.sort_by(|a, b| {
        let sa = a.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    (StatusCode::OK, Json(json!({ "results": results }))).into_response()
}

#[utoipa::path(get, path = "/health", tag = "system", responses((status = 200, description = "Service health", body = HealthResponse)))]
async fn health() -> impl IntoResponse {
    let body = HealthResponse { status: "ok" };
    (StatusCode::OK, Json(body))
}

#[derive(Deserialize, ToSchema, IntoParams)]
struct LimitQuery {
    limit: Option<usize>,
}

fn map_rows_to_objects(headers: Vec<String>, mut rows: Vec<Vec<String>>, limit: Option<usize>) -> Vec<Value> {
    if let Some(max) = limit {
        if rows.len() > max {
            rows.truncate(max);
        }
    }
    rows
        .into_iter()
        .map(|row| {
            let mut obj = Map::new();
            for (idx, header) in headers.iter().enumerate() {
                if let Some(value) = row.get(idx) {
                    obj.insert(header.clone(), Value::String(value.clone()));
                }
            }
            Value::Object(obj)
        })
        .collect()
}

#[utoipa::path(get, path = "/news", params(LimitQuery), tag = "data", responses((status = 200, description = "Latest Finviz news")))]
async fn get_news(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    match News::default().scrape().await {
        Ok(result) => (
            StatusCode::OK,
            {
                let headers: Vec<String> = News::default_header().into_iter().map(|s| s.to_string()).collect();
                let news = map_rows_to_objects(headers.clone(), result.news, limit);
                let blogs = map_rows_to_objects(headers, result.blogs, limit);
                Json(json!({ "news": news, "blogs": blogs }))
            },
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to fetch news: {err}"),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/forex", params(LimitQuery), tag = "data", responses((status = 200, description = "Forex performance")))]
async fn get_forex(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    match Forex::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Forex::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            (StatusCode::OK, Json(json!({ "data": data })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to fetch forex: {err}") }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/crypto", params(LimitQuery), tag = "data", responses((status = 200, description = "Crypto performance")))]
async fn get_crypto(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    match Crypto::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Crypto::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            (StatusCode::OK, Json(json!({ "data": data })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to fetch crypto: {err}") }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/future", params(LimitQuery), tag = "data", responses((status = 200, description = "Futures performance")))]
async fn get_future(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    match Future::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Future::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            (StatusCode::OK, Json(json!({ "data": data })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to fetch future: {err}") }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/insider", params(LimitQuery), tag = "data", responses((status = 200, description = "Insider transactions")))]
async fn get_insider(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    match Insider::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Insider::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            (StatusCode::OK, Json(json!({ "data": data })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to fetch insider: {err}") }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/group", params(LimitQuery), tag = "data", responses((status = 200, description = "Group/Industry")))]
async fn get_group(_state: State<AppState>, Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    let group = Group::new(
        GroupBy::Industry,
        GroupType::Valuation,
        OrderBy::PerformanceWeek,
        Ordering::Ascending,
    );
    match group.scrape().await {
        Ok(rows) => {
            let headers = vec![
                "Name".to_string(),
                "Market Cap".to_string(),
                "P/E".to_string(),
                "Fwd P/E".to_string(),
                "PEG".to_string(),
                "P/S".to_string(),
                "P/B".to_string(),
                "P/C".to_string(),
                "P/FCF".to_string(),
                "EPS past 5Y".to_string(),
                "EPS next 5Y".to_string(),
                "Sales past 5Y".to_string(),
                "Change".to_string(),
                "Volume".to_string(),
            ];
            let data = map_rows_to_objects(headers, rows, limit);
            (StatusCode::OK, Json(json!({ "data": data })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to fetch group: {err}") }),
        )
            .into_response(),
    }
}

// ---- Yahoo metrics and rank (GET) ----
#[utoipa::path(get, path = "/metrics/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Metrics for single symbol")))]
async fn get_metrics_yahoo(Query(q): Query<YahooQuery>) -> impl IntoResponse {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q.periods_per_year.unwrap_or(match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    });

    let symbols: Vec<String> = q
        .symbols
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    if symbols.len() != 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "provide exactly one symbol in symbols".to_string() }),
        )
            .into_response();
    }

    let symbol = &symbols[0];
    match fetch_prices_for_symbol(symbol, period_label, interval).await {
        Ok(prices) => {
            let returns = metrics::compute_returns_from_prices(&prices);
            let m = metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, None);
            (StatusCode::OK, Json(json!({ "symbol": symbol, "metrics": m })) ).into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: err }),
        )
            .into_response(),
    }
}

#[utoipa::path(get, path = "/rank/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Rank multiple symbols")))]
async fn get_rank_yahoo(Query(q): Query<YahooQuery>) -> impl IntoResponse {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q.periods_per_year.unwrap_or(match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    });

    let symbols: Vec<String> = q
        .symbols
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    if symbols.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "symbols query param required".to_string() }),
        )
            .into_response();
    }

    let futures_vec = symbols.iter().map(|sym| async move {
        match fetch_prices_for_symbol(sym, period_label, interval).await {
            Ok(prices) => {
                let returns = metrics::compute_returns_from_prices(&prices);
                let m = metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, None);
                json!({ "symbol": sym, "metrics": m })
            }
            Err(err) => json!({ "symbol": sym, "error": err }),
        }
    });

    let mut results: Vec<Value> = join_all(futures_vec).await;
    results.sort_by(|a, b| {
        let sa = a.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    (StatusCode::OK, Json(json!({ "results": results }))).into_response()
}

// ---- Finviz screener candidates and recommendations ----
#[derive(Deserialize, ToSchema, IntoParams)]
struct ScreenerQuery {
    signal: Option<String>,   // e.g., TopGainers, TopLosers, NewHigh
    order: Option<String>,    // e.g., Price, MarketCap
    screener: Option<String>, // e.g., Performance, Financial
    limit: Option<usize>,
}

fn parse_signal(s: &str) -> SignalType {
    match s {
        "TopGainers" => SignalType::TopGainers,
        "TopLosers" => SignalType::TopLosers,
        "NewHigh" => SignalType::NewHigh,
        "NewLow" => SignalType::NewLow,
        _ => SignalType::TopGainers,
    }
}

fn parse_order(s: &str) -> OrderType {
    match s {
        "Price" => OrderType::Price,
        "MarketCap" => OrderType::MarketCap,
        "Change" => OrderType::Change,
        _ => OrderType::Price,
    }
}

fn parse_screener(s: &str) -> ScreenerType {
    match s {
        "Financial" => ScreenerType::Financial,
        "Ownership" => ScreenerType::Ownership,
        _ => ScreenerType::Performance,
    }
}

#[utoipa::path(get, path = "/screener/candidates", params(ScreenerQuery), tag = "data", responses((status = 200, description = "Finviz screener candidates")))]
async fn get_screener_candidates(Query(q): Query<ScreenerQuery>) -> impl IntoResponse {
    let signal = q.signal.as_deref().unwrap_or("TopGainers");
    let order = q.order.as_deref().unwrap_or("Price");
    let screener = q.screener.as_deref().unwrap_or("Performance");
    let limit = q.limit.unwrap_or(25);

    let mut s = Screener::new(parse_screener(screener));
    s.set_signal(parse_signal(signal));
    s.set_order(parse_order(order));

    match s.scrape().await {
        Ok(rows) => {
            let symbols: Vec<String> = rows
                .into_iter()
                .take(limit)
                .filter_map(|row| row.get(0).cloned())
                .collect();
            (StatusCode::OK, Json(json!({ "symbols": symbols })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("failed to scrape finviz screener: {err}") }),
        )
            .into_response(),
    }
}

#[derive(Deserialize, ToSchema, IntoParams)]
struct FinvizRecommendationsQuery {
    signal: Option<String>,
    order: Option<String>,
    screener: Option<String>,
    limit: Option<usize>,
    // Yahoo metrics params
    range: Option<String>,
    interval: Option<String>,
    rf_annual: Option<f64>,
    target_return_annual: Option<f64>,
    periods_per_year: Option<usize>,
}

#[utoipa::path(get, path = "/recommendations/finviz", params(FinvizRecommendationsQuery), tag = "data", responses((status = 200, description = "Evaluate candidates & rank")))]
async fn get_recommendations_finviz(Query(q): Query<FinvizRecommendationsQuery>) -> impl IntoResponse {
    let signal = q.signal.as_deref().unwrap_or("TopGainers");
    let order = q.order.as_deref().unwrap_or("Price");
    let screener = q.screener.as_deref().unwrap_or("Performance");
    let limit = q.limit.unwrap_or(25);

    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q.periods_per_year.unwrap_or(match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    });

    let mut s = Screener::new(parse_screener(screener));
    s.set_signal(parse_signal(signal));
    s.set_order(parse_order(order));

    let rows = match s.scrape().await {
        Ok(r) => r,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("screener error: {err}") }),
            )
                .into_response();
        }
    };

    let symbols: Vec<String> = rows.into_iter().filter_map(|row| row.get(0).cloned()).take(limit).collect();
    if symbols.is_empty() {
        return (StatusCode::OK, Json(json!({ "results": [] }))).into_response();
    }

    let futures_vec = symbols.iter().map(|sym| async move {
        match fetch_prices_for_symbol(sym, period_label, interval).await {
            Ok(prices) => {
                let returns = metrics::compute_returns_from_prices(&prices);
                let m = metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, None);
                json!({ "symbol": sym, "metrics": m })
            }
            Err(err) => json!({ "symbol": sym, "error": err }),
        }
    });

    let mut results: Vec<Value> = join_all(futures_vec).await;
    results.sort_by(|a, b| {
        let sa = a.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    (StatusCode::OK, Json(json!({ "results": results }))).into_response()
}

// ---- Yahoo options recommendations ----
#[derive(Deserialize, ToSchema, IntoParams)]
struct OptionsQuery {
    symbol: Option<String>,
    symbols: Option<String>,
    side: Option<String>, // call | put | both
    min_dte: Option<u32>,
    max_dte: Option<u32>,
    limit: Option<usize>,
    rf_annual: Option<f64>,
    // underlying metrics configuration
    range: Option<String>,      // e.g., 1mo, 3mo, 6mo
    interval: Option<String>,   // e.g., 1d, 1wk
    sharpe_w: Option<f64>,
    sortino_w: Option<f64>,
    calmar_w: Option<f64>,
    // option filters
    min_delta: Option<f64>,
    max_delta: Option<f64>,
    min_premium: Option<f64>,
    max_premium: Option<f64>,
    min_volume: Option<i64>,
    min_oi: Option<i64>,
    min_strike_ratio: Option<f64>, // strike/spot
    max_strike_ratio: Option<f64>, // strike/spot
    // Finviz sourcing when symbols omitted
    signal: Option<String>,
    order: Option<String>,
    screener: Option<String>,
    symbols_limit: Option<usize>,
    // additional ranking/filters
    per_symbol_limit: Option<usize>,
    max_spread_pct: Option<f64>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        get_news,
        get_forex,
        get_crypto,
        get_future,
        get_insider,
        get_group,
        get_metrics_yahoo,
        get_rank_yahoo,
        get_recommendations_yahoo,
        get_screener_candidates,
        get_recommendations_finviz,
        get_options_recommendations,
    ),
    components(schemas(
        HealthResponse,
        ErrorResponse,
        LimitQuery,
        YahooQuery,
        ScreenerQuery,
        FinvizRecommendationsQuery,
        OptionsQuery,
    )),
    tags(
        (name = "system", description = "Health & meta"),
        (name = "data", description = "Market data from Finviz & Yahoo"),
        (name = "options", description = "Options recommendations")
    )
)]
struct ApiDoc;

fn black_scholes_delta(spot: f64, strike: f64, r: f64, sigma: f64, t_years: f64, is_call: bool) -> Option<f64> {
    if spot <= 0.0 || strike <= 0.0 || sigma <= 0.0 || t_years <= 0.0 {
        return None;
    }
    let d1 = ((spot / strike).ln() + (r + 0.5 * sigma * sigma) * t_years) / (sigma * t_years.sqrt());
    let norm = statrs::distribution::Normal::new(0.0, 1.0).ok()?;
    let nd1 = statrs::distribution::ContinuousCDF::cdf(&norm, d1);
    if is_call {
        Some(nd1)
    } else {
        Some(nd1 - 1.0)
    }
}

async fn yahoo_latest_close(symbol: &str) -> Result<f64, String> {
    let now = OffsetDateTime::now_utc();
    let start = now - Duration::days(10);
    let provider = YahooConnector::new().map_err(|e| format!("yahoo connector error: {e}"))?;
    let resp = provider
        .get_quote_history(symbol, start, now)
        .await
        .map_err(|e| format!("yahoo history error: {e}"))?;
    let quotes = resp.quotes().map_err(|e| format!("quotes parse error: {e}"))?;
    quotes.last().map(|q| q.close).ok_or_else(|| "no quotes".to_string())
}

async fn fetch_options_chain(symbol: &str) -> Result<Value, String> {
    let url = format!("https://query2.finance.yahoo.com/v7/finance/options/{}", symbol);
    let client = Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("reqwest error: {e}"))?;
    let v: Value = resp.json().await.map_err(|e| format!("json error: {e}"))?;
    Ok(v)
}

#[utoipa::path(get, path = "/options/recommendations", params(OptionsQuery), tag = "options", responses((status = 200, description = "Rank options contracts")))]
async fn get_options_recommendations(Query(q): Query<OptionsQuery>) -> impl IntoResponse {
    let side = q.side.unwrap_or_else(|| "both".to_string());
    let min_dte = q.min_dte.unwrap_or(7);
    let max_dte = q.max_dte.unwrap_or(60);
    let limit = q.limit.unwrap_or(20);
    let rf_annual = q.rf_annual.unwrap_or(0.03);

    // underlying metrics params
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let periods_per_year = match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    };

    // composite weights
    let w_sharpe = q.sharpe_w.unwrap_or(0.4);
    let w_sortino = q.sortino_w.unwrap_or(0.4);
    let w_calmar = q.calmar_w.unwrap_or(0.2);

    // Build symbols set
    let mut symbols: Vec<String> = vec![];
    if let Some(list) = q.symbols.as_ref() {
        for s in list.split(',') {
            let t = s.trim();
            if !t.is_empty() { symbols.push(t.to_string()); }
        }
    }
    if let Some(sym) = q.symbol.as_ref() {
        if !sym.is_empty() { symbols.push(sym.clone()); }
    }
    if symbols.is_empty() {
        // source from finviz
        let signal = q.signal.as_deref().unwrap_or("TopGainers");
        let order = q.order.as_deref().unwrap_or("Price");
        let screener = q.screener.as_deref().unwrap_or("Performance");
        let symbols_limit = q.symbols_limit.unwrap_or(20);
        let mut s = Screener::new(parse_screener(screener));
        s.set_signal(parse_signal(signal));
        s.set_order(parse_order(order));
        match s.scrape().await {
            Ok(rows) => {
                for row in rows.into_iter().take(symbols_limit) {
                    if let Some(t) = row.get(0) { symbols.push(t.clone()); }
                }
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: format!("failed to source symbols from finviz: {err}") }),
                ).into_response();
            }
        }
    }
    if symbols.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "no symbols available".to_string() }),
        ).into_response();
    }

    // carry filters into async map
    let min_premium = q.min_premium;
    let max_premium = q.max_premium;
    let min_volume = q.min_volume;
    let min_oi = q.min_oi;
    let min_delta = q.min_delta;
    let max_delta = q.max_delta;
    let min_sr = q.min_strike_ratio;
    let max_sr = q.max_strike_ratio;
    let per_symbol_limit = q.per_symbol_limit.unwrap_or(usize::MAX);
    let max_spread_pct = q.max_spread_pct;

    let tasks = symbols.into_iter().map(|symbol| {
        let side = side.clone();
        async move {
            let spot = match yahoo_latest_close(&symbol).await { Ok(s) => s, Err(_) => return Vec::new() };
            let prices = match fetch_prices_for_symbol(&symbol, period_label, interval).await { Ok(p) => p, Err(_) => return Vec::new() };
            let returns = metrics::compute_returns_from_prices(&prices);
            let under_metrics = metrics::compute_metrics_from_returns(
                &returns,
                rf_annual,
                rf_annual,
                periods_per_year,
                Some(metrics::CompositeWeights { sharpe: w_sharpe, sortino: w_sortino, calmar: w_calmar }),
            );
            let base_score = under_metrics.composite_score;

            let chain = match fetch_options_chain(&symbol).await { Ok(v) => v, Err(_) => return Vec::new() };
            let now_ts = OffsetDateTime::now_utc().unix_timestamp();
            let mut out: Vec<Value> = Vec::new();
            if let Some(result) = chain.get("optionChain").and_then(|c| c.get("result")).and_then(|r| r.get(0)) {
                if let Some(options_arr) = result.get("options").and_then(|o| o.as_array()).and_then(|arr| arr.get(0)) {
                    let mut process_contracts = |contracts: &Vec<Value>, is_call: bool| {
                        for c in contracts {
                            let strike = c.get("strike").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let bid = c.get("bid").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let ask = c.get("ask").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let last = c.get("lastPrice").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let iv = c.get("impliedVolatility").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let exp_ts = c.get("expiration").and_then(|v| v.as_i64()).unwrap_or(0);
                            let contract_symbol = c.get("contractSymbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let volume = c.get("volume").and_then(|v| v.as_i64()).unwrap_or(0);
                            let open_interest = c.get("openInterest").and_then(|v| v.as_i64()).unwrap_or(0);
                            if strike <= 0.0 || exp_ts <= 0 { continue; }
                            let dte_days = ((exp_ts - now_ts) as f64 / 86_400.0).max(0.0);
                            if dte_days < min_dte as f64 || dte_days > max_dte as f64 { continue; }
                            let premium = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { last };
                            if premium <= 0.0 { continue; }
                            let mid = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { premium };
                            let spread = if ask > 0.0 && bid > 0.0 { ask - bid } else { 0.0 };
                            let spread_pct = if mid > 0.0 { spread / mid } else { f64::INFINITY };
                            if let Some(max_sp) = max_spread_pct { if spread_pct.is_finite() && spread_pct > max_sp { continue; } }
                            if let Some(min_p) = min_premium { if premium < min_p { continue; } }
                            if let Some(max_p) = max_premium { if premium > max_p { continue; } }
                            if let Some(min_v) = min_volume { if volume < min_v { continue; } }
                            if let Some(min_o) = min_oi { if open_interest < min_o { continue; } }
                            let t_years = dte_days / 365.0;
                            let sigma = iv.abs();
                            let delta = match black_scholes_delta(spot, strike, rf_annual, sigma, t_years, is_call) { Some(d) => d, None => continue };
                            if let Some(min_d) = min_delta { if delta < min_d { continue; } }
                            if let Some(max_d) = max_delta { if delta > max_d { continue; } }
                            let strike_ratio = strike / spot;
                            if let Some(lo) = min_sr { if strike_ratio < lo { continue; } }
                            if let Some(hi) = max_sr { if strike_ratio > hi { continue; } }
                            let leverage = (delta.abs() * spot) / premium;
                            let score = base_score * delta * (spot / premium) / (1.0 + dte_days / 30.0);
                            out.push(json!({
                                "symbol": symbol,
                                "contract": contract_symbol,
                                "side": if is_call { "call" } else { "put" },
                                "strike": strike,
                                "expiration": exp_ts,
                                "dte_days": dte_days,
                                "premium": premium,
                                "mid": mid,
                                "spread": spread,
                                "spread_pct": spread_pct,
                                "implied_vol": sigma,
                                "delta": delta,
                                "leverage": leverage,
                                "volume": volume,
                                "open_interest": open_interest,
                                "strike_ratio": strike_ratio,
                                "score": score,
                                "underlying_metrics": under_metrics,
                            }));
                        }
                    };
                    if side == "call" || side == "both" {
                        if let Some(calls) = options_arr.get("calls").and_then(|v| v.as_array()) {
                            process_contracts(calls, true);
                        }
                    }
                    if side == "put" || side == "both" {
                        if let Some(puts) = options_arr.get("puts").and_then(|v| v.as_array()) {
                            process_contracts(puts, false);
                        }
                    }
                }
            }
            // per-symbol capping by score
            out.sort_by(|a, b| {
                let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            if out.len() > per_symbol_limit { out.truncate(per_symbol_limit); }
            out
        }
    });

    let mut options_list: Vec<Value> = join_all(tasks).await.into_iter().flatten().collect();
    options_list.sort_by(|a, b| {
        let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    options_list.truncate(limit);

    (StatusCode::OK, Json(json!({ "results": options_list }))).into_response()
}