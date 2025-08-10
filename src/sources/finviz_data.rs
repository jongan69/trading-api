use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use std::collections::HashSet;
use crate::types::LimitQuery;
use crate::errors::ApiError;
use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
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
use futures::future::join_all;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use utoipa::{ToSchema, IntoParams};

pub async fn get_trending_from_finviz() -> Vec<String> {
    let mut trending_stocks: HashSet<String> = HashSet::new();

    // Common headers
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    if let Ok(resp) = client
        .get("https://finviz.com/screener.ashx?v=111&s=ta_topgainers&f=sh_price_u5,sh_avgvol_o500")
        .send()
        .await
    {
        if let Ok(body) = resp.text().await {
            let document = Html::parse_document(&body);
            let selector = Selector::parse("a[href*='quote.ashx']").unwrap();
            for element in document.select(&selector) {
                let ticker = element.text().collect::<String>().trim().to_string();
                if !ticker.is_empty() && ticker.len() <= 5 {
                    trending_stocks.insert(ticker);
                }
            }
        }
    }

    trending_stocks.into_iter().collect()
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


/// Library function: fetch Finviz news as JSON without requiring axum extractors
pub async fn fetch_finviz_news(limit: Option<usize>) -> Result<Value, String> {
    match News::default().scrape().await {
        Ok(result) => {
            let headers: Vec<String> = News::default_header().into_iter().map(|s| s.to_string()).collect();
            let news = map_rows_to_objects(headers.clone(), result.news, limit);
            let blogs = map_rows_to_objects(headers, result.blogs, limit);
            Ok(json!({ "news": news, "blogs": blogs }))
        }
        Err(err) => Err(format!("failed to fetch news: {err}")),
    }
}

#[utoipa::path(get, path = "/forex", params(LimitQuery), tag = "data", responses((status = 200, description = "Forex performance")))]
pub async fn get_forex(Query(query): Query<LimitQuery>) -> Result<impl IntoResponse, ApiError> {
    let LimitQuery { limit } = query;
    match Forex::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Forex::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            Ok((StatusCode::OK, Json(json!({ "data": data }))))
        }
        Err(err) => Err(ApiError::Upstream(format!("failed to fetch forex: {err}"))),
    }
}

#[utoipa::path(get, path = "/crypto", params(LimitQuery), tag = "data", responses((status = 200, description = "Crypto performance")))]
pub async fn get_crypto(Query(query): Query<LimitQuery>) -> Result<impl IntoResponse, ApiError> {
    let LimitQuery { limit } = query;
    match Crypto::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Crypto::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            Ok((StatusCode::OK, Json(json!({ "data": data }))))
        }
        Err(err) => Err(ApiError::Upstream(format!("failed to fetch crypto: {err}"))),
    }
}

#[utoipa::path(get, path = "/future", params(LimitQuery), tag = "data", responses((status = 200, description = "Futures performance")))]
pub async fn get_future(Query(query): Query<LimitQuery>) -> Result<impl IntoResponse, ApiError> {
    let LimitQuery { limit } = query;
    match Future::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Future::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            Ok((StatusCode::OK, Json(json!({ "data": data }))))
        }
        Err(err) => Err(ApiError::Upstream(format!("failed to fetch future: {err}"))),
    }
}

#[utoipa::path(get, path = "/insider", params(LimitQuery), tag = "data", responses((status = 200, description = "Insider transactions")))]
pub async fn get_insider(Query(query): Query<LimitQuery>) -> Result<impl IntoResponse, ApiError> {
    let LimitQuery { limit } = query;
    match Insider::default().scrape().await {
        Ok(rows) => {
            let headers: Vec<String> = Insider::default_header().into_iter().map(|s| s.to_string()).collect();
            let data = map_rows_to_objects(headers, rows, limit);
            Ok((StatusCode::OK, Json(json!({ "data": data }))))
        }
        Err(err) => Err(ApiError::Upstream(format!("failed to fetch insider: {err}"))),
    }
}

#[utoipa::path(get, path = "/group", params(LimitQuery), tag = "data", responses((status = 200, description = "Group/Industry")))]
pub async fn get_group(Query(query): Query<LimitQuery>) -> Result<impl IntoResponse, ApiError> {
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
            Ok((StatusCode::OK, Json(json!({ "data": data }))))
        }
        Err(err) => Err(ApiError::Upstream(format!("failed to fetch group: {err}"))),
    }
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ScreenerQuery {
    pub signal: Option<String>,   // e.g., TopGainers, TopLosers, NewHigh
    pub order: Option<String>,    // e.g., Price, MarketCap
    pub screener: Option<String>, // e.g., Performance, Financial
    pub limit: Option<usize>,
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
pub async fn get_screener_candidates(Query(q): Query<ScreenerQuery>) -> impl IntoResponse {
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
                .filter_map(|row| row.first().cloned())
                .collect();
            (StatusCode::OK, Json(json!({ "symbols": symbols })) ).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::types::ErrorResponse { error: format!("failed to scrape finviz screener: {err}") }),
        )
            .into_response(),
    }
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct FinvizRecommendationsQuery {
    pub signal: Option<String>,
    pub order: Option<String>,
    pub screener: Option<String>,
    pub limit: Option<usize>,
    // Yahoo metrics params
    pub range: Option<String>,
    pub interval: Option<String>,
    pub rf_annual: Option<f64>,
    pub target_return_annual: Option<f64>,
    pub periods_per_year: Option<usize>,
}

#[utoipa::path(get, path = "/recommendations/finviz", params(FinvizRecommendationsQuery), tag = "data", responses((status = 200, description = "Evaluate candidates & rank")))]
pub async fn get_recommendations_finviz(Query(q): Query<FinvizRecommendationsQuery>) -> impl IntoResponse {
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
                Json(crate::types::ErrorResponse { error: format!("screener error: {err}") }),
            )
                .into_response();
        }
    };

    let symbols: Vec<String> = rows.into_iter().filter_map(|row| row.first().cloned()).take(limit).collect();
    if symbols.is_empty() {
        return (StatusCode::OK, Json(json!({ "results": [] }))).into_response();
    }

    let futures_vec = symbols.iter().map(|sym| async move {
        match crate::services::yahoo::fetch_prices_for_symbol_default(sym, period_label).await {
            Ok(prices) => {
                let returns = crate::helpers::metrics::compute_returns_from_prices(&prices);
                let m = crate::helpers::metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, None);
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

pub async fn fetch_finviz_symbols(signal: &str, order: &str, screener: &str, symbols_limit: usize) -> Result<Vec<String>, String> {
    let mut s = Screener::new(parse_screener(screener));
    s.set_signal(parse_signal(signal));
    s.set_order(parse_order(order));
    match s.scrape().await {
        Ok(rows) => {
            let mut symbols: Vec<String> = Vec::new();
            for row in rows.into_iter().take(symbols_limit) {
                if let Some(t) = row.first() {
                    if t != "Ticker" && !t.is_empty() {
                        symbols.push(t.clone());
                    }
                }
            }
            Ok(symbols)
        }
        Err(err) => Err(format!("failed to source symbols from finviz: {err}")),
    }
}


