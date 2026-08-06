use axum::{extract::{Path, Query}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::errors::ApiError;
use crate::sources::dexscreener_data;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct PumpFunQuery {
    pub limit: Option<usize>,
    pub sort_by: Option<String>,
    pub time_range: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunResponse<T: Serialize + ToSchema> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunTrendingResponse {
    pub tokens: Vec<dexscreener_data::SolanaTokenPair>,
    pub total_tokens: usize,
}

pub fn router() -> Router {
    Router::new()
        .route("/pumpfun/trending", get(get_trending_pumpfun))
        .route("/pumpfun/token/{address}", get(get_token_info))
        .route("/pumpfun/search", get(search_tokens))
        .route("/pumpfun/market-summary", get(get_market_summary))
        .route("/pumpfun/bonding-curve/{address}", get(get_bonding_curve))
        .route("/pumpfun/new-tokens", get(get_new_tokens))
        .route("/pumpfun/top-gainers", get(get_top_gainers))
        .route("/pumpfun/top-losers", get(get_top_losers))
}

fn ts() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }

/// Get trending pump.fun tokens from DexScreener.
#[utoipa::path(get, path = "/pumpfun/trending", tag = "pumpfun")]
pub async fn get_trending_pumpfun(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let tokens = dexscreener_data::get_pumpfun_tokens(limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    let total = tokens.len();
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true,
        data: PumpFunTrendingResponse { tokens, total_tokens: total },
        timestamp: ts(),
        source: "dexscreener".into(),
    })))
}

/// Get pump.fun token info by contract address.
#[utoipa::path(get, path = "/pumpfun/token/{address}", tag = "pumpfun")]
pub async fn get_token_info(Path(address): Path<String>) -> Result<impl IntoResponse, ApiError> {
    if address.len() < 32 || address.len() > 44 {
        return Err(ApiError::BadRequest("Invalid address length".into()));
    }
    let pairs = dexscreener_data::get_token_pairs(&address).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true,
        data: pairs,
        timestamp: ts(),
        source: "dexscreener".into(),
    })))
}

/// Search pump.fun tokens by name/symbol.
#[utoipa::path(get, path = "/pumpfun/search", tag = "pumpfun")]
pub async fn search_tokens(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let query = q.query.unwrap_or_default();
    let limit = q.limit.unwrap_or(20);
    if query.is_empty() {
        return Err(ApiError::BadRequest("query parameter required".into()));
    }
    let tokens = dexscreener_data::search_tokens(&query, limit).await
        .map_err(|e| ApiError::Upstream(e))?;
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true,
        data: tokens,
        timestamp: ts(),
        source: "dexscreener".into(),
    })))
}

/// Market summary — trending + volume + gainers.
#[utoipa::path(get, path = "/pumpfun/market-summary", tag = "pumpfun")]
pub async fn get_market_summary(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(10);
    let tokens = dexscreener_data::get_pumpfun_tokens(limit * 3).await
        .map_err(|e| ApiError::Upstream(e))?;

    let total_mcap: f64 = tokens.iter().filter_map(|t| t.market_cap).sum();
    let total_vol: f64 = tokens.iter().filter_map(|t| t.volume_24h).sum();

    let mut gainers = tokens.clone();
    gainers.sort_by(|a, b| b.price_change_24h.unwrap_or(f64::MIN).partial_cmp(&a.price_change_24h.unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal));

    let mut losers = tokens.clone();
    losers.sort_by(|a, b| a.price_change_24h.unwrap_or(f64::MAX).partial_cmp(&b.price_change_24h.unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal));

    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "timestamp": ts(),
        "source": "dexscreener",
        "data": {
            "total_market_cap": total_mcap,
            "total_volume_24h": total_vol,
            "tokens_tracked": tokens.len(),
            "top_gainers": &gainers[..gainers.len().min(limit)],
            "top_losers": &losers[..losers.len().min(limit)],
        }
    }))))
}

/// Bonding curve — derived from DexScreener liquidity ratio.
#[utoipa::path(get, path = "/pumpfun/bonding-curve/{address}", tag = "pumpfun")]
pub async fn get_bonding_curve(Path(address): Path<String>) -> Result<impl IntoResponse, ApiError> {
    if address.len() < 32 || address.len() > 44 {
        return Err(ApiError::BadRequest("Invalid address length".into()));
    }
    let pairs = dexscreener_data::get_token_pairs(&address).await
        .map_err(|e| ApiError::Upstream(e))?;

    if let Some(pair) = pairs.first() {
        let progress = pair.liquidity_usd.map(|liq| {
            pair.market_cap.map(|mc| if mc > 0.0 { (liq / mc * 100.0).min(100.0) } else { 0.0 }).unwrap_or(0.0)
        });
        Ok((StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "data": {
                "token_address": address,
                "symbol": pair.symbol,
                "current_price_usd": pair.price_usd,
                "market_cap": pair.market_cap,
                "liquidity_usd": pair.liquidity_usd,
                "bonding_curve_progress_pct": progress,
                "volume_24h": pair.volume_24h,
            }
        }))))
    } else {
        Err(ApiError::NotFound(format!("No pairs found for {address}")))
    }
}

/// New tokens — sorted by creation time.
#[utoipa::path(get, path = "/pumpfun/new-tokens", tag = "pumpfun")]
pub async fn get_new_tokens(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let mut tokens = dexscreener_data::get_pumpfun_tokens(limit * 2).await
        .map_err(|e| ApiError::Upstream(e))?;
    tokens.sort_by(|a, b| b.created_at.unwrap_or(0).cmp(&a.created_at.unwrap_or(0)));
    tokens.truncate(limit);
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true, data: tokens, timestamp: ts(), source: "dexscreener".into(),
    })))
}

/// Top gainers.
#[utoipa::path(get, path = "/pumpfun/top-gainers", tag = "pumpfun")]
pub async fn get_top_gainers(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let mut tokens = dexscreener_data::get_pumpfun_tokens(limit * 2).await
        .map_err(|e| ApiError::Upstream(e))?;
    tokens.sort_by(|a, b| b.price_change_24h.unwrap_or(f64::MIN).partial_cmp(&a.price_change_24h.unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal));
    tokens.truncate(limit);
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true, data: tokens, timestamp: ts(), source: "dexscreener".into(),
    })))
}

/// Top losers.
#[utoipa::path(get, path = "/pumpfun/top-losers", tag = "pumpfun")]
pub async fn get_top_losers(Query(q): Query<PumpFunQuery>) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    let mut tokens = dexscreener_data::get_pumpfun_tokens(limit * 2).await
        .map_err(|e| ApiError::Upstream(e))?;
    tokens.sort_by(|a, b| a.price_change_24h.unwrap_or(f64::MAX).partial_cmp(&b.price_change_24h.unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal));
    tokens.truncate(limit);
    Ok((StatusCode::OK, Json(PumpFunResponse {
        success: true, data: tokens, timestamp: ts(), source: "dexscreener".into(),
    })))
}
