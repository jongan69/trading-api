pub mod helpers;
pub mod sources;
pub mod state;
pub mod routes;
pub mod services;
pub mod types;
pub mod errors;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::trace::TraceLayer;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::system::health,
        crate::routes::data::news_aggregated,
        crate::sources::finviz_data::get_forex,
        crate::sources::finviz_data::get_crypto,
        crate::sources::finviz_data::get_future,
        crate::sources::finviz_data::get_insider,
        crate::sources::finviz_data::get_group,
        crate::routes::data::get_reddit_stocks,
        crate::routes::data::get_trending_stocks,
        crate::routes::yahoo::get_metrics_yahoo,
        crate::routes::yahoo::get_rank_yahoo,
        crate::routes::yahoo::get_recommendations_yahoo,
        crate::sources::finviz_data::get_screener_candidates,
        crate::sources::finviz_data::get_recommendations_finviz,
        crate::routes::options::get_options_recommendations,
    ),
    components(schemas(
        crate::types::HealthResponse,
        crate::types::ErrorResponse,
        crate::types::LimitQuery,
        crate::types::YahooQuery,
        crate::sources::finviz_data::ScreenerQuery,
        crate::sources::finviz_data::FinvizRecommendationsQuery,
        crate::types::OptionsQuery,
    )),
    tags(
        (name = "system", description = "Health & meta"),
        (name = "data", description = "Aggregated market data from Finviz, Reddit, Yahoo,and Alpaca"),
        (name = "options", description = "Options recommendations")
    )
)]
struct ApiDoc;

pub fn build_app(state: state::AppState) -> Router {
    let openapi = ApiDoc::openapi();
    Router::new()
        .merge(routes::system::router(state.clone()))
        .merge(routes::data::router(state.clone()))
        .merge(routes::yahoo::router(state.clone()))
        .merge(routes::options::router(state))
        .route("/screener/candidates", axum::routing::get(crate::sources::finviz_data::get_screener_candidates))
        .route("/recommendations/finviz", axum::routing::get(crate::sources::finviz_data::get_recommendations_finviz))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
        .layer(TraceLayer::new_for_http())
}


