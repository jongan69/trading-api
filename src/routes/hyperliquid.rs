use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::sources::hyperliquid_data::{
    HyperliquidDataSource, HyperliquidMarket, HyperliquidOrderbook, HyperliquidCandle,
    HyperliquidUserState, HyperliquidFunding, HyperliquidTrade,
};
use crate::errors::ApiError;
use crate::state::AppState;
use crate::types::TrendingItem;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct HyperliquidQuery {
    pub limit: Option<u32>,
    pub depth: Option<u32>,
    pub interval: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HyperliquidResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketMoversResponse {
    pub gainers: Vec<HyperliquidMarket>,
    pub losers: Vec<HyperliquidMarket>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/markets", get(get_all_markets))
        .route("/market/{coin}", get(get_market_data))
        .route("/orderbook/{coin}", get(get_orderbook))
        .route("/trades/{coin}", get(get_recent_trades))
        .route("/candles/{coin}", get(get_candles))
        .route("/user/{address}", get(get_user_state))
        .route("/funding", get(get_funding_rates))
        .route("/trending", get(get_trending_defi))
        .route("/volume/top", get(get_top_volume))
        .route("/movers", get(get_top_movers))
        .route("/stats/overview", get(get_market_overview))
        .with_state(state)
}

/// Get all available markets
#[utoipa::path(
    get,
    path = "/hyperliquid/markets",
    tag = "hyperliquid",
    responses((status = 200, description = "All available markets", body = HyperliquidResponse<Vec<HyperliquidMarket>>))
)]
pub async fn get_all_markets(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let markets = hyperliquid.get_all_markets().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: markets,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get market data for a specific coin
#[utoipa::path(
    get,
    path = "/hyperliquid/market/{coin}",
    params(
        ("coin" = String, Path, description = "Coin symbol (e.g., BTC, ETH)")
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Market data for coin", body = HyperliquidResponse<HyperliquidMarket>))
)]
pub async fn get_market_data(
    State(_state): State<AppState>,
    Path(coin): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let market = hyperliquid.get_market_data(&coin).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: market,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get orderbook for a specific coin
#[utoipa::path(
    get,
    path = "/hyperliquid/orderbook/{coin}",
    params(
        ("coin" = String, Path, description = "Coin symbol"),
        HyperliquidQuery
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Orderbook data", body = HyperliquidResponse<HyperliquidOrderbook>))
)]
pub async fn get_orderbook(
    State(_state): State<AppState>,
    Path(coin): Path<String>,
    Query(query): Query<HyperliquidQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let orderbook = hyperliquid.get_orderbook(&coin, query.depth).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: orderbook,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get recent trades for a coin
#[utoipa::path(
    get,
    path = "/hyperliquid/trades/{coin}",
    params(
        ("coin" = String, Path, description = "Coin symbol"),
        HyperliquidQuery
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Recent trades", body = HyperliquidResponse<Vec<HyperliquidTrade>>))
)]
pub async fn get_recent_trades(
    State(_state): State<AppState>,
    Path(coin): Path<String>,
    Query(query): Query<HyperliquidQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let trades = hyperliquid.get_recent_trades(&coin, query.limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: trades,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get candlestick data
#[utoipa::path(
    get,
    path = "/hyperliquid/candles/{coin}",
    params(
        ("coin" = String, Path, description = "Coin symbol"),
        HyperliquidQuery
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Candlestick data", body = HyperliquidResponse<Vec<HyperliquidCandle>>))
)]
pub async fn get_candles(
    State(_state): State<AppState>,
    Path(coin): Path<String>,
    Query(query): Query<HyperliquidQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let interval = query.interval.as_deref().unwrap_or("1h");
    let end_time = query.end_time.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64);
    let start_time = query.start_time.unwrap_or(end_time - 24 * 60 * 60 * 1000); // 24 hours ago
    
    let candles = hyperliquid.get_candles(&coin, interval, start_time, end_time).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: candles,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get user state (requires user address)
#[utoipa::path(
    get,
    path = "/hyperliquid/user/{address}",
    params(
        ("address" = String, Path, description = "User wallet address")
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "User state and positions", body = HyperliquidResponse<HyperliquidUserState>))
)]
pub async fn get_user_state(
    State(_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let user_state = hyperliquid.get_user_state(&address).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: user_state,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get funding rates for all markets
#[utoipa::path(
    get,
    path = "/hyperliquid/funding",
    tag = "hyperliquid",
    responses((status = 200, description = "Funding rates", body = HyperliquidResponse<Vec<HyperliquidFunding>>))
)]
pub async fn get_funding_rates(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let funding_rates = hyperliquid.get_funding_rates().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: funding_rates,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get trending DeFi assets
#[utoipa::path(
    get,
    path = "/hyperliquid/trending",
    params(
        ("limit" = Option<u32>, Query, description = "Number of trending assets to return")
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Trending DeFi assets", body = HyperliquidResponse<Vec<TrendingItem>>))
)]
pub async fn get_trending_defi(
    State(_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(20);
    
    let trending = hyperliquid.get_trending_defi_assets(limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: trending,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get top markets by volume
#[utoipa::path(
    get,
    path = "/hyperliquid/volume/top",
    params(
        ("limit" = Option<u32>, Query, description = "Number of top markets to return")
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Top markets by volume", body = HyperliquidResponse<Vec<HyperliquidMarket>>))
)]
pub async fn get_top_volume(
    State(_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(20);
    
    let top_markets = hyperliquid.get_top_volume_markets(limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: top_markets,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get top movers (gainers and losers)
#[utoipa::path(
    get,
    path = "/hyperliquid/movers",
    params(
        ("limit" = Option<u32>, Query, description = "Number of movers to return in each category")
    ),
    tag = "hyperliquid",
    responses((status = 200, description = "Top gainers and losers", body = HyperliquidResponse<MarketMoversResponse>))
)]
pub async fn get_top_movers(
    State(_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10);
    
    let (gainers, losers) = hyperliquid.get_top_movers(limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let movers_data = MarketMoversResponse { gainers, losers };
    
    let response = HyperliquidResponse {
        success: true,
        data: movers_data,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get market overview statistics
#[utoipa::path(
    get,
    path = "/hyperliquid/stats/overview",
    tag = "hyperliquid",
    responses((status = 200, description = "Market overview statistics", body = HyperliquidResponse<serde_json::Value>))
)]
pub async fn get_market_overview(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hyperliquid = HyperliquidDataSource::new().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let markets = hyperliquid.get_all_markets().await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    // Calculate overview statistics
    let total_volume_24h: f64 = markets.iter().map(|m| m.volume_24h).sum();
    let total_open_interest: f64 = markets.iter().map(|m| m.open_interest).sum();
    let active_markets = markets.len();
    
    let avg_price_change = if !markets.is_empty() {
        markets.iter().map(|m| m.price_change_percentage_24h).sum::<f64>() / markets.len() as f64
    } else {
        0.0
    };
    
    let overview = serde_json::json!({
        "total_volume_24h": total_volume_24h,
        "total_open_interest": total_open_interest,
        "active_markets": active_markets,
        "average_price_change_24h": avg_price_change,
        "top_volume_market": markets.iter()
            .max_by(|a, b| a.volume_24h.partial_cmp(&b.volume_24h).unwrap_or(std::cmp::Ordering::Equal))
            .map(|m| &m.coin),
        "biggest_gainer": markets.iter()
            .max_by(|a, b| a.price_change_percentage_24h.partial_cmp(&b.price_change_percentage_24h).unwrap_or(std::cmp::Ordering::Equal))
            .map(|m| format!("{}: +{:.2}%", m.coin, m.price_change_percentage_24h)),
        "biggest_loser": markets.iter()
            .min_by(|a, b| a.price_change_percentage_24h.partial_cmp(&b.price_change_percentage_24h).unwrap_or(std::cmp::Ordering::Equal))
            .map(|m| format!("{}: {:.2}%", m.coin, m.price_change_percentage_24h)),
    });
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = HyperliquidResponse {
        success: true,
        data: overview,
        timestamp,
        source: "hyperliquid".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}
