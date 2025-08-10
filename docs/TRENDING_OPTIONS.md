# Trending Options Analysis

This module provides comprehensive analysis of trending tickers combined with undervalued options identification. It aggregates trending stocks from multiple sources, analyzes their underlying metrics using your established risk-adjusted return framework, and identifies high open interest options that may be undervalued.

## Features

- **Multi-source trending tickers**: Combines data from Finviz, Yahoo Finance, and Reddit
- **Underlying metrics analysis**: Applies Sharpe, Sortino, Calmar, and Kelly metrics
- **High open interest options**: Identifies the most actively traded options
- **Undervalued indicators**: Calculates liquidity, spread, and momentum scores
- **Comprehensive scoring**: Combines underlying performance with options characteristics
- **Batch processing**: Efficiently analyzes multiple tickers concurrently

## API Endpoint

### Trending Options Analysis

```
GET /trending-options
```

**Query Parameters:**
- `option_type` (optional): "call", "put", or "both" (default: "call")
- `rf_annual` (optional): Risk-free annual rate (default: 0.03)
- `range` (optional): Time range for underlying analysis: "1mo", "3mo", "6mo", "1y", "2y", "5y" (default: "3mo")
- `interval` (optional): Time interval: "1d", "1wk", "1mo" (default: "1d")
- `sharpe_w` (optional): Weight for Sharpe ratio (default: 0.4)
- `sortino_w` (optional): Weight for Sortino ratio (default: 0.4)
- `calmar_w` (optional): Weight for Calmar ratio (default: 0.2)
- `limit` (optional): Maximum results to return (default: 10)
- `min_underlying_score` (optional): Minimum underlying composite score (default: 0.0)
- `min_undervalued_score` (optional): Minimum undervalued score (default: 0.0)

**Response:**
```json
{
  "results": [
    {
      "symbol": "AAPL",
      "underlying_metrics": {
        "symbol": "AAPL",
        "spot_price": 175.50,
        "metrics": {
          "composite_score": 0.85,
          "sharpe": 1.2,
          "sortino": 1.5,
          "calmar": 0.8,
          "kelly_fraction": 0.3,
          "volatility": 0.25,
          "max_drawdown": 0.15,
          "cagr": 0.12,
          "n_periods": 63
        }
      },
      "options_analysis": [
        {
          "contract_type": "short_term",
          "contract": {
            "symbol": "AAPL240119C00175000",
            "underlying_symbol": "AAPL",
            "strike_price": 175.0,
            "expiration_date": "2024-01-19",
            "type": "call",
            "open_interest": 1234,
            "ask_price": 3.50,
            "bid_price": 3.45,
            "last_price": 3.48,
            "implied_volatility": 0.25
          },
          "option_score": 0.72,
          "undervalued_indicators": {
            "liquidity_score": 1.0,
            "spread_score": 0.9,
            "underlying_momentum": 0.85,
            "overall_undervalued_score": 0.91,
            "spread_percentage": 0.014,
            "open_interest": 1234,
            "is_liquid": true,
            "is_tight_spread": true,
            "has_momentum": true
          }
        },
        {
          "contract_type": "leap",
          "contract": {
            "symbol": "AAPL250117C00180000",
            "underlying_symbol": "AAPL",
            "strike_price": 180.0,
            "expiration_date": "2025-01-17",
            "type": "call",
            "open_interest": 567,
            "ask_price": 18.50,
            "bid_price": 18.30,
            "last_price": 18.40,
            "implied_volatility": 0.30
          },
          "option_score": 0.65,
          "undervalued_indicators": {
            "liquidity_score": 0.7,
            "spread_score": 0.8,
            "underlying_momentum": 0.85,
            "overall_undervalued_score": 0.81,
            "spread_percentage": 0.011,
            "open_interest": 567,
            "is_liquid": true,
            "is_tight_spread": true,
            "has_momentum": true
          }
        }
      ],
      "error": null
    }
  ],
  "summary": {
    "total_analyzed": 15,
    "total_with_options": 12,
    "average_underlying_score": 0.73,
    "average_undervalued_score": 0.68,
    "top_underlying_tickers": ["AAPL", "TSLA", "NVDA", "META", "MSFT"],
    "top_undervalued_tickers": ["AAPL", "TSLA", "NVDA", "META", "AMD"]
  }
}
```

## Usage Examples

### Basic Usage

```bash
# Get trending options analysis for call options
curl "http://localhost:3000/trending-options?option_type=call&limit=5"

# Get trending options analysis for put options
curl "http://localhost:3000/trending-options?option_type=put&limit=5"

# Filter by minimum scores
curl "http://localhost:3000/trending-options?min_underlying_score=0.5&min_undervalued_score=0.6"
```

### Advanced Usage

```bash
# Custom metrics weights and time range
curl "http://localhost:3000/trending-options?sharpe_w=0.5&sortino_w=0.3&calmar_w=0.2&range=6mo&interval=1d"

# High-quality options only
curl "http://localhost:3000/trending-options?min_underlying_score=0.7&min_undervalued_score=0.8&limit=3"
```

### Using the Helper Functions

```rust
use trading_api::helpers::trending_options::get_trending_options_analysis;
use trading_api::helpers::metrics::CompositeWeights;

// Configure analysis parameters
let weights = CompositeWeights {
    sharpe: 0.4,
    sortino: 0.4,
    calmar: 0.2,
};

// Get trending options analysis
let results = get_trending_options_analysis(
    &state,
    0.03, // rf_annual
    252,  // periods_per_year (daily)
    &weights,
    Some("call"),
    10,   // limit
).await;
```

## Analysis Methodology

### 1. Trending Ticker Aggregation

The system aggregates trending tickers from multiple sources:

- **Finviz**: Top gainers, losers, and most active stocks
- **Yahoo Finance**: Trending stocks and predefined lists
- **Reddit**: Community-driven trending stocks

Tickers are deduplicated while preserving order from the most reliable sources.

### 2. Underlying Metrics Analysis

For each trending ticker, the system calculates comprehensive risk-adjusted metrics:

- **Sharpe Ratio**: Excess return per unit of total volatility
- **Sortino Ratio**: Excess return per unit of downside deviation
- **Calmar Ratio**: CAGR divided by maximum drawdown
- **Kelly Fraction**: Optimal position sizing based on return/variance ratio
- **Composite Score**: Weighted combination of the above metrics

### 3. High Open Interest Options Identification

For each ticker with good underlying metrics, the system:

- Fetches high open interest contracts for both short-term (1-60 days) and LEAP (1-2 years) expirations
- Retrieves current pricing data including bid/ask spreads
- Calculates option-specific metrics and scores

### 4. Undervalued Indicators

Each option contract is evaluated using multiple indicators:

- **Liquidity Score**: Based on open interest (higher = more liquid)
- **Spread Score**: Based on bid-ask spread percentage (tighter = better)
- **Underlying Momentum**: Based on the underlying stock's composite score
- **Overall Undervalued Score**: Weighted combination of the above

### 5. Final Scoring and Ranking

Options are ranked using a combination of:

- Underlying stock performance (composite score)
- Option-specific characteristics (delta, leverage, time decay)
- Undervalued indicators (liquidity, spread, momentum)

## Configuration

The following environment variables are required:

- `ALPACA_API_KEY_ID` or `APCA_API_KEY_ID`: Your Alpaca API key
- `ALPACA_API_SECRET_KEY` or `APCA_API_SECRET_KEY`: Your Alpaca API secret
- `REDDIT_CLIENT_ID`, `REDDIT_CLIENT_SECRET`, etc.: For Reddit trending data (optional)

## Error Handling

The API handles various scenarios gracefully:

- **Missing data**: Tickers without sufficient price history are filtered out
- **No options available**: Tickers without options data are included but marked accordingly
- **Rate limiting**: Built-in delays and error handling for API rate limits
- **Network errors**: Comprehensive error reporting and fallback mechanisms

## Performance Considerations

- **Concurrent processing**: Multiple tickers are analyzed in parallel
- **Caching**: Underlying metrics are computed once per ticker
- **Rate limiting**: Built-in delays to respect API limits
- **Filtering**: Early filtering to reduce unnecessary API calls

## Limitations

- Requires valid Alpaca API credentials for options data
- Subject to rate limits from various data providers
- Limited to US equity options
- Some ticker symbols may have limited options availability
- Reddit trending data requires API credentials (optional)

## Running the Example

```bash
cargo run --example trending_options_example
```

This will demonstrate the full trending options analysis pipeline with real market data.
