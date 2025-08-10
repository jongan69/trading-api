# Kraken Integration

This document describes the Kraken cryptocurrency exchange integration in the trading API.

## Overview

The Kraken integration provides access to real-time cryptocurrency market data through Kraken's REST API and WebSocket feeds. It includes functionality for:

- Real-time ticker data
- Order book information
- Recent trades
- OHLC (Open/High/Low/Close) data
- Asset and pair information
- Trending cryptocurrencies by volume
- System status and server time

## Features

### REST API Integration
- **Ticker Data**: Get current prices, volume, and 24h changes for any trading pair
- **Order Books**: Retrieve bid/ask orders with customizable depth
- **Recent Trades**: Access recent trade history for any pair
- **OHLC Data**: Get historical price data with various time intervals
- **Asset Information**: Retrieve details about all available assets
- **Pair Information**: Get comprehensive information about trading pairs

### WebSocket Integration
- **Real-time Order Books**: Subscribe to live order book updates
- **Automatic Reconnection**: Built-in connection management
- **Checksum Validation**: Ensures data integrity

### Trending Analysis
- **Volume-based Trending**: Identify top cryptocurrencies by trading volume
- **Multi-source Aggregation**: Combine data from multiple sources
- **Customizable Limits**: Configurable result limits

## API Endpoints

### Kraken-specific Endpoints

#### GET `/kraken/ticker`
Get ticker information for specified pairs.

**Query Parameters:**
- `pairs` (optional): Comma-separated list of trading pairs (default: XBT/USD, ETH/USD)

**Example:**
```bash
curl "http://localhost:3000/kraken/ticker?pairs=XBT/USD,ETH/USD"
```

#### GET `/kraken/ticker/{pair}`
Get ticker information for a specific pair.

**Example:**
```bash
curl "http://localhost:3000/kraken/ticker/XBT/USD"
```

#### GET `/kraken/orderbook/{pair}`
Get order book for a specific pair.

**Query Parameters:**
- `depth` (optional): Order book depth (default: 10)

**Example:**
```bash
curl "http://localhost:3000/kraken/orderbook/XBT/USD?depth=5"
```

#### GET `/kraken/assets`
Get all available assets.

**Example:**
```bash
curl "http://localhost:3000/kraken/assets"
```

#### GET `/kraken/pairs`
Get all available asset pairs.

**Example:**
```bash
curl "http://localhost:3000/kraken/pairs"
```

#### GET `/kraken/trades/{pair}`
Get recent trades for a specific pair.

**Query Parameters:**
- `since` (optional): Return committed OHLC data since given ID

**Example:**
```bash
curl "http://localhost:3000/kraken/trades/XBT/USD"
```

#### GET `/kraken/ohlc/{pair}`
Get OHLC data for a specific pair.

**Query Parameters:**
- `interval` (optional): Time frame interval in minutes
- `since` (optional): Return committed OHLC data since given ID

**Example:**
```bash
curl "http://localhost:3000/kraken/ohlc/XBT/USD?interval=1"
```

#### GET `/kraken/trending`
Get trending crypto pairs by volume.

**Query Parameters:**
- `limit` (optional): Number of results to return (default: 10)

**Example:**
```bash
curl "http://localhost:3000/kraken/trending?limit=20"
```

#### GET `/kraken/summary/{pair}`
Get comprehensive market summary for a specific pair.

**Example:**
```bash
curl "http://localhost:3000/kraken/summary/XBT/USD"
```

#### GET `/kraken/status`
Get Kraken system status.

**Example:**
```bash
curl "http://localhost:3000/kraken/status"
```

#### GET `/kraken/time`
Get Kraken server time.

**Example:**
```bash
curl "http://localhost:3000/kraken/time"
```

### General Data Endpoints

#### GET `/trending/crypto`
Get trending cryptocurrencies from multiple sources.

**Query Parameters:**
- `limit` (optional): Number of results to return (default: 10)

**Example:**
```bash
curl "http://localhost:3000/trending/crypto?limit=15"
```

## Data Structures

### KrakenTicker
```rust
pub struct KrakenTicker {
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub change_24h: f64,
    pub change_pct_24h: f64,
}
```

### KrakenOrderBook
```rust
pub struct KrakenOrderBook {
    pub pair: String,
    pub bids: Vec<(f64, f64)>, // (price, volume)
    pub asks: Vec<(f64, f64)>, // (price, volume)
    pub timestamp: u64,
}
```

### KrakenAsset
```rust
pub struct KrakenAsset {
    pub name: String,
    pub altname: String,
    pub aclass: String,
    pub decimals: u32,
    pub display_decimals: u32,
}
```

### KrakenAssetPair
```rust
pub struct KrakenAssetPair {
    pub name: String,
    pub altname: String,
    pub wsname: String,
    pub aclass_base: String,
    pub base: String,
    pub aclass_quote: String,
    pub quote: String,
    pub lot: String,
    pub pair_decimals: u32,
    pub lot_decimals: u32,
    pub lot_multiplier: u32,
    pub leverage_buy: Vec<u32>,
    pub leverage_sell: Vec<u32>,
    pub fees: Vec<(f64, f64)>,
    pub fees_maker: Vec<(f64, f64)>,
    pub fee_volume_currency: String,
    pub margin_call: u32,
    pub margin_stop: u32,
    pub ordermin: String,
}
```

## Usage Examples

### Basic Usage
```rust
use trading_api::sources::kraken_data::KrakenDataSource;

// Initialize the data source
let data_source = KrakenDataSource::new()?;

// Get ticker data for Bitcoin
let tickers = data_source.get_tickers(vec!["XBT/USD".to_string()])?;
for ticker in tickers {
    println!("{}: ${:.2}", ticker.pair, ticker.price);
}
```

### WebSocket Real-time Data
```rust
use trading_api::sources::kraken_data::KrakenWebSocketFeed;

// Create WebSocket feed
let feed = KrakenWebSocketFeed::new(
    vec!["XBT/USD".to_string()], 
    10
)?;

// Get real-time order books
let order_books = feed.get_order_books();
for (pair, book) in order_books {
    println!("Order book for {}: {} bids, {} asks", 
        pair, book.bids.len(), book.asks.len());
}
```

### Trending Cryptocurrencies
```rust
use trading_api::sources::kraken_data::get_trending_crypto_pairs;

// Get top 10 trending cryptos by volume
let trending = get_trending_crypto_pairs(10).await?;
for pair in trending {
    println!("Trending: {}", pair);
}
```

## Error Handling

The Kraken integration includes comprehensive error handling:

- **Network Errors**: Automatic retry logic for transient failures
- **Rate Limiting**: Respects Kraken's API rate limits
- **Data Validation**: Validates responses before processing
- **Graceful Degradation**: Continues operation even if some endpoints fail

## Rate Limits

Kraken's API has the following rate limits:
- **Public API**: 1 request per second
- **Private API**: 3 requests per 5 seconds
- **WebSocket**: No rate limits

The integration automatically handles rate limiting by implementing appropriate delays between requests.

## Dependencies

The Kraken integration uses the following dependencies:
- `krakenrs`: Unofficial Rust bindings for Kraken API
- `serde`: Serialization/deserialization
- `tokio`: Async runtime
- `tracing`: Logging

## Testing

Run the Kraken integration tests:
```bash
cargo test kraken_data
```

Run the example:
```bash
cargo run --example kraken_example
```

## Contributing

When contributing to the Kraken integration:

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation
4. Ensure error handling is comprehensive
5. Respect rate limits in tests

## License

This integration is part of the trading-api project and follows the same license terms.
