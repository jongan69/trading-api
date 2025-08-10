use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use crate::types::{OptionContract, OptionPrices, HighOpenInterestResult};

/// Helper function to fetch current option prices for a specific contract
async fn get_option_prices(contract: &OptionContract) -> Result<OptionPrices, String> {
    let normalized_symbol = if contract.symbol.contains("FB") {
        contract.symbol.replace("FB", "META")
    } else {
        contract.symbol.clone()
    };

    let key = std::env::var("ALPACA_API_KEY_ID")
        .or_else(|_| std::env::var("APCA_API_KEY_ID"))
        .map_err(|_| "ALPACA_API_KEY_ID/APCA_API_KEY_ID missing".to_string())?;
    
    let secret = std::env::var("ALPACA_API_SECRET_KEY")
        .or_else(|_| std::env::var("APCA_API_SECRET_KEY"))
        .map_err(|_| "ALPACA_API_SECRET_KEY/APCA_API_SECRET_KEY missing".to_string())?;

    let url = format!("https://api.alpaca.markets/v2/options/contracts/{}", normalized_symbol);
    
    let client = Client::new();
    let response = client.get(&url)
        .header("Apca-Api-Key-Id", key)
        .header("Apca-Api-Secret-Key", secret)
        .send()
        .await
        .map_err(|e| format!("Error fetching option prices: {}", e))?;

    if !response.status().is_success() {
        if response.status().as_u16() == 429 {
            return Err("Rate limit hit for option prices".to_string());
        }
        return Err(format!("Error fetching option prices: {} {}", response.status(), response.status().canonical_reason().unwrap_or("")));
    }

    let data: Value = response.json().await
        .map_err(|e| format!("Error parsing option prices JSON: {}", e))?;

    // Check if we have valid price data
    let close_price = data.get("close_price")
        .and_then(|v| v.as_f64())
        .ok_or("No close price found for option")?;

    let open_interest = data.get("open_interest")
        .and_then(|v| v.as_u64());

    let open_interest_date = data.get("open_interest_date")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let close_price_date = data.get("close_price_date")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(OptionPrices {
        ask_price: close_price,
        bid_price: close_price,
        last_price: close_price,
        implied_volatility: 0.0, // We don't have IV in the response
        open_interest,
        open_interest_date,
        close_price_date,
    })
}

/// Fetch contracts for a specific expiration range
async fn fetch_contracts(
    ticker: &str,
    option_type: &str,
    expiration_start: &str,
    expiration_end: &str,
) -> Result<Option<OptionContract>, String> {
    let normalized_ticker = if ticker == "FB" { "META" } else { ticker };

    let key = std::env::var("ALPACA_API_KEY_ID")
        .or_else(|_| std::env::var("APCA_API_KEY_ID"))
        .map_err(|_| "ALPACA_API_KEY_ID/APCA_API_KEY_ID missing".to_string())?;
    
    let secret = std::env::var("ALPACA_API_SECRET_KEY")
        .or_else(|_| std::env::var("APCA_API_SECRET_KEY"))
        .map_err(|_| "ALPACA_API_SECRET_KEY/APCA_API_SECRET_KEY missing".to_string())?;

    let url = format!(
        "https://api.alpaca.markets/v2/options/contracts?underlying_symbol={}&status=active&expiration_date_gte={}&expiration_date_lte={}&type={}&limit=100",
        normalized_ticker, expiration_start, expiration_end, option_type
    );

    let client = Client::new();
    let response = client.get(&url)
        .header("Apca-Api-Key-Id", key)
        .header("Apca-Api-Secret-Key", secret)
        .send()
        .await
        .map_err(|e| format!("Network error fetching contracts for {}: {}", ticker, e))?;

    if !response.status().is_success() {
        if response.status().as_u16() == 422 {
            let error_message = if normalized_ticker == "META" {
                format!("Invalid ticker symbol: {} (Note: Meta's ticker changed from FB to META in June 2022)", ticker)
            } else {
                format!("Invalid ticker symbol: {}", ticker)
            };
            return Err(error_message);
        }
        if response.status().as_u16() == 429 {
            return Err(format!("Rate limit hit for {}", ticker));
        }
        return Err(format!("Error fetching contracts for {}: {} {}", ticker, response.status(), response.status().canonical_reason().unwrap_or("")));
    }

    let data: Value = response.json().await
        .map_err(|e| format!("Error parsing contracts JSON: {}", e))?;

    let contracts = data.get("option_contracts")
        .and_then(|v| v.as_array())
        .ok_or(format!("No {} contracts found for {}", option_type, ticker))?;

    if contracts.is_empty() {
        return Ok(None);
    }

    // Parse contracts and sort by open interest
    let mut parsed_contracts: Vec<OptionContract> = Vec::new();
    for contract_value in contracts {
        if let Ok(contract) = serde_json::from_value::<OptionContract>(contract_value.clone()) {
            parsed_contracts.push(contract);
        }
    }

    if parsed_contracts.is_empty() {
        return Ok(None);
    }

    // Sort by open interest (descending) and get the highest one
    parsed_contracts.sort_by(|a, b| {
        let a_oi = a.open_interest.unwrap_or(0);
        let b_oi = b.open_interest.unwrap_or(0);
        b_oi.cmp(&a_oi)
    });

    let mut best_contract = parsed_contracts[0].clone();

    // Fetch current prices for the contract
    match get_option_prices(&best_contract).await {
        Ok(prices) => {
            // Merge the price data with the contract data
            best_contract.ask_price = Some(prices.ask_price);
            best_contract.bid_price = Some(prices.bid_price);
            best_contract.last_price = Some(prices.last_price);
            best_contract.implied_volatility = Some(prices.implied_volatility);
            best_contract.close_price = Some(prices.last_price);
            best_contract.close_price_date = prices.close_price_date;
        }
        Err(e) => {
            eprintln!("Failed to get price data for contract {}: {}", best_contract.symbol, e);
        }
    }

    Ok(Some(best_contract))
}

/// Fetch high open-interest contracts for a given ticker
pub async fn get_high_open_interest_contracts(
    ticker: &str,
    option_type: Option<&str>,
) -> HighOpenInterestResult {
    let option_type = option_type.unwrap_or("call");
    
    // Calculate date ranges
    let now = Utc::now();
    let short_term_start = (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let short_term_end = (now + chrono::Duration::days(60)).format("%Y-%m-%d").to_string();
    let leap_start = (now + chrono::Duration::days(365)).format("%Y-%m-%d").to_string();
    let leap_end = (now + chrono::Duration::days(730)).format("%Y-%m-%d").to_string();

    // Fetch short-term contracts
    let short_term_result = fetch_contracts(ticker, option_type, &short_term_start, &short_term_end).await;
    let short_term = match short_term_result {
        Ok(contract) => contract,
        Err(e) => {
            eprintln!("Error fetching short-term contracts for {}: {}", ticker, e);
            None
        }
    };

    // Fetch leap contracts
    let leap_result = fetch_contracts(ticker, option_type, &leap_start, &leap_end).await;
    let leap = match leap_result {
        Ok(contract) => contract,
        Err(e) => {
            eprintln!("Error fetching leap contracts for {}: {}", ticker, e);
            None
        }
    };

    HighOpenInterestResult {
        short_term,
        leap,
        error: None,
    }
}

/// Fetch high open-interest contracts for multiple tickers
pub async fn get_high_open_interest_contracts_batch(
    tickers: &[String],
    option_type: Option<&str>,
) -> Vec<(String, HighOpenInterestResult)> {
    let mut results = Vec::new();
    
    for ticker in tickers {
        let result = get_high_open_interest_contracts(ticker, option_type).await;
        results.push((ticker.clone(), result));
        
        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    results
}
