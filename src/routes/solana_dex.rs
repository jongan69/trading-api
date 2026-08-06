use axum::{extract::{Path, Query}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::errors::ApiError;
use crate::sources::dexscreener_data;

#[derive(Debug, Deserialize, IntoParams)]
pub struct DexQuery {
    pub limit: Option<usize>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TokenQuery {
    pub address: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/solana-dex/trending", get(get_trending))
        .route("/solana-dex/top-volume", get(get_top_volume))
        .route("/solana-dex/top-gainers", get(get_top_gainers))
        .route("/solana-dex/market-stats", get(get_market_stats))
        .route("/solana-dex/token/{address}", get(get_token))
        .route("/solana-dex/search", get(search_tokens))
}

/// Get trending Solana tokens from DexScreener.
#[utoipa::path(get, path = "/solana-dex/trending", params(DexQuery), tag = "solana-dex")]
pub async fn get_trending(Query(q): Query<DexQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let data = dexscreener_data::get_trending_tokens(limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}

/// Get Solana tokens sorted by 24h volume.
#[utoipa::path(get, path = "/solana-dex/top-volume", params(DexQuery), tag = "solana-dex")]
pub async fn get_top_volume(Query(q): Query<DexQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let data = dexscreener_data::get_top_by_volume(limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}

/// Get top Solana gainers by 24h price change.
#[utoipa::path(get, path = "/solana-dex/top-gainers", params(DexQuery), tag = "solana-dex")]
pub async fn get_top_gainers(Query(q): Query<DexQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let data = dexscreener_data::get_top_gainers(limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}

/// Get combined Solana market stats (trending, volume, gainers).
#[utoipa::path(get, path = "/solana-dex/market-stats", params(DexQuery), tag = "solana-dex")]
pub async fn get_market_stats(Query(q): Query<DexQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let data = dexscreener_data::get_market_stats(limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}

/// Get all trading pairs for a specific Solana token address.
#[utoipa::path(get, path = "/solana-dex/token/{address}", tag = "solana-dex")]
pub async fn get_token(Path(address): Path<String>) -> Result<impl IntoResponse, ApiError> {
    let data = dexscreener_data::get_token_pairs(&address).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}

/// Search Solana tokens by name or symbol via DexScreener.
#[utoipa::path(get, path = "/solana-dex/search", params(DexQuery), tag = "solana-dex")]
pub async fn search_tokens(Query(q): Query<DexQuery>) -> Result<impl IntoResponse, ApiError> {
    let query = q.query.unwrap_or_else(|| "SOL".to_string());
    let limit = q.limit.unwrap_or(20);
    let data = dexscreener_data::search_tokens(&query, limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true, "data": data }))))
}
