# High Open Interest Contracts

This module provides functionality to fetch high open interest option contracts from Alpaca Markets. It helps identify the most actively traded options for a given underlying stock.

## Features

- Fetch high open interest contracts for both short-term (1-60 days) and LEAP (1-2 years) expirations
- Support for both call and put options
- Automatic ticker symbol normalization (e.g., FB → META)
- Batch processing for multiple tickers
- Price data integration with open interest data

## API Endpoints

### Single Ticker

```
GET /high-open-interest/{ticker}?option_type={call|put}
```

**Parameters:**
- `ticker` (path): Stock ticker symbol (e.g., AAPL, TSLA, META)
- `option_type` (query, optional): Option type - "call" or "put" (default: "call")

**Response:**
```json
{
  "ticker": "AAPL",
  "result": {
    "short_term": {
      "symbol": "AAPL240119C00170000",
      "underlying_symbol": "AAPL",
      "strike_price": 170.0,
      "expiration_date": "2024-01-19",
      "type": "call",
      "open_interest": 12345,
      "open_interest_date": "2024-01-15",
      "close_price": 2.50,
      "close_price_date": "2024-01-15",
      "ask_price": 2.55,
      "bid_price": 2.45,
      "last_price": 2.50,
      "implied_volatility": 0.25
    },
    "leap": {
      "symbol": "AAPL250117C00180000",
      "underlying_symbol": "AAPL",
      "strike_price": 180.0,
      "expiration_date": "2025-01-17",
      "type": "call",
      "open_interest": 5678,
      "open_interest_date": "2024-01-15",
      "close_price": 15.20,
      "close_price_date": "2024-01-15",
      "ask_price": 15.30,
      "bid_price": 15.10,
      "last_price": 15.20,
      "implied_volatility": 0.30
    },
    "error": null
  }
}
```

### Batch Processing

```
GET /high-open-interest/batch?tickers=AAPL,TSLA,META&option_type=call
```

**Parameters:**
- `tickers` (query): Comma-separated list of stock ticker symbols
- `option_type` (query, optional): Option type - "call" or "put" (default: "call")

**Response:**
```json
[
  {
    "ticker": "AAPL",
    "result": { ... }
  },
  {
    "ticker": "TSLA", 
    "result": { ... }
  },
  {
    "ticker": "META",
    "result": { ... }
  }
]
```

## Usage Examples

### Using the Helper Functions

```rust
use trading_api::helpers::high_open_interest::get_high_open_interest_contracts;

// Get high open interest call contracts for AAPL
let result = get_high_open_interest_contracts("AAPL", Some("call")).await;

// Get high open interest put contracts for TSLA
let result = get_high_open_interest_contracts("TSLA", Some("put")).await;

// Get high open interest call contracts for multiple tickers
let tickers = vec!["AAPL".to_string(), "TSLA".to_string(), "META".to_string()];
let results = get_high_open_interest_contracts_batch(&tickers, Some("call")).await;
```

### Running the Example

```bash
cargo run --example high_open_interest_example
```

## Configuration

The following environment variables are required:

- `ALPACA_API_KEY_ID` or `APCA_API_KEY_ID`: Your Alpaca API key
- `ALPACA_API_SECRET_KEY` or `APCA_API_SECRET_KEY`: Your Alpaca API secret

## Error Handling

The API handles various error scenarios:

- **Rate Limiting**: Returns appropriate error messages when Alpaca rate limits are hit
- **Invalid Tickers**: Provides helpful error messages for invalid symbols (including FB → META conversion)
- **No Contracts**: Gracefully handles cases where no contracts are found
- **Network Errors**: Comprehensive error reporting for network issues

## Data Sources

- **Alpaca Markets**: Primary data source for option contracts and pricing
- **Real-time Data**: Uses Alpaca's v2 options API for current market data
- **Open Interest**: Prioritizes contracts with the highest open interest

## Limitations

- Requires valid Alpaca API credentials
- Subject to Alpaca's rate limiting policies
- Limited to US equity options
- Some ticker symbols may have limited option availability

## Rate Limiting

The implementation includes built-in rate limiting protection:

- Automatic delays between batch requests (100ms)
- Graceful handling of 429 status codes
- Error reporting for rate limit violations

## Ticker Symbol Handling

Special handling for certain ticker symbols:

- **FB → META**: Automatic conversion for Meta Platforms (formerly Facebook)
- **Case Sensitivity**: Ticker symbols are processed as-is from the API
- **Validation**: Invalid symbols return appropriate error messages
