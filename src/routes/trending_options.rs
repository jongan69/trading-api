use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::helpers::trending_options::get_trending_options_analysis;
use crate::helpers::metrics::CompositeWeights;
use crate::helpers::params::periods_per_year_from_interval;
use crate::state::AppState;
use crate::types::ErrorResponse;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct TrendingOptionsQuery {
    /// Option type to analyze: "call", "put", or "both" (default: "call")
    pub option_type: Option<String>,
    /// Risk-free annual rate (default: 0.03)
    pub rf_annual: Option<f64>,
    /// Time range for underlying analysis: "1mo", "3mo", "6mo", "1y", "2y", "5y" (default: "3mo")
    pub range: Option<String>,
    /// Time interval for underlying analysis: "1d", "1wk", "1mo" (default: "1d")
    pub interval: Option<String>,
    /// Weight for Sharpe ratio in composite score (default: 0.4)
    pub sharpe_w: Option<f64>,
    /// Weight for Sortino ratio in composite score (default: 0.4)
    pub sortino_w: Option<f64>,
    /// Weight for Calmar ratio in composite score (default: 0.2)
    pub calmar_w: Option<f64>,
    /// Maximum number of results to return (default: 10)
    pub limit: Option<usize>,
    /// Minimum underlying composite score to include (default: 0.0)
    pub min_underlying_score: Option<f64>,
    /// Minimum undervalued score to include (default: 0.0)
    pub min_undervalued_score: Option<f64>,
}

#[derive(Serialize, ToSchema)]
pub struct TrendingOptionsResponse {
    pub results: Vec<serde_json::Value>,
    pub summary: TrendingOptionsSummary,
}

#[derive(Serialize, ToSchema)]
pub struct TrendingOptionsSummary {
    pub total_analyzed: usize,
    pub total_with_options: usize,
    pub average_underlying_score: f64,
    pub average_undervalued_score: f64,
    pub top_underlying_tickers: Vec<String>,
    pub top_undervalued_tickers: Vec<String>,
}

/// Get trending tickers with undervalued options analysis
#[utoipa::path(
    get,
    path = "/trending-options",
    params(TrendingOptionsQuery),
    responses(
        (status = 200, description = "Successfully retrieved trending options analysis", body = TrendingOptionsResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "trending-options"
)]
pub async fn get_trending_options_handler(
    State(state): State<AppState>,
    Query(query): Query<TrendingOptionsQuery>,
) -> Result<Json<TrendingOptionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let option_type = query.option_type.as_deref();
    let rf_annual = query.rf_annual.unwrap_or(0.03);
    let interval = query.interval.as_deref().unwrap_or("1d");
    let periods_per_year = periods_per_year_from_interval(interval);
    let limit = query.limit.unwrap_or(10);
    let min_underlying_score = query.min_underlying_score.unwrap_or(0.0);
    let min_undervalued_score = query.min_undervalued_score.unwrap_or(0.0);

    let weights = CompositeWeights {
        sharpe: query.sharpe_w.unwrap_or(0.4),
        sortino: query.sortino_w.unwrap_or(0.4),
        calmar: query.calmar_w.unwrap_or(0.2),
    };

    // Get trending options analysis
    let mut results = get_trending_options_analysis(
        &state,
        rf_annual,
        periods_per_year,
        &weights,
        option_type,
        limit * 2, // Get more results initially for filtering
    ).await;

    // Filter results based on minimum scores
    results.retain(|result| {
        let underlying_score = result
            .get("underlying_metrics")
            .and_then(|m| m.get("metrics"))
            .and_then(|m| m.get("composite_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);

        let max_undervalued_score = result
            .get("options_analysis")
            .and_then(|oa| oa.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|opt| {
                        opt.get("undervalued_indicators")
                            .and_then(|ui| ui.get("overall_undervalued_score"))
                            .and_then(|s| s.as_f64())
                    })
                    .fold(0.0, f64::max)
            })
            .unwrap_or(0.0);

        underlying_score >= min_underlying_score && max_undervalued_score >= min_undervalued_score
    });

    // Apply final limit
    if results.len() > limit {
        results.truncate(limit);
    }

    // Calculate summary statistics
    let total_analyzed = results.len();
    let total_with_options = results
        .iter()
        .filter(|r| {
            r.get("options_analysis")
                .and_then(|oa| oa.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        })
        .count();

    let underlying_scores: Vec<f64> = results
        .iter()
        .filter_map(|r| {
            r.get("underlying_metrics")
                .and_then(|m| m.get("metrics"))
                .and_then(|m| m.get("composite_score"))
                .and_then(|s| s.as_f64())
        })
        .collect();

    let average_underlying_score = if !underlying_scores.is_empty() {
        underlying_scores.iter().sum::<f64>() / underlying_scores.len() as f64
    } else {
        0.0
    };

    let undervalued_scores: Vec<f64> = results
        .iter()
        .filter_map(|r| {
            r.get("options_analysis")
                .and_then(|oa| oa.as_array())
                .and_then(|arr| {
                    arr.iter()
                        .filter_map(|opt| {
                            opt.get("undervalued_indicators")
                                .and_then(|ui| ui.get("overall_undervalued_score"))
                                .and_then(|s| s.as_f64())
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                })
        })
        .collect();

    let average_undervalued_score = if !undervalued_scores.is_empty() {
        undervalued_scores.iter().sum::<f64>() / undervalued_scores.len() as f64
    } else {
        0.0
    };

    // Get top tickers by underlying score
    let mut tickers_with_scores: Vec<(String, f64)> = results
        .iter()
        .filter_map(|r| {
            let symbol = r.get("symbol")?.as_str()?;
            let score = r
                .get("underlying_metrics")
                .and_then(|m| m.get("metrics"))
                .and_then(|m| m.get("composite_score"))
                .and_then(|s| s.as_f64())?;
            Some((symbol.to_string(), score))
        })
        .collect();

    tickers_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_underlying_tickers: Vec<String> = tickers_with_scores
        .iter()
        .take(5)
        .map(|(symbol, _)| symbol.clone())
        .collect();

    // Get top tickers by undervalued score
    let mut tickers_with_undervalued: Vec<(String, f64)> = results
        .iter()
        .filter_map(|r| {
            let symbol = r.get("symbol")?.as_str()?;
            let max_score = r
                .get("options_analysis")
                .and_then(|oa| oa.as_array())
                .and_then(|arr| {
                    arr.iter()
                        .filter_map(|opt| {
                            opt.get("undervalued_indicators")
                                .and_then(|ui| ui.get("overall_undervalued_score"))
                                .and_then(|s| s.as_f64())
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                })?;
            Some((symbol.to_string(), max_score))
        })
        .collect();

    tickers_with_undervalued.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_undervalued_tickers: Vec<String> = tickers_with_undervalued
        .iter()
        .take(5)
        .map(|(symbol, _)| symbol.clone())
        .collect();

    let summary = TrendingOptionsSummary {
        total_analyzed,
        total_with_options,
        average_underlying_score,
        average_undervalued_score,
        top_underlying_tickers,
        top_undervalued_tickers,
    };

    Ok(Json(TrendingOptionsResponse { results, summary }))
}

pub fn router(state: AppState) -> Router {
    Router::new().route("/trending-options", get(get_trending_options_handler)).with_state(state)
}
