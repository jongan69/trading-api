use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::sources::pumpfun_data::{
    BondingCurveInfo, TokenInfo,
};
use crate::errors::ApiError;
use crate::state::AppState;
use crate::types::TrendingItem;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct PumpFunQuery {
    pub limit: Option<u32>,
    pub sort_by: Option<String>, // market_cap, volume, created_at
    pub time_range: Option<String>, // 1h, 24h, 7d, 30d
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunTrendingResponse {
    pub trending_tokens: Vec<TrendingItem>,
    pub total_tokens: u32,
    pub market_summary: PumpFunMarketSummary,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunMarketSummary {
    pub total_market_cap: Option<f64>,
    pub total_volume_24h: Option<f64>,
    pub new_tokens_24h: u32,
    pub top_gainers_24h: Vec<TokenInfo>,
    pub top_losers_24h: Vec<TokenInfo>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/trending", get(get_trending_pumpfun))
        .route("/token/{mint_address}", get(get_token_info))
        .route("/bonding-curve/{mint_address}", get(get_bonding_curve))
        .route("/market-summary", get(get_market_summary))
        .route("/new-tokens", get(get_new_tokens))
        .route("/top-gainers", get(get_top_gainers))
        .route("/top-losers", get(get_top_losers))
        .route("/search/{query}", get(search_tokens))
        .with_state(state)
}

/// Get trending pump.fun tokens
#[utoipa::path(
    get,
    path = "/pumpfun/trending",
    params(
        ("limit" = Option<u32>, Query, description = "Number of trending tokens to return (default: 20)"),
        ("sort_by" = Option<String>, Query, description = "Sort by: market_cap, volume, created_at (default: market_cap)"),
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 24h, 7d, 30d (default: 24h)")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Trending pump.fun tokens", body = PumpFunResponse<PumpFunTrendingResponse>))
)]
pub async fn get_trending_pumpfun(
    State(_state): State<AppState>,
    Query(params): Query<PumpFunQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20) as usize;
    let _sort_by = params.sort_by.unwrap_or_else(|| "market_cap".to_string());
    let _time_range = params.time_range.unwrap_or_else(|| "24h".to_string());

    // For now, return mock trending data - in a real implementation, you would:
    // 1. Use pump.fun's API or on-chain data to get trending tokens
    // 2. Calculate market metrics and rankings
    // 3. Sort by the specified criteria
    
    let mock_trending = generate_mock_trending_tokens(limit);
    
    let response = PumpFunResponse {
        success: true,
        data: PumpFunTrendingResponse {
            trending_tokens: mock_trending,
            total_tokens: 1000, // Mock total
            market_summary: PumpFunMarketSummary {
                total_market_cap: Some(50_000_000.0),
                total_volume_24h: Some(10_000_000.0),
                new_tokens_24h: 150,
                top_gainers_24h: vec![],
                top_losers_24h: vec![],
            },
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get detailed information about a specific token
#[utoipa::path(
    get,
    path = "/pumpfun/token/{mint_address}",
    params(
        ("mint_address" = String, Path, description = "Token mint address")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Token information", body = PumpFunResponse<TokenInfo>))
)]
pub async fn get_token_info(
    State(_state): State<AppState>,
    Path(mint_address): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate mint address format
    if mint_address.len() != 44 {
        return Err(ApiError::InvalidInput("Invalid mint address format".to_string()));
    }

    // Mock token info - in a real implementation, fetch from pump.fun API or on-chain data
    let token_info = TokenInfo {
        mint_address: mint_address.clone(),
        name: "Example Meme Token".to_string(),
        symbol: "EMT".to_string(),
        description: "A trendy meme token on pump.fun".to_string(),
        image_url: Some("https://example.com/token.png".to_string()),
        creator: "11111111111111111111111111111112".to_string(),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        market_cap: Some(100_000.0),
        price: Some(0.001),
        volume_24h: Some(50_000.0),
    };

    let response = PumpFunResponse {
        success: true,
        data: token_info,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get bonding curve information for a token
#[utoipa::path(
    get,
    path = "/pumpfun/bonding-curve/{mint_address}",
    params(
        ("mint_address" = String, Path, description = "Token mint address")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Bonding curve information", body = PumpFunResponse<BondingCurveInfo>))
)]
pub async fn get_bonding_curve(
    State(_state): State<AppState>,
    Path(mint_address): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate mint address format
    if mint_address.len() != 44 {
        return Err(ApiError::InvalidInput("Invalid mint address format".to_string()));
    }

    // This would use the PumpFunService to get actual bonding curve data
    // For now, return mock data
    let bonding_curve = BondingCurveInfo {
        mint_address: mint_address.clone(),
        virtual_token_reserves: 1_000_000_000_000,
        virtual_sol_reserves: 30_000_000_000,
        real_token_reserves: 800_000_000_000,
        real_sol_reserves: 20_000_000_000,
        token_total_supply: 1_000_000_000_000,
        complete: false,
    };

    let response = PumpFunResponse {
        success: true,
        data: bonding_curve,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get market summary for pump.fun
#[utoipa::path(
    get,
    path = "/pumpfun/market-summary",
    tag = "pumpfun",
    responses((status = 200, description = "Market summary", body = PumpFunResponse<PumpFunMarketSummary>))
)]
pub async fn get_market_summary(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let summary = PumpFunMarketSummary {
        total_market_cap: Some(50_000_000.0),
        total_volume_24h: Some(10_000_000.0),
        new_tokens_24h: 150,
        top_gainers_24h: generate_mock_tokens(5),
        top_losers_24h: generate_mock_tokens(5),
    };

    let response = PumpFunResponse {
        success: true,
        data: summary,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get newly created tokens
#[utoipa::path(
    get,
    path = "/pumpfun/new-tokens",
    params(
        ("limit" = Option<u32>, Query, description = "Number of new tokens to return (default: 20)")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "New tokens", body = PumpFunResponse<Vec<TokenInfo>>))
)]
pub async fn get_new_tokens(
    State(_state): State<AppState>,
    Query(params): Query<PumpFunQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20) as usize;
    let tokens = generate_mock_tokens(limit);

    let response = PumpFunResponse {
        success: true,
        data: tokens,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get top gaining tokens
#[utoipa::path(
    get,
    path = "/pumpfun/top-gainers",
    params(
        ("limit" = Option<u32>, Query, description = "Number of top gainers to return (default: 20)")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Top gaining tokens", body = PumpFunResponse<Vec<TokenInfo>>))
)]
pub async fn get_top_gainers(
    State(_state): State<AppState>,
    Query(params): Query<PumpFunQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20) as usize;
    let tokens = generate_mock_tokens(limit);

    let response = PumpFunResponse {
        success: true,
        data: tokens,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get top losing tokens
#[utoipa::path(
    get,
    path = "/pumpfun/top-losers",
    params(
        ("limit" = Option<u32>, Query, description = "Number of top losers to return (default: 20)")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Top losing tokens", body = PumpFunResponse<Vec<TokenInfo>>))
)]
pub async fn get_top_losers(
    State(_state): State<AppState>,
    Query(params): Query<PumpFunQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20) as usize;
    let tokens = generate_mock_tokens(limit);

    let response = PumpFunResponse {
        success: true,
        data: tokens,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Search for tokens by name or symbol
#[utoipa::path(
    get,
    path = "/pumpfun/search/{query}",
    params(
        ("query" = String, Path, description = "Search query"),
        ("limit" = Option<u32>, Query, description = "Number of results to return (default: 20)")
    ),
    tag = "pumpfun",
    responses((status = 200, description = "Search results", body = PumpFunResponse<Vec<TokenInfo>>))
)]
pub async fn search_tokens(
    State(_state): State<AppState>,
    Path(query): Path<String>,
    Query(params): Query<PumpFunQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20) as usize;
    
    // Mock search results - in a real implementation, search through token database
    let tokens = generate_mock_search_results(&query, limit);

    let response = PumpFunResponse {
        success: true,
        data: tokens,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source: "pumpfun".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// Helper functions for mock data generation
fn generate_mock_trending_tokens(limit: usize) -> Vec<TrendingItem> {
    let mock_tokens = vec![
        ("PEPE2", "Pepe 2.0", 0.00045, 15.2),
        ("DOGE3", "Doge 3.0", 0.0012, 8.7),
        ("MOON", "To The Moon", 0.00089, 22.1),
        ("ROCKET", "Rocket Ship", 0.0034, 12.4),
        ("MEME", "Ultimate Meme", 0.0067, 18.9),
    ];

    mock_tokens
        .into_iter()
        .take(limit)
        .enumerate()
        .map(|(i, (symbol, name, price, change))| TrendingItem {
            id: format!("{symbol}111111111111111111111111111111{i:02}"),
            symbol: symbol.to_string(),
            name: name.to_string(),
            price: Some(price),
            price_change_24h: Some(change),
            price_change_percentage_24h: Some(change),
            volume: Some(100000.0 + i as f64 * 50000.0),
            market_cap: Some(price * 1_000_000_000.0),
            market_cap_rank: Some(i as u32 + 1),
            score: Some(100.0 - i as f64 * 5.0),
            source: "pumpfun".to_string(),
            image_url: Some(format!("https://example.com/{}.png", symbol.to_lowercase())),
            last_updated: Some(chrono::Utc::now().timestamp().to_string()),
        })
        .collect()
}

fn generate_mock_tokens(limit: usize) -> Vec<TokenInfo> {
    let mock_tokens = vec![
        ("PEPE2", "Pepe 2.0"),
        ("DOGE3", "Doge 3.0"),
        ("MOON", "To The Moon"),
        ("ROCKET", "Rocket Ship"),
        ("MEME", "Ultimate Meme"),
    ];

    mock_tokens
        .into_iter()
        .cycle()
        .take(limit)
        .enumerate()
        .map(|(i, (symbol, name))| TokenInfo {
            mint_address: format!("{symbol}111111111111111111111111111111{i:02}"),
            name: name.to_string(),
            symbol: symbol.to_string(),
            description: format!("A trendy {name} token on pump.fun"),
            image_url: Some(format!("https://example.com/{}.png", symbol.to_lowercase())),
            creator: "11111111111111111111111111111112".to_string(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            market_cap: Some(100_000.0 + i as f64 * 10_000.0),
            price: Some(0.001 + i as f64 * 0.0001),
            volume_24h: Some(50_000.0 + i as f64 * 5_000.0),
        })
        .collect()
}

fn generate_mock_search_results(query: &str, limit: usize) -> Vec<TokenInfo> {
    // Mock search - in reality, this would search through a database
    let mut results = generate_mock_tokens(limit);
    
    // Modify names to include the search query
    for token in results.iter_mut() {
        token.name = format!("{query} {}", token.name);
        token.symbol = format!("{}_{}", query.to_uppercase(), token.symbol);
    }
    
    results
}
