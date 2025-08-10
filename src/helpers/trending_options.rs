use futures::future::join_all;
use serde_json::Value;
use crate::helpers::metrics::{compute_metrics_from_returns, CompositeWeights};
use crate::helpers::high_open_interest::get_high_open_interest_contracts;
use crate::services::yahoo::{fetch_prices_for_symbol, latest_close};
use crate::sources;
use crate::state::AppState;
use crate::types::OptionContract;

/// Get trending stocks from multiple sources and deduplicate
pub async fn get_trending_tickers() -> Vec<String> {
    let mut all_tickers = Vec::new();
    
    // Get trending stocks from finviz
    if let Ok(finviz_tickers) = sources::finviz_data::fetch_finviz_symbols("TopGainers", "MarketCap", "Performance", 20).await {
        all_tickers.extend(finviz_tickers);
    }
    
    // Get trending stocks from yahoo
    if let Ok(yahoo_tickers) = sources::yahoo_data::yahoo_trending("US", 20).await {
        all_tickers.extend(yahoo_tickers);
    }
    
    // Get trending stocks from reddit
    let reddit_tickers = sources::reddit_data::get_reddit_trending_stocks().await;
    all_tickers.extend(reddit_tickers);
    
    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    all_tickers.retain(|s| seen.insert(s.clone()));
    
    all_tickers
}

/// Analyze underlying metrics for a ticker
pub async fn analyze_underlying_metrics(
    yahoo: &yahoo_finance_api::YahooConnector,
    symbol: &str,
    rf_annual: f64,
    periods_per_year: usize,
    weights: &CompositeWeights,
) -> Option<Value> {
    let period_label = "3mo";
    
    // Fetch price data
    let prices = match fetch_prices_for_symbol(yahoo, symbol, period_label).await {
        Ok(p) => p,
        Err(_) => return None,
    };
    
    if prices.len() < 10 {
        return None; // Need sufficient data for meaningful metrics
    }
    
    // Compute returns and metrics
    let returns = crate::helpers::metrics::compute_returns_from_prices(&prices);
    let metrics = compute_metrics_from_returns(
        &returns,
        rf_annual,
        rf_annual,
        periods_per_year,
        Some(weights.clone()),
    );
    
    // Get current spot price
    let spot = match latest_close(yahoo, symbol).await {
        Ok(s) => s,
        Err(_) => return None,
    };
    
    Some(serde_json::json!({
        "symbol": symbol,
        "spot_price": spot,
        "metrics": {
            "composite_score": metrics.composite_score,
            "sharpe": metrics.sharpe,
            "sortino": metrics.sortino,
            "calmar": metrics.calmar,
            "kelly_fraction": metrics.kelly_fraction,
            "volatility": metrics.volatility,
            "max_drawdown": metrics.max_drawdown,
            "cagr": metrics.cagr,
            "n_periods": metrics.n_periods
        }
    }))
}

/// Analyze options for a ticker using high open interest contracts
pub async fn analyze_ticker_options(
    symbol: &str,
    underlying_metrics: &Value,
    option_type: Option<&str>,
) -> Option<Value> {
    // Get high open interest contracts
    let hoi_result = get_high_open_interest_contracts(symbol, option_type).await;
    
    let spot_price = underlying_metrics.get("spot_price")?.as_f64()?;
    let composite_score = underlying_metrics.get("metrics")?.get("composite_score")?.as_f64()?;
    
    let mut options_analysis = Vec::new();
    
    // Analyze short-term contracts
    if let Some(contract) = hoi_result.short_term {
        let option_score = calculate_option_score(&contract, spot_price, composite_score);
        options_analysis.push(serde_json::json!({
            "contract_type": "short_term",
            "contract": contract,
            "option_score": option_score,
            "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
        }));
    }
    
    // Analyze LEAP contracts
    if let Some(contract) = hoi_result.leap {
        let option_score = calculate_option_score(&contract, spot_price, composite_score);
        options_analysis.push(serde_json::json!({
            "contract_type": "leap",
            "contract": contract,
            "option_score": option_score,
            "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
        }));
    }
    
    Some(serde_json::json!({
        "symbol": symbol,
        "underlying_metrics": underlying_metrics,
        "options_analysis": options_analysis,
        "error": hoi_result.error
    }))
}

/// Calculate option score based on underlying metrics and option characteristics
fn calculate_option_score(contract: &OptionContract, spot_price: f64, underlying_score: f64) -> f64 {
    let premium = contract.last_price.unwrap_or(0.0);
    if premium <= 0.0 {
        return 0.0;
    }
    
    let delta = contract.implied_volatility.unwrap_or(0.0); // Using IV as proxy for delta
    let _leverage = (delta.abs() * spot_price) / premium;
    
    // Base score from underlying
    let base_score = underlying_score;
    
    // Option-specific adjustments
    let option_score = base_score * delta * (spot_price / premium) / (1.0 + 30.0 / 30.0); // Assuming 30 DTE for short-term
    
    option_score
}

/// Calculate undervalued indicators for an option contract
fn calculate_undervalued_indicators(contract: &OptionContract, _spot_price: f64, underlying_score: f64) -> Value {
    let premium = contract.last_price.unwrap_or(0.0);
    let bid = contract.bid_price.unwrap_or(0.0);
    let ask = contract.ask_price.unwrap_or(0.0);
    let open_interest = contract.open_interest.unwrap_or(0);
    
    let mid_price = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { premium };
    let spread = if ask > 0.0 && bid > 0.0 { ask - bid } else { 0.0 };
    let spread_pct = if mid_price > 0.0 { spread / mid_price } else { f64::INFINITY };
    
    // Calculate various undervalued indicators
    let liquidity_score = if open_interest > 1000 { 1.0 } else if open_interest > 500 { 0.7 } else if open_interest > 100 { 0.4 } else { 0.1 };
    let spread_score = if spread_pct < 0.05 { 1.0 } else if spread_pct < 0.10 { 0.7 } else if spread_pct < 0.20 { 0.4 } else { 0.1 };
    let underlying_momentum = underlying_score; // Higher underlying score = better momentum
    
    let overall_undervalued_score = (liquidity_score * 0.3 + spread_score * 0.3 + underlying_momentum * 0.4).min(1.0);
    
    serde_json::json!({
        "liquidity_score": liquidity_score,
        "spread_score": spread_score,
        "underlying_momentum": underlying_momentum,
        "overall_undervalued_score": overall_undervalued_score,
        "spread_percentage": spread_pct,
        "open_interest": open_interest,
        "is_liquid": open_interest > 500,
        "is_tight_spread": spread_pct < 0.10,
        "has_momentum": underlying_score > 0.5
    })
}

/// Get trending tickers with options analysis
pub async fn get_trending_options_analysis(
    state: &AppState,
    rf_annual: f64,
    periods_per_year: usize,
    weights: &CompositeWeights,
    option_type: Option<&str>,
    limit: usize,
) -> Vec<Value> {
    // Get trending tickers
    let trending_tickers = get_trending_tickers().await;
    
    // Analyze each ticker
    let analysis_futures = trending_tickers.into_iter().map(|symbol| {
        let yahoo = state.yahoo.clone();
        let weights = weights.clone();
        let option_type = option_type.map(|s| s.to_string());
        
        async move {
            // Analyze underlying metrics
            let underlying_metrics = analyze_underlying_metrics(&yahoo, &symbol, rf_annual, periods_per_year, &weights).await;
            
            if let Some(metrics) = underlying_metrics {
                // Analyze options
                let options_analysis = analyze_ticker_options(&symbol, &metrics, option_type.as_deref()).await;
                
                if let Some(analysis) = options_analysis {
                    return Some(analysis);
                }
            }
            
            None
        }
    });
    
    let mut results: Vec<Value> = join_all(analysis_futures).await.into_iter().flatten().collect();
    
    // Sort by overall score (combination of underlying metrics and option analysis)
    results.sort_by(|a, b| {
        let a_score = a.get("underlying_metrics")
            .and_then(|m| m.get("metrics"))
            .and_then(|m| m.get("composite_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        
        let b_score = b.get("underlying_metrics")
            .and_then(|m| m.get("metrics"))
            .and_then(|m| m.get("composite_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        
        b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Apply limit
    if results.len() > limit {
        results.truncate(limit);
    }
    
    results
}