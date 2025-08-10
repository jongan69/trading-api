use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use futures::future::join_all;
use serde_json::{json, Value};

use crate::helpers::params::{parse_symbols_csv, periods_per_year_from_interval};
use crate::services::yahoo::{fetch_prices_for_symbol_cached, metrics_for_prices};
use crate::state::AppState;
use crate::types::YahooQuery;
use crate::errors::ApiError;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/metrics/yahoo", get(get_metrics_yahoo))
        .route("/rank/yahoo", get(get_rank_yahoo))
        .route("/recommendations/yahoo", get(get_recommendations_yahoo))
        .with_state(state)
}

#[utoipa::path(get, path = "/metrics/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Metrics for single symbol")))]
pub async fn get_metrics_yahoo(axum::extract::State(state): axum::extract::State<AppState>, Query(q): Query<YahooQuery>) -> Result<impl IntoResponse, ApiError> {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q
        .periods_per_year
        .unwrap_or(periods_per_year_from_interval(interval));

    let symbols = parse_symbols_csv(&q.symbols.clone().unwrap_or_default());
    if symbols.len() != 1 {
        return Err(ApiError::BadRequest("provide exactly one symbol in symbols".to_string()));
    }
    let symbol = &symbols[0];
    let prices = fetch_prices_for_symbol_cached(&state.yahoo, symbol, period_label, &state.cache)
        .await
        .map_err(ApiError::Upstream)?;
    let m = metrics_for_prices(&prices, rf_annual, target_annual, periods_per_year, None);
    Ok((StatusCode::OK, Json(json!({ "symbol": symbol, "metrics": m }))))
}

#[utoipa::path(get, path = "/rank/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Rank multiple symbols")))]
pub async fn get_rank_yahoo(axum::extract::State(state): axum::extract::State<AppState>, Query(q): Query<YahooQuery>) -> Result<impl IntoResponse, ApiError> {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q
        .periods_per_year
        .unwrap_or(periods_per_year_from_interval(interval));

    let symbols = parse_symbols_csv(&q.symbols.clone().unwrap_or_default());
    if symbols.is_empty() {
        return Err(ApiError::BadRequest("symbols query param required".to_string()));
    }

    let yahoo = state.yahoo.clone();
    let cache = state.cache.clone();
    let futures_vec = symbols.iter().map(move |sym| {
        let yahoo = yahoo.clone();
        let cache = cache.clone();
        let sym = sym.to_string();
        async move {
        match fetch_prices_for_symbol_cached(&yahoo, &sym, period_label, &cache).await {
            Ok(prices) => {
                let m = metrics_for_prices(&prices, rf_annual, target_annual, periods_per_year, None);
                json!({ "symbol": sym, "metrics": m })
            }
            Err(err) => json!({ "symbol": sym, "error": err }),
        }
    }});

    let mut results: Vec<Value> = join_all(futures_vec).await;
    results.sort_by(|a, b| {
        let sa = a.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((StatusCode::OK, Json(json!({ "results": results }))))
}

#[utoipa::path(get, path = "/recommendations/yahoo", params(YahooQuery), tag = "data", responses((status = 200, description = "Recommendations for symbols")))]
pub async fn get_recommendations_yahoo(axum::extract::State(state): axum::extract::State<AppState>, Query(q): Query<YahooQuery>) -> Result<impl IntoResponse, ApiError> {
    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let rf_annual = q.rf_annual.unwrap_or(0.0);
    let target_annual = q.target_return_annual.unwrap_or(rf_annual);
    let periods_per_year = q
        .periods_per_year
        .unwrap_or(periods_per_year_from_interval(interval));

    let symbols = parse_symbols_csv(&q.symbols.clone().unwrap_or_default());
    if symbols.is_empty() {
        return Err(ApiError::BadRequest("symbols query param required".to_string()));
    }

    let yahoo = state.yahoo.clone();
    let cache = state.cache.clone();
    let futures_vec = symbols.iter().map(move |sym| {
        let yahoo = yahoo.clone();
        let cache = cache.clone();
        let sym = sym.to_string();
        async move {
        match fetch_prices_for_symbol_cached(&yahoo, &sym, period_label, &cache).await {
            Ok(prices) => {
                let m = metrics_for_prices(&prices, rf_annual, target_annual, periods_per_year, None);
                json!({ "symbol": sym, "metrics": m })
            }
            Err(err) => json!({ "symbol": sym, "error": err }),
        }
    }});

    let mut results: Vec<Value> = join_all(futures_vec).await;
    results.sort_by(|a, b| {
        let sa = a.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("metrics").and_then(|m| m.get("composite_score")).and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((StatusCode::OK, Json(json!({ "results": results }))))
}


