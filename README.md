# Trading API (Rust + Axum)

A lightweight HTTP API for sourcing market data and generating trading recommendations. It combines Finviz screening with Yahoo Finance historical and options data, then ranks assets/options using a composite of Kelly-inspired position sizing with Sharpe, Sortino, and Calmar risk-adjusted metrics.

## Highlights
- GET-only REST API built with Axum + Tokio
- Data sources:
  - Finviz via `finviz_rs` (screeners, news, groups, etc.)
  - Yahoo Finance via `yahoo_finance_api` (historical quotes) and options JSON endpoint
- Metrics engine (`src/metrics.rs`): Sharpe, Sortino, Calmar, Kelly fraction, composite score
- Dynamic options recommendations without specifying symbols (auto-sources tickers from Finviz)

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
- Implemented in `src/metrics.rs`:
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
curl "http://localhost:3000/news?limit=5"
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

## Notes & Disclaimers
- Yahoo options data is fetched from the unofficial options JSON. This may change or rate-limit unexpectedly.
- Finviz data is scraped via `finviz_rs`. Respect Finviz’s robots.txt/ToS and avoid aggressive usage.
- This software is for educational/research purposes and not financial advice. Use at your own risk.

## Project Structure
```
src/
  main.rs        # axum routes and handlers
  metrics.rs     # metrics engine (Sharpe, Sortino, Calmar, Kelly, composites)
Cargo.toml       # dependencies
```

## Contributing
- PRs welcome for:
  - Additional data sources
  - More robust error handling, caching, and rate limiting
  - CORS/config toggles and auth
  - Unit/integration tests

## Troubleshooting
- Build issues: ensure a recent Rust toolchain and run `cargo clean && cargo build`.
- Network errors: Yahoo/Finviz endpoints may rate-limit; backoff and try again.
- macOS OpenSSL issues: `reqwest` is configured with `rustls-tls` to avoid system OpenSSL dependencies.
