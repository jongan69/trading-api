use trading_api::helpers::trending_options::get_trending_options_analysis;
use trading_api::helpers::metrics::CompositeWeights;
use trading_api::state::AppState;
use yahoo_finance_api::YahooConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    println!("=== Trending Options Analysis Example ===\n");

    // Load configuration
    let config = trading_api::config::Config::from_env()
        .unwrap_or_default();

    // Create app state
    let cache = std::sync::Arc::new(trading_api::cache::MemoryCache::new());
    let state = AppState {
        http: reqwest::Client::new(),
        yahoo: std::sync::Arc::new(YahooConnector::new()?),
        concurrency_options: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
        config: std::sync::Arc::new(config),
        cache: cache.clone(),
        rate_limiter: std::sync::Arc::new(trading_api::middleware::RateLimiter::new(trading_api::middleware::RateLimitConfig::default())),
        optimized_client: trading_api::optimized_client::OptimizedApiClient::new(cache)?,
    };

    // Configure analysis parameters
    let rf_annual = 0.03;
    let periods_per_year = 252; // Daily data
    let weights = CompositeWeights {
        sharpe: 0.4,
        sortino: 0.4,
        calmar: 0.2,
    };
    let option_type = Some("call");
    let limit = 5;

    println!("1. Getting trending options analysis for call options:");
    let results = get_trending_options_analysis(
        &state,
        rf_annual,
        periods_per_year,
        &weights,
        option_type,
        limit,
    ).await;

    println!("Found {} trending tickers with options analysis\n", results.len());

    for (i, result) in results.iter().enumerate() {
        let symbol = result.get("symbol").and_then(|s| s.as_str()).unwrap_or("Unknown");
        let spot_price = result
            .get("underlying_metrics")
            .and_then(|m| m.get("spot_price"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        
        let composite_score = result
            .get("underlying_metrics")
            .and_then(|m| m.get("metrics"))
            .and_then(|m| m.get("composite_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);

        println!("{}. {} (Spot: ${:.2}, Score: {:.3})", i + 1, symbol, spot_price, composite_score);

        // Show options analysis
        if let Some(options_analysis) = result.get("options_analysis").and_then(|oa| oa.as_array()) {
            for option in options_analysis {
                let contract_type = option.get("contract_type").and_then(|ct| ct.as_str()).unwrap_or("Unknown");
                let option_score = option.get("option_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                
                if let Some(contract) = option.get("contract") {
                    let strike = contract.get("strike_price").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let expiration = contract.get("expiration_date").and_then(|e| e.as_str()).unwrap_or("Unknown");
                    let open_interest = contract.get("open_interest").and_then(|oi| oi.as_u64()).unwrap_or(0);
                    let last_price = contract.get("last_price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    
                    println!("   {contract_type}: Strike ${strike:.2}, Exp: {expiration}, OI: {open_interest}, Price: ${last_price:.2}, Score: {option_score:.3}");
                }

                // Show undervalued indicators
                if let Some(indicators) = option.get("undervalued_indicators") {
                    let overall_score = indicators.get("overall_undervalued_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let liquidity_score = indicators.get("liquidity_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let spread_score = indicators.get("spread_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let is_liquid = indicators.get("is_liquid").and_then(|l| l.as_bool()).unwrap_or(false);
                    let is_tight_spread = indicators.get("is_tight_spread").and_then(|s| s.as_bool()).unwrap_or(false);
                    
                    println!("      Undervalued Score: {overall_score:.3} (Liquidity: {liquidity_score:.3}, Spread: {spread_score:.3}, Liquid: {is_liquid}, Tight Spread: {is_tight_spread})");
                }
            }
        }
        println!();
    }

    println!("2. Getting trending options analysis for put options:");
    let put_results = get_trending_options_analysis(
        &state,
        rf_annual,
        periods_per_year,
        &weights,
        Some("put"),
        3,
    ).await;

    println!("Found {} trending tickers with put options analysis\n", put_results.len());

    for (i, result) in put_results.iter().enumerate() {
        let symbol = result.get("symbol").and_then(|s| s.as_str()).unwrap_or("Unknown");
        let composite_score = result
            .get("underlying_metrics")
            .and_then(|m| m.get("metrics"))
            .and_then(|m| m.get("composite_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);

        println!("{}. {} (Score: {:.3})", i + 1, symbol, composite_score);

        if let Some(options_analysis) = result.get("options_analysis").and_then(|oa| oa.as_array()) {
            for option in options_analysis {
                let contract_type = option.get("contract_type").and_then(|ct| ct.as_str()).unwrap_or("Unknown");
                let option_score = option.get("option_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                
                if let Some(contract) = option.get("contract") {
                    let strike = contract.get("strike_price").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let expiration = contract.get("expiration_date").and_then(|e| e.as_str()).unwrap_or("Unknown");
                    let open_interest = contract.get("open_interest").and_then(|oi| oi.as_u64()).unwrap_or(0);
                    
                    println!("   {contract_type}: Strike ${strike:.2}, Exp: {expiration}, OI: {open_interest}, Score: {option_score:.3}");
                }
            }
        }
        println!();
    }

    println!("=== Example completed ===");
    Ok(())
}
