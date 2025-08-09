use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::helpers;
use crate::sources;
use crate::types::LimitQuery;
use crate::errors::ApiError;
use crate::state::AppState;

pub fn router(_state: AppState) -> Router {
    Router::new()
        .route("/news", get(news_aggregated))
        .route("/forex", get(sources::finviz_data::get_forex))
        .route("/crypto", get(sources::finviz_data::get_crypto))
        .route("/future", get(sources::finviz_data::get_future))
        .route("/insider", get(sources::finviz_data::get_insider))
        .route("/group", get(sources::finviz_data::get_group))
        .route("/reddit/stocks", get(get_reddit_stocks))
        .route("/trending/stocks", get(get_trending_stocks))
}

#[utoipa::path(get, path = "/news", tag = "data", responses((status = 200, description = "Aggregated news from Finviz, Reddit, and Alpaca")))]
pub async fn news_aggregated() -> Result<impl IntoResponse, ApiError> {
    let v = helpers::news::get_news().await.map_err(ApiError::Upstream)?;
    Ok((StatusCode::OK, Json(v)))
}

#[utoipa::path(get, path = "/reddit/stocks", params(LimitQuery), tag = "data", responses((status = 200, description = "Trending tickers from Reddit")))]
pub async fn get_reddit_stocks(Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    let mut symbols = sources::reddit_data::get_reddit_trending_stocks().await;
    if let Some(max) = limit { if symbols.len() > max { symbols.truncate(max); } }
    (StatusCode::OK, Json(json!({ "symbols": symbols }))).into_response()
}

#[utoipa::path(get, path = "/trending/stocks", params(LimitQuery), tag = "data", responses((status = 200, description = "Trending tickers from web sources")))]
pub async fn get_trending_stocks(Query(query): Query<LimitQuery>) -> impl IntoResponse {
    let LimitQuery { limit } = query;
    let mut symbols = helpers::trending_stocks::get_trending_penny_stocks().await;
    if let Some(max) = limit { if symbols.len() > max { symbols.truncate(max); } }
    (StatusCode::OK, Json(json!({ "symbols": symbols }))).into_response()
}


