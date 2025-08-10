# CoinGecko Integration

This document describes the CoinGecko integration for the trading API, which provides comprehensive cryptocurrency market data and analysis capabilities.

## Overview

The CoinGecko integration provides access to real-time cryptocurrency market data including:
- Top cryptocurrencies by market cap
- Top gainers and losers
- Trending cryptocurrencies
- Market overview statistics
- Simple price data
- Market context for AI analysis

## Features

### 1. Market Data
- **Top Coins**: Get top cryptocurrencies by market capitalization
- **Gainers/Losers**: Get top performing and worst performing cryptocurrencies
- **Trending**: Get trending cryptocurrencies based on social media and search data
- **Market Overview**: Get comprehensive market statistics including total market cap, volume, and Bitcoin dominance

### 2. Price Data
- **Simple Price**: Get current prices for specific cryptocurrencies in multiple currencies
- **Historical Data**: Get price change percentages for 24h, 7d, and 30d periods
- **Sparkline Data**: Get 7-day price sparkline data for charts

### 3. Market Analysis
- **Market Context**: Generate comprehensive market context for AI analysis
- **Market Cap Distribution**: Get market cap and volume percentages for different cryptocurrencies
- **Bitcoin Dominance**: Calculate Bitcoin's market dominance

## API Endpoints

### Base URL
All CoinGecko endpoints are available under `/coingecko/`

### Endpoints

#### 1. Top Cryptocurrencies
```
GET /coingecko/top?limit=10
```
Returns top cryptocurrencies by market cap.

**Parameters:**
- `limit` (optional): Number of coins to return (default: 10)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "bitcoin",
      "symbol": "btc",
      "name": "Bitcoin",
      "current_price": 45000.0,
      "market_cap": 850000000000,
      "price_change_percentage_24h": 2.5,
      "price_change_percentage_7d_in_currency": 5.2,
      "price_change_percentage_30d_in_currency": -3.1
    }
  ],
  "timestamp": 1640995200
}
```

#### 2. Top Gainers
```
GET /coingecko/gainers?limit=5
```
Returns top gainers in the last 24 hours.

**Parameters:**
- `limit` (optional): Number of coins to return (default: 10)

#### 3. Top Losers
```
GET /coingecko/losers?limit=5
```
Returns top losers in the last 24 hours.

**Parameters:**
- `limit` (optional): Number of coins to return (default: 10)

#### 4. Trending Cryptocurrencies
```
GET /coingecko/trending
```
Returns trending cryptocurrencies based on social media and search data.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "item": {
        "id": "bitcoin",
        "name": "Bitcoin",
        "symbol": "btc",
        "market_cap_rank": 1,
        "score": 1000
      }
    }
  ],
  "timestamp": 1640995200
}
```

#### 5. Market Overview
```
GET /coingecko/market-overview
```
Returns comprehensive market statistics.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_market_cap": 2000000000000,
    "total_volume": 100000000000,
    "bitcoin_dominance": 45.5,
    "market_cap_percentage": {
      "btc": 45.5,
      "eth": 18.2
    },
    "volume_percentage": {
      "btc": 35.0,
      "eth": 25.0
    }
  },
  "timestamp": 1640995200
}
```

#### 6. Market Context
```
GET /coingecko/market-context
```
Returns formatted market context for AI analysis.

**Response:**
```json
{
  "success": true,
  "context": "TOP 10 CRYPTOCURRENCIES BY MARKET CAP:\n1. Bitcoin (BTC): $45,000.00 | 24h: +2.50% | 7d: +5.20% | 30d: -3.10%\n...",
  "timestamp": 1640995200
}
```

#### 7. Trending Symbols
```
GET /coingecko/trending-symbols
```
Returns just the symbols of trending cryptocurrencies.

**Response:**
```json
{
  "success": true,
  "data": ["BTC", "ETH", "ADA", "SOL", "DOT"],
  "timestamp": 1640995200
}
```

#### 8. Simple Price
```
GET /coingecko/simple-price?ids=bitcoin,ethereum&vs_currencies=usd,eur&include_24hr_change=true
```
Returns simple price data for specific cryptocurrencies.

**Parameters:**
- `ids`: Comma-separated list of cryptocurrency IDs
- `vs_currencies`: Comma-separated list of target currencies
- `include_24hr_change` (optional): Include 24h price change (default: false)

**Response:**
```json
{
  "success": true,
  "data": {
    "bitcoin": {
      "usd": 45000,
      "eur": 40000,
      "usd_24h_change": 2.5
    },
    "ethereum": {
      "usd": 3000,
      "eur": 2700,
      "usd_24h_change": 1.8
    }
  },
  "timestamp": 1640995200
}
```

## Data Structures

### CoinGeckoCoin
```rust
pub struct CoinGeckoCoin {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub total_volume: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_7d_in_currency: Option<f64>,
    pub price_change_percentage_30d_in_currency: Option<f64>,
    // ... other fields
}
```

### MarketOverview
```rust
pub struct MarketOverview {
    pub total_market_cap: f64,
    pub total_volume: f64,
    pub bitcoin_dominance: f64,
    pub market_cap_percentage: HashMap<String, f64>,
    pub volume_percentage: HashMap<String, f64>,
}
```

### TrendingCoin
```rust
pub struct TrendingCoin {
    pub item: TrendingItem,
}

pub struct TrendingItem {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: u32,
    pub score: u32,
    // ... other fields
}
```

## Usage Examples

### Using the API Client

```rust
use trading_api::sources::coingecko_data::CoinGeckoClient;

#[tokio::main]
async fn main() {
    let client = CoinGeckoClient::new();
    
    // Get top 10 cryptocurrencies
    match client.get_top_coins(10).await {
        Ok(coins) => {
            for coin in coins {
                println!("{} ({}): ${:,.2f}", 
                    coin.name, 
                    coin.symbol.to_uppercase(),
                    coin.current_price.unwrap_or(0.0)
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Using Convenience Functions

```rust
use trading_api::sources::coingecko_data::{
    get_top_coins, get_market_overview, get_market_context
};

#[tokio::main]
async fn main() {
    // Get top coins
    let coins = get_top_coins(5).await?;
    
    // Get market overview
    let overview = get_market_overview().await?;
    
    // Get market context for AI analysis
    let context = get_market_context().await?;
}
```

### Integration with Trending Cryptos

```rust
use trading_api::helpers::trending_cryptos::get_trending_cryptos_coingecko;

#[tokio::main]
async fn main() {
    // Get trending cryptos from CoinGecko
    let trending = get_trending_cryptos_coingecko(10).await;
    
    // Get aggregated trending from all sources
    let aggregated = get_trending_cryptos_aggregated(20).await;
}
```

## Error Handling

All functions return `Result<T, String>` where the error is a descriptive message. Common error scenarios:

- **API Rate Limiting**: CoinGecko has rate limits that may cause temporary failures
- **Network Issues**: Connection timeouts or network errors
- **Invalid Parameters**: Invalid cryptocurrency IDs or currency codes
- **API Changes**: CoinGecko API structure changes

## Rate Limiting

CoinGecko has rate limits:
- Free tier: 10-50 calls per minute
- Pro tier: Higher limits available

The client includes proper error handling and retry logic for rate limit scenarios.

## Integration with AI Analysis

The market context endpoint provides formatted data specifically designed for AI analysis:

```rust
// Get market context for AI
let context = get_market_context().await?;

// Use with AI system
let ai_prompt = format!(
    "Analyze the current crypto market:\n\n{}",
    context
);
```

## Testing

Run the example to test the integration:

```bash
cargo run --example coingecko_example
```

## API Documentation

Full API documentation is available at `/docs` when the server is running, including:
- Interactive Swagger UI
- Request/response examples
- Parameter descriptions
- Error codes

## Dependencies

The CoinGecko integration requires:
- `reqwest`: HTTP client
- `serde`: Serialization/deserialization
- `serde_json`: JSON handling
- `chrono`: Timestamp handling

## Contributing

When adding new features to the CoinGecko integration:

1. Add new data structures to `src/sources/coingecko_data.rs`
2. Add corresponding API endpoints to `src/routes/coingecko.rs`
3. Update OpenAPI documentation in `src/lib.rs`
4. Add tests and examples
5. Update this documentation

## License

This integration follows the same license as the main trading API project.
