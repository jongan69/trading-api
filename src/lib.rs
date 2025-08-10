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
        crate::routes::data::get_trending_crypto,
        crate::routes::yahoo::get_metrics_yahoo,
        crate::routes::yahoo::get_rank_yahoo,
        crate::routes::yahoo::get_recommendations_yahoo,
        crate::sources::finviz_data::get_screener_candidates,
        crate::sources::finviz_data::get_recommendations_finviz,
        crate::routes::options::get_options_recommendations,
        crate::routes::kraken::get_ticker,
        crate::routes::kraken::get_ticker_by_pair,
        crate::routes::kraken::get_order_book,
        crate::routes::kraken::get_assets,
        crate::routes::kraken::get_asset_pairs,
        crate::routes::kraken::get_recent_trades,
        crate::routes::kraken::get_ohlc,
        crate::routes::kraken::get_trending_crypto,
        crate::routes::kraken::get_market_summary_route,
        crate::routes::kraken::get_system_status,
        crate::routes::kraken::get_server_time,
        crate::routes::coingecko::get_top_cryptocurrencies,
        crate::routes::coingecko::get_top_gainers_route,
        crate::routes::coingecko::get_top_losers_route,
        crate::routes::coingecko::get_trending_cryptocurrencies,
        crate::routes::coingecko::get_market_overview_route,
        crate::routes::coingecko::get_market_context_route,
        crate::routes::coingecko::get_trending_symbols,
        crate::routes::coingecko::get_simple_price_route,
        crate::routes::high_open_interest::get_high_open_interest_handler,
        crate::routes::high_open_interest::get_high_open_interest_batch_handler,
        crate::routes::trending_options::get_trending_options_handler,
    ),
    components(schemas(
        crate::types::HealthResponse,
        crate::types::ErrorResponse,
        crate::types::LimitQuery,
        crate::types::YahooQuery,
        crate::sources::finviz_data::ScreenerQuery,
        crate::sources::finviz_data::FinvizRecommendationsQuery,
        crate::types::OptionsQuery,
        crate::routes::kraken::KrakenQuery,
        crate::sources::kraken_data::KrakenTicker,
        crate::sources::kraken_data::KrakenOrderBook,
        crate::sources::kraken_data::KrakenAsset,
        crate::sources::kraken_data::KrakenAssetPair,
        crate::routes::coingecko::CoinGeckoQuery,
        crate::routes::coingecko::SimplePriceQuery,
        crate::routes::coingecko::CoinGeckoResponse<crate::sources::coingecko_data::CoinGeckoCoin>,
        crate::routes::coingecko::CoinGeckoResponse<crate::sources::coingecko_data::TrendingCoin>,
        crate::routes::coingecko::CoinGeckoResponse<crate::sources::coingecko_data::MarketOverview>,
        crate::routes::coingecko::CoinGeckoResponse<Vec<String>>,
        crate::routes::coingecko::CoinGeckoResponse<serde_json::Value>,
        crate::routes::coingecko::MarketContextResponse,
        crate::sources::coingecko_data::CoinGeckoCoin,
        crate::sources::coingecko_data::MarketOverview,
        crate::sources::coingecko_data::TrendingCoin,
        crate::types::OptionContract,
        crate::types::OptionPrices,
        crate::types::HighOpenInterestResult,
        crate::routes::high_open_interest::HighOpenInterestResponse,
        crate::routes::high_open_interest::HighOpenInterestQuery,
        crate::routes::high_open_interest::HighOpenInterestBatchQuery,
        crate::routes::trending_options::TrendingOptionsQuery,
        crate::routes::trending_options::TrendingOptionsResponse,
        crate::routes::trending_options::TrendingOptionsSummary,
    )),
    tags(
        (name = "system", description = "Health & meta"),
        (name = "data", description = "Aggregated market data from Finviz, Reddit, Yahoo,and Alpaca"),
        (name = "options", description = "Options recommendations"),
        (name = "kraken", description = "Kraken cryptocurrency exchange data"),
        (name = "high-open-interest", description = "High open interest option contracts from Alpaca"),
        (name = "trending-options", description = "Trending tickers with undervalued options analysis")
    )
)]
struct ApiDoc;

pub fn build_app(state: state::AppState) -> Router {
    let openapi = ApiDoc::openapi();
    Router::new()
        .merge(routes::system::router(state.clone()))
        .merge(routes::data::router(state.clone()))
        .merge(routes::yahoo::router(state.clone()))
        .merge(routes::options::router(state.clone()))
        .merge(routes::high_open_interest::router())
        .merge(routes::trending_options::router(state.clone()))
        .nest("/kraken", routes::kraken::router(state))
        .nest("/coingecko", routes::coingecko::coingecko_routes())
        .route("/screener/candidates", axum::routing::get(crate::sources::finviz_data::get_screener_candidates))
        .route("/recommendations/finviz", axum::routing::get(crate::sources::finviz_data::get_recommendations_finviz))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
        .layer(TraceLayer::new_for_http())
}


