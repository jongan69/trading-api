use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::sources::kraken_data::{
    KrakenDataSource, get_trending_crypto_pairs, get_market_summary,
};
use crate::errors::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct KrakenQuery {
    pub pairs: Option<String>,
    pub depth: Option<u32>,
    pub limit: Option<usize>,
    pub since: Option<u64>,
    pub interval: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct KrakenResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
}

pub fn router(_state: AppState) -> Router {
    Router::new()
        .route("/ticker", get(get_ticker))
        .route("/ticker/{pair}", get(get_ticker_by_pair))
        .route("/orderbook/{pair}", get(get_order_book))
        .route("/assets", get(get_assets))
        .route("/pairs", get(get_asset_pairs))
        .route("/trades/{pair}", get(get_recent_trades))
        .route("/ohlc/{pair}", get(get_ohlc))
        .route("/trending", get(get_trending_crypto))
        .route("/summary/{pair}", get(get_market_summary_route))
        .route("/status", get(get_system_status))
        .route("/time", get(get_server_time))
}

/// Get ticker information for specified pairs
#[utoipa::path(
    get,
    path = "/kraken/ticker",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "Ticker information for specified pairs"))
)]
pub async fn get_ticker(Query(query): Query<KrakenQuery>) -> Result<impl IntoResponse, ApiError> {
    let pairs = query.pairs
        .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let tickers = data_source.get_tickers_async(pairs).await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: tickers,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get ticker information for a specific pair
#[utoipa::path(
    get,
    path = "/kraken/ticker/{pair}",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "Ticker information for specific pair"))
)]
pub async fn get_ticker_by_pair(
    Path(pair): Path<String>,
    Query(_query): Query<KrakenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let tickers = data_source.get_tickers_async(vec![pair]).await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: tickers.first().cloned(),
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get order book for a specific pair
#[utoipa::path(
    get,
    path = "/kraken/orderbook/{pair}",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "Order book for specific pair"))
)]
pub async fn get_order_book(
    Path(pair): Path<String>,
    Query(query): Query<KrakenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let depth = query.depth.unwrap_or(10);
    let order_book = tokio::task::spawn_blocking(move || {
        data_source.get_order_book(&pair, depth)
    })
    .await
    .map_err(|e| ApiError::Upstream(format!("Task join error: {e}")))?
    .map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: order_book,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get all available assets
#[utoipa::path(
    get,
    path = "/kraken/assets",
    tag = "kraken",
    responses((status = 200, description = "All available assets"))
)]
pub async fn get_assets() -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let assets = data_source.get_assets().map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: assets,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get all available asset pairs
#[utoipa::path(
    get,
    path = "/kraken/pairs",
    tag = "kraken",
    responses((status = 200, description = "All available asset pairs"))
)]
pub async fn get_asset_pairs() -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let pairs = data_source.get_asset_pairs().map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: pairs,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get recent trades for a pair
#[utoipa::path(
    get,
    path = "/kraken/trades/{pair}",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "Recent trades for specific pair"))
)]
pub async fn get_recent_trades(
    Path(pair): Path<String>,
    Query(query): Query<KrakenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let trades = data_source.get_recent_trades(&pair, query.since).map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: trades,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get OHLC data for a pair
#[utoipa::path(
    get,
    path = "/kraken/ohlc/{pair}",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "OHLC data for specific pair"))
)]
pub async fn get_ohlc(
    Path(pair): Path<String>,
    Query(query): Query<KrakenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let ohlc = data_source.get_ohlc(&pair, query.interval, query.since).map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: ohlc,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get trending crypto pairs
#[utoipa::path(
    get,
    path = "/kraken/trending",
    params(KrakenQuery),
    tag = "kraken",
    responses((status = 200, description = "Trending crypto pairs by volume"))
)]
pub async fn get_trending_crypto(Query(query): Query<KrakenQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(10);
    let trending_items = get_trending_crypto_pairs(limit).await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: trending_items,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get market summary for a specific pair
#[utoipa::path(
    get,
    path = "/kraken/summary/{pair}",
    tag = "kraken",
    responses((status = 200, description = "Market summary for specific pair"))
)]
pub async fn get_market_summary_route(Path(pair): Path<String>) -> Result<impl IntoResponse, ApiError> {
    let summary = get_market_summary(&pair).await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: summary,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get system status
#[utoipa::path(
    get,
    path = "/kraken/status",
    tag = "kraken",
    responses((status = 200, description = "Kraken system status"))
)]
pub async fn get_system_status() -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async()
        .await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let status = tokio::task::spawn_blocking(move || data_source.get_system_status())
        .await
        .map_err(|e| ApiError::Upstream(format!("Task join error: {e}")))?
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: status,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get server time
#[utoipa::path(
    get,
    path = "/kraken/time",
    tag = "kraken",
    responses((status = 200, description = "Kraken server time"))
)]
pub async fn get_server_time() -> Result<impl IntoResponse, ApiError> {
    let data_source = KrakenDataSource::new_async()
        .await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let time = tokio::task::spawn_blocking(move || data_source.get_server_time())
        .await
        .map_err(|e| ApiError::Upstream(format!("Task join error: {e}")))?
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = KrakenResponse {
        success: true,
        data: time,
        timestamp,
    };
    
    Ok((StatusCode::OK, Json(response)))
}
