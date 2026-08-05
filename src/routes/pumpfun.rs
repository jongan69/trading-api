use axum::{extract::{Path, Query}, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::state::AppState;
use crate::sources::pumpfun_data::TokenInfo;

/// 501 body shared by all Pump.fun endpoints while the integration is offline.
#[derive(Serialize, ToSchema)]
struct NotImplementedBody {
    error: String,
    code: String,
    status: u16,
}

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
    pub tokens: Vec<TokenInfo>,
    pub total_tokens: usize,
    pub avg_market_cap: Option<f64>,
    pub top_by_volume: Vec<TokenInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PumpFunMarketSummary {
    pub total_tokens: Option<usize>,
    pub total_market_cap: Option<f64>,
    pub total_volume_24h: Option<f64>,
    pub new_tokens_24h: usize,
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
        .route("/search", get(search_tokens))
        .with_state(state)
}

/// All Pump.fun endpoints return 501 while the integration is rebuilt.
fn pumpfun_not_implemented() -> (axum::http::StatusCode, Json<NotImplementedBody>) {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(NotImplementedBody {
            error: "Pump.fun integration is not currently available".into(),
            code: "NOT_IMPLEMENTED".into(),
            status: 501,
        }),
    )
}

#[utoipa::path(get, path = "/pumpfun/trending", tag = "pumpfun")]
pub async fn get_trending_pumpfun() -> impl IntoResponse { pumpfun_not_implemented() }

#[utoipa::path(get, path = "/pumpfun/token/{mint_address}", tag = "pumpfun")]
pub async fn get_token_info(Path(mint_address): Path<String>) -> impl IntoResponse {
    if mint_address.len() < 32 || mint_address.len() > 44 {
        return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid mint address", "code": "BAD_REQUEST"}))).into_response();
    }
    pumpfun_not_implemented().into_response()
}

#[utoipa::path(get, path = "/pumpfun/bonding-curve/{mint_address}", tag = "pumpfun")]
pub async fn get_bonding_curve(Path(mint_address): Path<String>) -> impl IntoResponse {
    if mint_address.len() < 32 || mint_address.len() > 44 {
        return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid mint address", "code": "BAD_REQUEST"}))).into_response();
    }
    pumpfun_not_implemented().into_response()
}

#[utoipa::path(get, path = "/pumpfun/market-summary", tag = "pumpfun")]
pub async fn get_market_summary() -> impl IntoResponse { pumpfun_not_implemented() }

#[utoipa::path(get, path = "/pumpfun/new-tokens", tag = "pumpfun")]
pub async fn get_new_tokens(Query(_q): Query<PumpFunQuery>) -> impl IntoResponse { pumpfun_not_implemented() }

#[utoipa::path(get, path = "/pumpfun/top-gainers", tag = "pumpfun")]
pub async fn get_top_gainers(Query(_q): Query<PumpFunQuery>) -> impl IntoResponse { pumpfun_not_implemented() }

#[utoipa::path(get, path = "/pumpfun/top-losers", tag = "pumpfun")]
pub async fn get_top_losers(Query(_q): Query<PumpFunQuery>) -> impl IntoResponse { pumpfun_not_implemented() }

#[utoipa::path(get, path = "/pumpfun/search", tag = "pumpfun")]
pub async fn search_tokens(Query(_q): Query<PumpFunQuery>) -> impl IntoResponse { pumpfun_not_implemented() }
