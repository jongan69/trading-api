use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::sources::coingecko_data::{
    CoinGeckoCoin, MarketOverview, TrendingCoin, get_top_coins, get_top_gainers, 
    get_top_losers, get_trending_coins, get_market_overview, get_market_context,
    get_trending_cryptos, get_simple_price
};
use serde_json::Value;

#[derive(Debug, Deserialize, IntoParams, utoipa::ToSchema)]
pub struct CoinGeckoQuery {
    pub limit: Option<usize>,
    pub vs_currency: Option<String>,
    pub order: Option<String>,
    pub page: Option<usize>,
    pub sparkline: Option<bool>,
    pub price_change_percentage: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, utoipa::ToSchema)]
pub struct SimplePriceQuery {
    pub ids: String,
    pub vs_currencies: String,
    pub include_24hr_change: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CoinGeckoResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketContextResponse {
    pub success: bool,
    pub context: String,
    pub timestamp: i64,
}

/// Get top cryptocurrencies by market cap
#[utoipa::path(
    get,
    path = "/coingecko/top",
    params(CoinGeckoQuery),
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Vec<CoinGeckoCoin>>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_top_cryptocurrencies(
    Query(query): Query<CoinGeckoQuery>,
) -> Result<Json<CoinGeckoResponse<Vec<CoinGeckoCoin>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10);
    
    match get_top_coins(limit).await {
        Ok(coins) => {
            let response = CoinGeckoResponse {
                success: true,
                data: coins,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get top gainers in the last 24 hours
#[utoipa::path(
    get,
    path = "/coingecko/gainers",
    params(CoinGeckoQuery),
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Vec<CoinGeckoCoin>>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_top_gainers_route(
    Query(query): Query<CoinGeckoQuery>,
) -> Result<Json<CoinGeckoResponse<Vec<CoinGeckoCoin>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10);
    
    match get_top_gainers(limit).await {
        Ok(coins) => {
            let response = CoinGeckoResponse {
                success: true,
                data: coins,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get top losers in the last 24 hours
#[utoipa::path(
    get,
    path = "/coingecko/losers",
    params(CoinGeckoQuery),
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Vec<CoinGeckoCoin>>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_top_losers_route(
    Query(query): Query<CoinGeckoQuery>,
) -> Result<Json<CoinGeckoResponse<Vec<CoinGeckoCoin>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10);
    
    match get_top_losers(limit).await {
        Ok(coins) => {
            let response = CoinGeckoResponse {
                success: true,
                data: coins,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get trending cryptocurrencies
#[utoipa::path(
    get,
    path = "/coingecko/trending",
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Vec<TrendingCoin>>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_trending_cryptocurrencies() -> Result<Json<CoinGeckoResponse<Vec<TrendingCoin>>>, (StatusCode, String)> {
    match get_trending_coins().await {
        Ok(coins) => {
            let response = CoinGeckoResponse {
                success: true,
                data: coins,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get market overview statistics
#[utoipa::path(
    get,
    path = "/coingecko/market-overview",
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<MarketOverview>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_market_overview_route() -> Result<Json<CoinGeckoResponse<MarketOverview>>, (StatusCode, String)> {
    match get_market_overview().await {
        Ok(overview) => {
            let response = CoinGeckoResponse {
                success: true,
                data: overview,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get comprehensive market context for analysis
#[utoipa::path(
    get,
    path = "/coingecko/market-context",
    responses(
        (status = 200, description = "Success", body = MarketContextResponse),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_market_context_route() -> Result<Json<MarketContextResponse>, (StatusCode, String)> {
    match get_market_context().await {
        Ok(context) => {
            let response = MarketContextResponse {
                success: true,
                context,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get trending crypto symbols
#[utoipa::path(
    get,
    path = "/coingecko/trending-symbols",
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Vec<String>>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_trending_symbols() -> Result<Json<CoinGeckoResponse<Vec<String>>>, (StatusCode, String)> {
    match get_trending_cryptos().await {
        Ok(symbols) => {
            let response = CoinGeckoResponse {
                success: true,
                data: symbols,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

/// Get simple price data for specific coins
#[utoipa::path(
    get,
    path = "/coingecko/simple-price",
    params(SimplePriceQuery),
    responses(
        (status = 200, description = "Success", body = CoinGeckoResponse<Value>),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "CoinGecko"
)]
pub async fn get_simple_price_route(
    Query(query): Query<SimplePriceQuery>,
) -> Result<Json<CoinGeckoResponse<Value>>, (StatusCode, String)> {
    let ids: Vec<String> = query.ids.split(',').map(|s| s.trim().to_string()).collect();
    let vs_currencies: Vec<String> = query.vs_currencies.split(',').map(|s| s.trim().to_string()).collect();
    let include_24hr_change = query.include_24hr_change.unwrap_or(false);
    
    match get_simple_price(&ids, &vs_currencies, include_24hr_change).await {
        Ok(data) => {
            let response = CoinGeckoResponse {
                success: true,
                data,
                timestamp: chrono::Utc::now().timestamp(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub fn coingecko_routes() -> Router {
    Router::new()
        .route("/top", get(get_top_cryptocurrencies))
        .route("/gainers", get(get_top_gainers_route))
        .route("/losers", get(get_top_losers_route))
        .route("/trending", get(get_trending_cryptocurrencies))
        .route("/market-overview", get(get_market_overview_route))
        .route("/market-context", get(get_market_context_route))
        .route("/trending-symbols", get(get_trending_symbols))
        .route("/simple-price", get(get_simple_price_route))
}
