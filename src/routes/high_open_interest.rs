use axum::{
    extract::{Query, Path},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::helpers::high_open_interest::get_high_open_interest_contracts;
use crate::types::HighOpenInterestResult;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct HighOpenInterestQuery {
    pub ticker: String,
    pub option_type: Option<String>, // "call" or "put"
}

#[derive(Serialize, ToSchema)]
pub struct HighOpenInterestResponse {
    pub ticker: String,
    pub result: HighOpenInterestResult,
}

/// Get high open interest option contracts for a specific ticker
#[utoipa::path(
    get,
    path = "/high-open-interest/{ticker}",
    params(
        ("ticker" = String, Path, description = "Stock ticker symbol"),
        ("option_type" = Option<String>, Query, description = "Option type: 'call' or 'put' (default: 'call')")
    ),
    responses(
        (status = 200, description = "Successfully retrieved high open interest contracts", body = HighOpenInterestResponse),
        (status = 400, description = "Bad request", body = crate::types::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::types::ErrorResponse)
    ),
    tag = "high-open-interest"
)]
pub async fn get_high_open_interest_handler(
    Path(ticker): Path<String>,
    Query(query): Query<HighOpenInterestQuery>,
) -> Result<Json<HighOpenInterestResponse>, (StatusCode, Json<crate::types::ErrorResponse>)> {
    let option_type = query.option_type.as_deref();
    
    let result = get_high_open_interest_contracts(&ticker, option_type).await;
    
    Ok(Json(HighOpenInterestResponse {
        ticker,
        result,
    }))
}

/// Get high open interest option contracts for multiple tickers
#[utoipa::path(
    get,
    path = "/high-open-interest/batch",
    params(
        ("tickers" = String, Query, description = "Comma-separated list of stock ticker symbols"),
        ("option_type" = Option<String>, Query, description = "Option type: 'call' or 'put' (default: 'call')")
    ),
    responses(
        (status = 200, description = "Successfully retrieved high open interest contracts for multiple tickers", body = Vec<HighOpenInterestResponse>),
        (status = 400, description = "Bad request", body = crate::types::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::types::ErrorResponse)
    ),
    tag = "high-open-interest"
)]
pub async fn get_high_open_interest_batch_handler(
    Query(query): Query<HighOpenInterestBatchQuery>,
) -> Result<Json<Vec<HighOpenInterestResponse>>, (StatusCode, Json<crate::types::ErrorResponse>)> {
    let tickers: Vec<String> = query.tickers
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tickers.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(crate::types::ErrorResponse {
                error: "No tickers provided".to_string(),
            }),
        ));
    }

    let option_type = query.option_type.as_deref();
    let mut responses = Vec::new();

    for ticker in tickers {
        let result = get_high_open_interest_contracts(&ticker, option_type).await;
        responses.push(HighOpenInterestResponse {
            ticker,
            result,
        });
    }

    Ok(Json(responses))
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct HighOpenInterestBatchQuery {
    pub tickers: String, // Comma-separated list
    pub option_type: Option<String>, // "call" or "put"
}

pub fn router() -> Router {
    Router::new()
        .route("/high-open-interest/{ticker}", get(get_high_open_interest_handler))
        .route("/high-open-interest/batch", get(get_high_open_interest_batch_handler))
}
