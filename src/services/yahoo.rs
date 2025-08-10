use time::{Duration, OffsetDateTime};
use yahoo_finance_api::YahooConnector;
use serde_json::json;

use crate::helpers::metrics;
use crate::cache::{MemoryCache, cache_key};

pub async fn fetch_prices_for_symbol(
    provider: &YahooConnector,
    symbol: &str,
    range_label: &str,
) -> Result<Vec<f64>, String> {
    let now = OffsetDateTime::now_utc();
    let start = match range_label {
        "1mo" => now - Duration::days(30),
        "3mo" => now - Duration::days(90),
        "6mo" => now - Duration::days(180),
        "1y" => now - Duration::days(365),
        "2y" => now - Duration::days(730),
        "5y" => now - Duration::days(365 * 5),
        _ => now - Duration::days(30),
    };
    let resp = provider
        .get_quote_history(symbol, start, now)
        .await
        .map_err(|e| format!("yahoo history error: {e}"))?;
    let quotes = resp
        .quotes()
        .map_err(|e| format!("quotes parse error: {e}"))?;
    if quotes.len() < 2 {
        return Err("not enough quotes".to_string());
    }
    Ok(quotes.into_iter().map(|q| q.close).collect())
}

pub async fn fetch_prices_for_symbol_cached(
    provider: &YahooConnector,
    symbol: &str,
    range_label: &str,
    cache: &MemoryCache,
) -> Result<Vec<f64>, String> {
    let cache_key = cache_key("yahoo_prices", &[("symbol", symbol), ("range", range_label)]);
    
    if let Some(cached) = cache.get(&cache_key).await {
        if let Some(prices_array) = cached.as_array() {
            let prices: Result<Vec<f64>, _> = prices_array
                .iter()
                .map(|v| v.as_f64().ok_or("Invalid cached price data"))
                .collect();
            if let Ok(prices) = prices {
                return Ok(prices);
            }
        }
    }

    let prices = fetch_prices_for_symbol(provider, symbol, range_label).await?;
    
    let cache_ttl = match range_label {
        "1mo" | "3mo" => std::time::Duration::from_secs(300), // 5 minutes
        "6mo" | "1y" => std::time::Duration::from_secs(900), // 15 minutes
        _ => std::time::Duration::from_secs(1800), // 30 minutes
    };
    
    let cache_data = json!(prices);
    cache.set(cache_key, cache_data, cache_ttl).await;
    
    Ok(prices)
}

pub async fn latest_close(provider: &YahooConnector, symbol: &str) -> Result<f64, String> {
    let now = OffsetDateTime::now_utc();
    let start = now - Duration::days(10);
    let resp = provider
        .get_quote_history(symbol, start, now)
        .await
        .map_err(|e| format!("yahoo history error: {e}"))?;
    let quotes = resp
        .quotes()
        .map_err(|e| format!("quotes parse error: {e}"))?;
    quotes
        .last()
        .map(|q| q.close)
        .ok_or_else(|| "no quotes".to_string())
}

pub fn metrics_for_prices(
    prices: &[f64],
    rf_annual: f64,
    target_annual: f64,
    periods_per_year: usize,
    weights: Option<metrics::CompositeWeights>,
) -> metrics::MetricsResult {
    let returns = metrics::compute_returns_from_prices(prices);
    metrics::compute_metrics_from_returns(&returns, rf_annual, target_annual, periods_per_year, weights)
}

pub async fn fetch_prices_for_symbol_default(symbol: &str, range_label: &str) -> Result<Vec<f64>, String> {
    let provider = YahooConnector::new().map_err(|e| format!("yahoo connector error: {e}"))?;
    fetch_prices_for_symbol(&provider, symbol, range_label).await
}


