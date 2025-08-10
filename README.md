# Trading API (Rust + Axum)

A lightweight HTTP API for sourcing market data and generating trading recommendations. It combines Finviz screening with Yahoo Finance historical and options data, then ranks assets/options using a composite of Kelly-inspired position sizing with Sharpe, Sortino, and Calmar risk-adjusted metrics.

## Highlights
- Axum + Tokio: GET-only REST API with modular routes/services
- Data sources:
  - Finviz via `finviz_rs` (screeners, news, groups, etc.)
  - Yahoo Finance via `yahoo_finance_api` (historical quotes) and options JSON endpoint
  - CoinGecko API (cryptocurrency market data, trending, prices)
  - Kraken API (cryptocurrency exchange data, order books, trades)
  - Reddit API (trending stocks)
- Metrics engine: `src/helpers/metrics.rs` (Sharpe, Sortino, Calmar, Kelly, composite score)
- Options recommendations: dynamic ranking with auto-sourced symbols (Finviz or Yahoo lists)
- OpenAPI + Swagger UI: browse docs at `/docs` (served from `/openapi.json`)
- Observability: `tracing` + `tower-http` request logging (enable via `RUST_LOG`)

## Environment
Create a `.env` file (or export variables) for providers that require auth:

```
# Server
PORT=3000

# Alpaca (either ALPACA_* or APCA_* are accepted)
ALPACA_API_KEY_ID=your_key
ALPACA_API_SECRET_KEY=your_secret
# or
APCA_API_KEY_ID=your_key
APCA_API_SECRET_KEY=your_secret

# Logging (optional)
RUST_LOG=info,tower_http=info

# Reddit API (optional, for trending stocks)
REDDIT_CLIENT_ID=your_reddit_client_id
REDDIT_CLIENT_SECRET=your_reddit_client_secret
REDDIT_USERNAME=your_reddit_username
REDDIT_PASSWORD=your_reddit_password
```

## 📐 Background: Key Formulas

In quantitative finance, measuring risk-adjusted return is fundamental. Several established ratios offer complementary lenses on volatility, drawdown, and return dynamics. Our approach draws on the Kelly Criterion, the Calmar Ratio, and the Sortino Ratio, while taking inspiration from the Medallion Fund’s emphasis on probabilistic, risk-managed trading.

### 1. Kelly Criterion

The Kelly Criterion determines the optimal fraction of capital to allocate to a trade to maximize long-term growth:

```math
f^* = \frac{\mu - r}{\sigma^2}
```

Where:
- \(f^*\): optimal fraction of capital to wager
- ($`\mu`$): expected return per period
- ($`r`$): risk‑free rate per period
- ($`sigma^2`$): variance of returns per period

For assets with a known expected return ($`\mu`$) and variance ($`sigma^2`$), a continuous form often used is:

```math
f^* = \frac{\mu}{\sigma^2}
```

This connects closely to the Sharpe Ratio.

### 2. Sharpe Ratio

Measures excess return per unit of total volatility:

```math
\text{Sharpe} = \frac{R_p - R_f}{\sigma_p}
```

Where ($`R_p`$) is portfolio return, ($`R_f`$) the risk‑free rate, and ($`sigma_p`$) the standard deviation of returns. Sharpe treats upside and downside volatility symmetrically.

### 3. Sortino Ratio

Focuses on downside risk only:

```math
\text{Sortino} = \frac{R_p - R_f}{\sigma_D}
```

with downside deviation ($`sigma_D`$):

```math
\sigma_D = \sqrt{\frac{1}{N} \sum_{i=1}^{N} \min\big(0,\; R_i - T\big)^2}
```

Where ($`T`$) is a target return (e.g., 0 or a benchmark). Sortino better captures asymmetric risk.

### 4. Calmar Ratio

Emphasizes drawdown risk, useful for trend‑following or short‑term systems:

```math
\text{Calmar} = \frac{\text{CAGR}}{\text{Max Drawdown}}
```

Where CAGR is compound annual growth rate and Max Drawdown is the peak‑to‑trough loss.

### 🧠 Philosophical Influence: The Medallion Fund

The Medallion Fund’s exceptional track record (average annual returns exceeding 66% before fees) inspires a scientific, probabilistic, and risk‑managed framework:
- Position sizing optimized probabilistically (Kelly‑like)
- Short holding periods with asymmetric risk (Sortino‑like)
- Aggressive drawdown control and capital preservation (Calmar‑like)

These ideas motivate our composite metric for short‑term, risk‑aware capital allocation.

## Metrics & Scoring
- Implemented in `src/helpers/metrics.rs`:
  - Sharpe Ratio (annualized), Sortino Ratio (annualized), Calmar Ratio, Kelly fraction (clamped 0..1)
  - Downside deviation, volatility, CAGR, max drawdown
- Composite score = `w_sharpe*Sharpe + w_sortino*Sortino + w_calmar*Calmar`
- Options score ~ `UnderlyingComposite * Delta * (Spot / Premium) / (1 + DTE/30)`
  - Filters can exclude contracts by liquidity, spread, moneyness, delta, DTE, etc.


## Requirements
- Rust toolchain (1.75+ recommended)
- macOS/Linux/Windows

## Install & Run
```bash
# build
cargo build

# run (default on http://localhost:3000)
cargo run
```

## Testing
- Quick compile/unit tests:
  ```bash
  cargo test
  ```
- End-to-end (live network) tests for every endpoint:
  ```bash
  RUN_E2E=1 cargo test --tests
  ```
  - Spawns the server on an ephemeral port using `build_app` from `src/lib.rs` and hits all routes with `reqwest`.
  - Requires internet; may be rate‑limited by providers. Keep `limit` params small.
  - Alpaca keys are optional; options tests fall back to Yahoo when missing.

## Endpoints
All endpoints are GET.

### Health
- Route: `/health`
- Returns `{ "status": "ok" }` when the server is up.

### Finviz: News
- Route: `/news`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/news"
```

### Finviz: Forex
- Route: `/forex`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/forex?limit=3"
```

### Finviz: Crypto
- Route: `/crypto`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/crypto?limit=3"
```

### Finviz: Futures
- Route: `/future`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/future?limit=3"
```

### Finviz: Insider
- Route: `/insider`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/insider?limit=3"
```

### Finviz: Group (preset)
- Route: `/group`
- Returns group/industry metrics (preset to Industry/Valuation/PerformanceWeek/Ascending).
- Query: `limit?` (int)

### Reddit: Trending Stocks
- Route: `/reddit/stocks`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/reddit/stocks?limit=10"
```

### Trending: Stocks
- Route: `/trending/stocks`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/trending/stocks?limit=10"
```

### Trending: Crypto
- Route: `/trending/crypto`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/trending/crypto?limit=10"
```

### Yahoo: Single-symbol metrics
- Route: `/metrics/yahoo`
- Query:
  - `symbols` (required, exactly one symbol)
  - `range?`: `1mo|3mo|6mo|1y|2y|5y` (default: `3mo`)
  - `interval?`: `1d|1wk|1mo` (default: `1d`)
  - `rf_annual?`: risk-free annual rate (default: `0.0`)
  - `target_return_annual?`, `periods_per_year?`
- Example:
```bash
curl "http://localhost:3000/metrics/yahoo?symbols=AAPL&range=3mo&interval=1d&rf_annual=0.03"
```

### Yahoo: Rank multiple symbols
- Route: `/rank/yahoo`
- Query: same as above, but `symbols` can be comma-separated
- Example:
```bash
curl "http://localhost:3000/rank/yahoo?symbols=AAPL,MSFT,NVDA&range=3mo&interval=1d&rf_annual=0.03"
```

### Yahoo: Recommendations for provided symbols
- Route: `/recommendations/yahoo`
- Query: same as `/rank/yahoo`
- Returns a ranked list of `{ symbol, metrics }`.

### Finviz: Screener candidates
- Route: `/screener/candidates`
- Query:
  - `signal?`: `TopGainers|TopLosers|NewHigh|NewLow` (default: `TopGainers`)
  - `order?`: `Price|MarketCap|Change` (default: `Price`)
  - `screener?`: `Performance|Financial|Ownership` (default: `Performance`)
  - `limit?` (int, default: `25`)
- Example:
```bash
curl "http://localhost:3000/screener/candidates?signal=TopGainers&order=Price&screener=Performance&limit=25"
```

### Finviz: Recommendations (auto-evaluate candidates)
- Route: `/recommendations/finviz`
- Query (Finviz): `signal?`, `order?`, `screener?`, `limit?`
- Query (metrics): `range?`, `interval?`, `rf_annual?`, `target_return_annual?`, `periods_per_year?`
- Evaluates each candidate via Yahoo quotes and returns ranked `{ symbol, metrics }`.

### Options: Dynamic recommendations (auto-source symbols)
- Route: `/options/recommendations`
- Symbols (optional):
  - `symbols`: comma-separated symbols
  - `symbol`: single symbol
  - If omitted, symbols are sourced from Finviz using `signal|order|screener|symbols_limit`.
- Underlying metrics:
  - `range?`: `1mo|3mo|6mo|1y|2y|5y` (default: `3mo`)
  - `interval?`: `1d|1wk|1mo` (default: `1d`)
  - `rf_annual?`: (default: `0.03`)
  - Composite weights: `sharpe_w?` (default `0.4`), `sortino_w?` (default `0.4`), `calmar_w?` (default `0.2`)
- Option filters:
  - `side?`: `call|put|both` (default: `both`)
  - `min_dte?`, `max_dte?` (days to expiry; default: `7..60`)
  - `min_delta?`, `max_delta?`
  - `min_premium?`, `max_premium?`
  - `min_volume?`, `min_oi?`
  - `min_strike_ratio?`, `max_strike_ratio?` (strike / spot)
  - Spread quality: `max_spread_pct?` (bid-ask / mid)
- Ranking/output controls:
  - `per_symbol_limit?`: cap top contracts per underlying (default: unlimited)
  - `limit?`: total contracts to return (default: `20`)
- Response per contract:
  - `symbol`, `contract`, `side`, `strike`, `expiration`, `dte_days`, `premium`, `mid`, `spread`, `spread_pct`, `implied_vol`, `delta`, `leverage`, `volume`, `open_interest`, `strike_ratio`, `score`, `underlying_metrics`
- Example (fully automatic):
```bash
curl "http://localhost:3000/options/recommendations?side=both&min_dte=7&max_dte=45&min_volume=100&min_oi=200&min_strike_ratio=0.95&max_strike_ratio=1.05&signal=TopGainers&order=Price&screener=Performance&symbols_limit=30&per_symbol_limit=5&max_spread_pct=0.15&range=3mo&interval=1d&limit=50"
```
- Example (hand-picked symbols):
```bash
curl "http://localhost:3000/options/recommendations?symbols=AAPL,MSFT,NVDA&side=call&min_delta=0.2&max_delta=0.6&max_spread_pct=0.1&per_symbol_limit=10&range=6mo&interval=1d&sharpe_w=0.5&sortino_w=0.4&calmar_w=0.1&limit=40"
```

## Kraken Cryptocurrency Exchange

### Kraken: Ticker Information
- Route: `/kraken/ticker`
- Query: `pairs?` (comma-separated, default: XBT/USD,ETH/USD)
- Example:
```bash
curl "http://localhost:3000/kraken/ticker?pairs=XBT/USD,ETH/USD"
```

### Kraken: Ticker for Specific Pair
- Route: `/kraken/ticker/{pair}`
- Example:
```bash
curl "http://localhost:3000/kraken/ticker/XBT/USD"
```

### Kraken: Order Book
- Route: `/kraken/orderbook/{pair}`
- Query: `depth?` (int, default: 10)
- Example:
```bash
curl "http://localhost:3000/kraken/orderbook/XBT/USD?depth=20"
```

### Kraken: Assets
- Route: `/kraken/assets`
- Returns all available assets on Kraken
- Example:
```bash
curl "http://localhost:3000/kraken/assets"
```

### Kraken: Asset Pairs
- Route: `/kraken/pairs`
- Returns all available trading pairs on Kraken
- Example:
```bash
curl "http://localhost:3000/kraken/pairs"
```

### Kraken: Recent Trades
- Route: `/kraken/trades/{pair}`
- Query: `limit?` (int, default: 100), `since?` (timestamp)
- Example:
```bash
curl "http://localhost:3000/kraken/trades/XBT/USD?limit=50"
```

### Kraken: OHLC Data
- Route: `/kraken/ohlc/{pair}`
- Query: `interval?` (int, minutes: 1,5,15,30,60,240,1440,10080,21600), `since?` (timestamp)
- Example:
```bash
curl "http://localhost:3000/kraken/ohlc/XBT/USD?interval=60"
```

### Kraken: Trending Crypto
- Route: `/kraken/trending`
- Query: `limit?` (int)
- Example:
```bash
curl "http://localhost:3000/kraken/trending?limit=10"
```

### Kraken: Market Summary
- Route: `/kraken/summary/{pair}`
- Example:
```bash
curl "http://localhost:3000/kraken/summary/XBT/USD"
```

### Kraken: System Status
- Route: `/kraken/status`
- Returns Kraken system status
- Example:
```bash
curl "http://localhost:3000/kraken/status"
```

### Kraken: Server Time
- Route: `/kraken/time`
- Returns Kraken server time
- Example:
```bash
curl "http://localhost:3000/kraken/time"
```

## CoinGecko Cryptocurrency Market Data

### CoinGecko: Top Cryptocurrencies
- Route: `/coingecko/top`
- Query: `limit?` (int, default: 10)
- Returns top cryptocurrencies by market cap
- Example:
```bash
curl "http://localhost:3000/coingecko/top?limit=20"
```

### CoinGecko: Top Gainers
- Route: `/coingecko/gainers`
- Query: `limit?` (int, default: 10)
- Returns top gainers in the last 24 hours
- Example:
```bash
curl "http://localhost:3000/coingecko/gainers?limit=5"
```

### CoinGecko: Top Losers
- Route: `/coingecko/losers`
- Query: `limit?` (int, default: 10)
- Returns top losers in the last 24 hours
- Example:
```bash
curl "http://localhost:3000/coingecko/losers?limit=5"
```

### CoinGecko: Trending Cryptocurrencies
- Route: `/coingecko/trending`
- Returns trending cryptocurrencies based on social media and search data
- Example:
```bash
curl "http://localhost:3000/coingecko/trending"
```

### CoinGecko: Market Overview
- Route: `/coingecko/market-overview`
- Returns comprehensive market statistics including total market cap, volume, and Bitcoin dominance
- Example:
```bash
curl "http://localhost:3000/coingecko/market-overview"
```

### CoinGecko: Market Context
- Route: `/coingecko/market-context`
- Returns formatted market context for AI analysis
- Example:
```bash
curl "http://localhost:3000/coingecko/market-context"
```

### CoinGecko: Trending Symbols
- Route: `/coingecko/trending-symbols`
- Returns just the symbols of trending cryptocurrencies
- Example:
```bash
curl "http://localhost:3000/coingecko/trending-symbols"
```

### CoinGecko: Simple Price
- Route: `/coingecko/simple-price`
- Query: `ids` (required, comma-separated), `vs_currencies` (required, comma-separated), `include_24hr_change?` (bool, default: false)
- Returns simple price data for specific cryptocurrencies
- Example:
```bash
curl "http://localhost:3000/coingecko/simple-price?ids=bitcoin,ethereum&vs_currencies=usd,eur&include_24hr_change=true"
```

## High Open Interest Options Contracts

### High Open Interest: Single Ticker
- Route: `/high-open-interest/{ticker}`
- Query: `option_type?` (string, "call" or "put", default: "call")
- Returns high open interest contracts for both short-term (1-60 days) and LEAP (1-2 years) expirations
- Example:
```bash
curl "http://localhost:3000/high-open-interest/AAPL?option_type=call"
```

### High Open Interest: Batch Processing
- Route: `/high-open-interest/batch`
- Query: `tickers` (required, comma-separated), `option_type?` (string, "call" or "put", default: "call")
- Returns high open interest contracts for multiple tickers
- Example:
```bash
curl "http://localhost:3000/high-open-interest/batch?tickers=AAPL,TSLA,META&option_type=call"
```

## Trending Options Analysis

### Trending Options: Comprehensive Analysis
- Route: `/trending-options`
- Query: `option_type?` (string, "call", "put", or "both", default: "call"), `rf_annual?` (float, default: 0.03), `range?` (string, default: "3mo"), `interval?` (string, default: "1d"), `sharpe_w?` (float, default: 0.4), `sortino_w?` (float, default: 0.4), `calmar_w?` (float, default: 0.2), `limit?` (int, default: 10), `min_underlying_score?` (float, default: 0.0), `min_undervalued_score?` (float, default: 0.0)
- Returns trending tickers with comprehensive options analysis including underlying metrics and undervalued indicators
- Example:
```bash
curl "http://localhost:3000/trending-options?option_type=call&limit=5&min_underlying_score=0.5"
```

## Notes & Disclaimers
- Yahoo options data is fetched from the unofficial options JSON. This may change or rate-limit unexpectedly.
- Finviz data is scraped via `finviz_rs`. Respect Finviz’s robots.txt/ToS and avoid aggressive usage.
- This software is for educational/research purposes and not financial advice. Use at your own risk.
- CoinGecko API has rate limits (10-50 calls per minute for free tier). The API includes proper error handling for rate limiting scenarios.
- Kraken API provides real-time cryptocurrency exchange data. Respect their rate limits and terms of service.
- Reddit API requires authentication for trending data. Set up Reddit API credentials in your `.env` file.

## Project Structure
```
src/
  main.rs            # bootstrap: compose routers, state, tracing, Swagger
  state.rs           # AppState: shared clients and concurrency
  types.rs           # request/response DTOs (serde + utoipa schemas)
  errors.rs          # ApiError (IntoResponse)
  routes/            # HTTP layer (handlers only)
    system.rs        # /health
    data.rs          # /news, /forex, /crypto, /future, /insider, /group, /reddit/stocks, /trending/stocks
    yahoo.rs         # /metrics/yahoo, /rank/yahoo, /recommendations/yahoo
    options.rs       # /options/recommendations
    kraken.rs        # /kraken/* (ticker, orderbook, assets, trades, etc.)
    coingecko.rs     # /coingecko/* (top, gainers, losers, trending, etc.)
    high_open_interest.rs # /high-open-interest/* (high open interest option contracts)
    trending_options.rs # /trending-options (trending tickers with undervalued options analysis)
  services/          # business logic
    yahoo.rs         # price fetch, latest close, metrics helpers
  sources/           # external data clients
    finviz_data.rs   # Finviz screeners/news/groups
    yahoo_data.rs    # Yahoo options JSON + trending/predefined lists
    reddit_data.rs   # Reddit trending tickers
    alpaca_data.rs   # Alpaca news & options snapshots
    kraken_data.rs   # Kraken exchange data
    coingecko_data.rs # CoinGecko market data
  helpers/           # pure utilities
    metrics.rs       # Sharpe, Sortino, Calmar, Kelly, composite
    options.rs       # Black‑Scholes delta
    params.rs        # intervals, CSV parsing
    trending_cryptos.rs # Aggregated trending crypto from multiple sources
    high_open_interest.rs # High open interest option contracts from Alpaca
    trending_options.rs # Trending tickers with options analysis and undervalued indicators
```

## Contributing
- PRs welcome for:
  - Additional data sources
  - More robust error handling, caching, and rate limiting
  - CORS/config toggles and auth
  - Unit/integration tests

## Troubleshooting
- Build issues: ensure a recent Rust toolchain and run `cargo clean && cargo build`.
- Docs missing: confirm the app merged Swagger UI and visit `/docs`.
- Network errors: Yahoo/Finviz endpoints may rate‑limit; backoff and try again.
- CoinGecko rate limiting: Free tier allows 10-50 calls per minute. Consider upgrading to Pro for higher limits.
- Kraken API issues: Check system status at `/kraken/status` if endpoints fail.
- Reddit API errors: Ensure Reddit API credentials are properly configured in `.env`.
- macOS OpenSSL issues: `reqwest` uses `rustls-tls` to avoid system OpenSSL dependencies.
