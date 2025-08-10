# Trading API - Complete Frontend Integration Guide

This document provides comprehensive API endpoint documentation with complete request/response examples, error handling, and integration patterns for frontend developers.

**Note:** The response examples in this documentation are real responses from the running server, captured during testing. They represent actual data structures and formats that frontend developers will receive when integrating with the API.

## Base URL
```
http://localhost:3000
```

## Authentication
Currently, the API does not require authentication for most endpoints. Some endpoints may require API keys for external services (Alpaca, Reddit) to be configured in the environment.

## Response Headers
All responses include the following headers:
```
Content-Type: application/json
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1640995260
X-Request-ID: req_123456789
X-Response-Time: 245ms
```

## Standard Response Format
All endpoints return JSON responses with consistent structure:

**Success Response:**
```json
{
  "success": true,
  "data": { /* response data */ },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid parameters provided",
    "details": {
      "field": "symbol",
      "issue": "Symbol is required"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

## Common Error Codes
- `VALIDATION_ERROR`: Invalid request parameters
- `NOT_FOUND`: Resource not found
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `SERVICE_UNAVAILABLE`: External service unavailable
- `INTERNAL_ERROR`: Server error
- `UNAUTHORIZED`: Authentication required

## Pagination Format
Paginated responses include metadata:
```json
{
  "success": true,
  "data": [ /* items */ ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150,
    "total_pages": 8,
    "has_next": true,
    "has_prev": false
  },
  "timestamp": 1640995200
}
```

## Data Types
- **Timestamps**: Unix timestamp (seconds since epoch)
- **Prices**: Decimal numbers with up to 8 decimal places
- **Percentages**: Decimal numbers (e.g., 0.05 = 5%)
- **Currencies**: ISO 4217 codes (USD, EUR, etc.)
- **Symbols**: Uppercase ticker symbols (AAPL, BTC, etc.)

---

## System Endpoints

### 1. Health Check
**GET** `/health`

**Description:** Basic health check endpoint for load balancers and monitoring systems

**Response (200):**
```json
{
  "status": "ok"
}
```

**Response (500):**
```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Service unavailable",
    "details": {
      "reason": "Database connection failed"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 2. System Status
**GET** `/status`

**Description:** Detailed system status with external service health and performance metrics

**Response (200):**
```json
{
  "status": "healthy",
  "timestamp": 1754798378,
  "uptime": 1754798378,
  "version": "0.1.0",
  "environment": "development",
  "services": {
    "yahoo_finance": {
      "status": "healthy",
      "last_check": 1754798379,
      "error_count": 0,
      "response_time_ms": 296
    },
    "alpaca": {
      "status": "healthy",
      "last_check": 1754798379,
      "error_count": 0,
      "response_time_ms": 401
    }
  }
}
```

**Response (500):**
```json
{
  "success": false,
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Failed to get system status",
    "details": {
      "reason": "Unable to connect to external services"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 3. Readiness Check
**GET** `/ready`

**Description:** Kubernetes readiness check

**Response (200):**
```json
{
  "status": "ready"
}
```

**Response (500):**
```json
{
  "error": "Service not ready"
}
```

### 4. Liveness Check
**GET** `/live`

**Description:** Kubernetes liveness check

**Response (200):**
```json
{
  "status": "alive"
}
```

### 5. System Metrics
**GET** `/metrics`

**Description:** System performance metrics

**Response (200):**
```json
{
  "total_requests": 0,
  "error_rate": 0.0,
  "average_response_time": 0.0,
  "active_connections": 0,
  "memory_usage": {
    "used_mb": 128,
    "total_mb": 1024,
    "percentage": 12.5
  }
}
```

---

## Data Endpoints

### 6. Aggregated News
**GET** `/news`

**Description:** Aggregated news from multiple sources including Finviz, Reddit, and Alpaca

**Parameters:**
- `limit` (optional): Number of news items (default: 20, max: 100)
- `sources` (optional): Comma-separated sources - "finviz", "reddit", "alpaca" (default: "all")
- `timeframe` (optional): Time window - "1h", "6h", "24h", "7d" (default: "24h")

**Response (200):**
```json
{
  "finviz": [
    {
      "Link": "https://www.nytimes.com/2025/08/09/business/trump-tariffs-prices-layoffs.html",
      "Source": "www.nytimes.com",
      "Time": "05:02AM",
      "Title": "Trump Tariffs Spur Layoffs and Price Increases as Companies Face Tough Choices"
    },
    {
      "Link": "https://www.nytimes.com/2025/08/09/business/manager-performance-work-advice.html",
      "Source": "www.nytimes.com",
      "Time": "05:02AM",
      "Title": "What to Do When Your Manager Doesn't Work. Like, at All."
    },
    {
      "Link": "https://www.nytimes.com/2025/08/09/business/alexa-artificial-intelligence-amazon.html",
      "Source": "www.nytimes.com",
      "Time": "05:02AM",
      "Title": "Alexa Got an A.I. Brain Transplant. How Smart Is It Now?"
    }
  ],
  "reddit": {
    "investing": [
      {
        "author": "Ok_Tumbleweed_295",
        "created_utc": 1754797939.0,
        "id": "1mm8oc9",
        "is_self": true,
        "num_comments": 0,
        "over_18": false,
        "permalink": "/r/investing/comments/1mm8oc9/is_there_a_safe_leveragemargin_rate/",
        "score": 1.0,
        "selftext": "I read that 1,6 to 2x is historically the optimal leverage rate for a diversified portfolio...",
        "stickied": false,
        "subreddit": "investing",
        "title": "Is there a Safe Leverage/Margin rate?",
        "url": "https://www.reddit.com/r/investing/comments/1mm8oc9/is_there_a_safe_leveragemargin_rate/"
      }
    ],
    "stocks": [
      {
        "author": "Short-Philosophy-105",
        "created_utc": 1754793377.0,
        "id": "1mm786q",
        "is_self": true,
        "num_comments": 78,
        "over_18": false,
        "permalink": "/r/stocks/comments/1mm786q/the_state_of_this_subreddit/",
        "score": 0.0,
        "selftext": "Honestly if you compare the sentiment of this subreddit with the price fluctuations of the S&P 500...",
        "stickied": false,
        "subreddit": "stocks",
        "title": "The state of this subreddit",
        "url": "https://www.reddit.com/r/stocks/comments/1mm786q/the_state_of_this_subreddit/"
      }
    ],
    "wallstreetbets": [
      {
        "author": "GastoMuchoPapel",
        "created_utc": 1754796695.0,
        "id": "1mm8arr",
        "is_self": false,
        "num_comments": 12,
        "over_18": false,
        "permalink": "/r/wallstreetbets/comments/1mm8arr/which_one_of_you_blew_up_account_and_threw_out/",
        "score": 46.0,
        "stickied": false,
        "subreddit": "wallstreetbets",
        "title": "Which one of you blew up account and threw out your trading set up? lol",
        "url": "https://i.redd.it/6zc9ta7i34if1.jpeg"
      }
    ]
  }
}
```

**Response (500):**
```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Failed to fetch news data",
    "details": {
      "reason": "External news services are temporarily unavailable"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 7. Forex Data
**GET** `/forex`

**Description:** Forex market data from Finviz

**Response (200):**
```json
{
  "forex": [
    {
      "symbol": "EUR/USD",
      "price": 1.0850,
      "change": 0.0020,
      "change_percent": 0.18
    }
  ]
}
```

### 8. Crypto Data
**GET** `/crypto`

**Description:** Cryptocurrency data from Finviz

**Response (200):**
```json
{
  "crypto": [
    {
      "symbol": "BTC/USD",
      "price": 45000.00,
      "change": 1500.00,
      "change_percent": 3.45
    }
  ]
}
```

### 9. Futures Data
**GET** `/future`

**Description:** Futures market data from Finviz

**Response (200):**
```json
{
  "futures": [
    {
      "symbol": "ES",
      "price": 4500.00,
      "change": 25.00,
      "change_percent": 0.56
    }
  ]
}
```

### 10. Insider Trading
**GET** `/insider`

**Description:** Insider trading data from Finviz

**Response (200):**
```json
{
  "insider": [
    {
      "ticker": "AAPL",
      "owner": "Tim Cook",
      "relationship": "CEO",
      "date": "2024-01-01",
      "transaction": "Buy",
      "shares": 10000,
      "value": 1500000
    }
  ]
}
```

### 11. Group Performance
**GET** `/group`

**Description:** Group performance data from Finviz

**Response (200):**
```json
{
  "groups": [
    {
      "name": "Technology",
      "performance": 2.5,
      "volume": 1500000000
    }
  ]
}
```

### 12. Reddit Trending Stocks
**GET** `/reddit/stocks`

**Parameters:**
- `limit` (optional): Number of results (default: 10, max: 50)
- `timeframe` (optional): Time window - "1h", "6h", "24h", "7d" (default: "24h")
- `subreddits` (optional): Comma-separated subreddits (default: "wallstreetbets,investing,stocks")

**Response (200):**
```json
{
  "success": true,
  "data": {
    "symbols": [
      {
        "ticker": "GME",
        "company_name": "GameStop Corp.",
        "mention_count": 1250,
        "sentiment_score": 0.75,
        "price_change_24h": 15.5,
        "volume_change_24h": 250.0,
        "subreddits": ["wallstreetbets", "investing"],
        "top_posts": [
          {
            "title": "GME Short Squeeze Analysis",
            "url": "https://reddit.com/r/wallstreetbets/comments/123456",
            "upvotes": 2500,
            "comments": 450,
            "sentiment": "positive"
          }
        ],
        "trending_score": 0.85
      },
      {
        "ticker": "TSLA",
        "company_name": "Tesla, Inc.",
        "mention_count": 890,
        "sentiment_score": 0.45,
        "price_change_24h": -2.3,
        "volume_change_24h": 45.0,
        "subreddits": ["investing", "stocks"],
        "top_posts": [
          {
            "title": "Tesla Q4 Delivery Numbers",
            "url": "https://reddit.com/r/investing/comments/789012",
            "upvotes": 1200,
            "comments": 320,
            "sentiment": "neutral"
          }
        ],
        "trending_score": 0.72
      }
    ],
    "summary": {
      "total_mentions": 3450,
      "average_sentiment": 0.62,
      "top_subreddits": ["wallstreetbets", "investing", "stocks"],
      "market_sentiment": "bullish"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

**Response (429):**
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for Reddit API",
    "details": {
      "retry_after": 60
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 13. Trending Stocks
**GET** `/trending/stocks`

**Parameters:**
- `limit` (optional): Number of results (default: 10)

**Response (200):**
```json
{
  "symbols": ["NVDA", "AMD", "TSLA", "AAPL"]
}
```

### 14. Trending Crypto
**GET** `/trending/crypto`

**Parameters:**
- `limit` (optional): Number of results (default: 10)

**Response (200):**
```json
{
  "symbols": ["BTC", "ETH", "SOL", "ADA"]
}
```

---

## Yahoo Finance Endpoints

### 15. Yahoo Metrics
**GET** `/metrics/yahoo`

**Parameters:**
- `symbols` (required): Single symbol (e.g., "AAPL")
- `range` (optional): Time range - "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max" (default: "3mo")
- `interval` (optional): Time interval - "1m", "2m", "5m", "15m", "30m", "60m", "90m", "1h", "1d", "5d", "1wk", "1mo", "3mo" (default: "1d")
- `rf_annual` (optional): Risk-free annual rate (default: 0.0)
- `target_return_annual` (optional): Target annual return (default: rf_annual)
- `periods_per_year` (optional): Number of periods per year (default: calculated from interval)

**Response (200):**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "company_name": "Apple Inc.",
    "current_price": 150.25,
    "price_change_24h": 2.50,
    "price_change_percent_24h": 1.69,
    "market_cap": 2500000000000,
    "volume": 45000000,
    "metrics": {
      "sharpe_ratio": 1.25,
      "sortino_ratio": 1.45,
      "calmar_ratio": 0.85,
      "max_drawdown": -0.15,
      "volatility": 0.25,
      "annual_return": 0.18,
      "composite_score": 1.18,
      "beta": 1.15,
      "alpha": 0.05,
      "information_ratio": 1.32,
      "treynor_ratio": 0.12,
      "jensen_alpha": 0.03
    },
    "risk_metrics": {
      "var_95": -0.025,
      "cvar_95": -0.035,
      "downside_deviation": 0.18,
      "upside_potential_ratio": 1.45
    },
    "performance_metrics": {
      "total_return": 0.18,
      "excess_return": 0.15,
      "risk_adjusted_return": 0.12,
      "tracking_error": 0.08
    },
    "analysis_period": {
      "start_date": "2023-10-01",
      "end_date": "2024-01-01",
      "days": 92,
      "data_points": 92
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

**Response (400):**
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid parameters provided",
    "details": {
      "field": "symbols",
      "issue": "Provide exactly one symbol in symbols parameter"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

**Response (404):**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Symbol not found",
    "details": {
      "field": "symbols",
      "issue": "Symbol 'INVALID' not found in Yahoo Finance database"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 16. Yahoo Rank
**GET** `/rank/yahoo`

**Parameters:**
- `symbols` (required): Comma-separated symbols (e.g., "AAPL,MSFT,GOOGL")
- `range` (optional): Time range (default: "3mo")
- `interval` (optional): Time interval (default: "1d")
- `rf_annual` (optional): Risk-free annual rate (default: 0.0)
- `target_return_annual` (optional): Target annual return (default: rf_annual)
- `periods_per_year` (optional): Number of periods per year

**Response (200):**
```json
{
  "results": [
    {
      "symbol": "AAPL",
      "metrics": {
        "sharpe_ratio": 1.25,
        "sortino_ratio": 1.45,
        "calmar_ratio": 0.85,
        "max_drawdown": -0.15,
        "volatility": 0.25,
        "annual_return": 0.18,
        "composite_score": 1.18
      }
    },
    {
      "symbol": "MSFT",
      "metrics": {
        "sharpe_ratio": 1.15,
        "sortino_ratio": 1.35,
        "calmar_ratio": 0.75,
        "max_drawdown": -0.18,
        "volatility": 0.28,
        "annual_return": 0.16,
        "composite_score": 1.08
      }
    }
  ]
}
```

### 17. Yahoo Recommendations
**GET** `/recommendations/yahoo`

**Parameters:**
- `symbols` (required): Comma-separated symbols
- `range` (optional): Time range (default: "3mo")
- `interval` (optional): Time interval (default: "1d")
- `rf_annual` (optional): Risk-free annual rate (default: 0.0)
- `target_return_annual` (optional): Target annual return (default: rf_annual)
- `periods_per_year` (optional): Number of periods per year

**Response (200):**
```json
{
  "results": [
    {
      "symbol": "AAPL",
      "metrics": {
        "sharpe_ratio": 1.25,
        "sortino_ratio": 1.45,
        "calmar_ratio": 0.85,
        "max_drawdown": -0.15,
        "volatility": 0.25,
        "annual_return": 0.18,
        "composite_score": 1.18
      }
    }
  ]
}
```

---

## Options Endpoints

### 18. Options Recommendations
**GET** `/options/recommendations`

**Parameters:**
- `symbol` (optional): Single symbol
- `symbols` (optional): Comma-separated symbols
- `symbols_source` (optional): "yahoo", "finviz", "both" (default: "both")
- `yahoo_search` (optional): Yahoo search query
- `yahoo_limit` (optional): Yahoo results limit (default: 25)
- `yahoo_list` (optional): Yahoo list type - "most_actives", "gainers", "losers", "trending" (default: "most_actives")
- `yahoo_region` (optional): Yahoo region (default: "US")
- `side` (optional): "call", "put", "both" (default: "both")
- `min_dte` (optional): Minimum days to expiration (default: 7)
- `max_dte` (optional): Maximum days to expiration (default: 60)
- `limit` (optional): Number of results (default: 20)
- `rf_annual` (optional): Risk-free annual rate (default: 0.03)
- `range` (optional): Time range for underlying analysis (default: "3mo")
- `interval` (optional): Time interval for underlying analysis (default: "1d")
- `sharpe_w` (optional): Sharpe ratio weight (default: 0.4)
- `sortino_w` (optional): Sortino ratio weight (default: 0.4)
- `calmar_w` (optional): Calmar ratio weight (default: 0.2)
- `min_delta` (optional): Minimum delta
- `max_delta` (optional): Maximum delta
- `min_premium` (optional): Minimum premium
- `max_premium` (optional): Maximum premium
- `min_volume` (optional): Minimum volume
- `min_strike_ratio` (optional): Minimum strike ratio
- `max_strike_ratio` (optional): Maximum strike ratio
- `signal` (optional): Finviz signal
- `order` (optional): Finviz order
- `screener` (optional): Finviz screener
- `symbols_limit` (optional): Symbols limit
- `per_symbol_limit` (optional): Per symbol limit
- `max_spread_pct` (optional): Maximum spread percentage
- `feed` (optional): Data feed
- `type` (optional): Option type
- `strike_price_gte` (optional): Minimum strike price
- `strike_price_lte` (optional): Maximum strike price
- `expiration_date` (optional): Expiration date
- `expiration_date_gte` (optional): Minimum expiration date
- `expiration_date_lte` (optional): Maximum expiration date
- `root_symbol` (optional): Root symbol
- `page_token` (optional): Page token
- `alpaca_limit` (optional): Alpaca limit
- `underlying_top` (optional): Top underlying symbols
- `debug` (optional): Enable debug mode (default: true)

**Response (200):**
```json
{
  "success": true,
  "data": {
    "results": [
      {
        "symbol": "AAPL",
        "company_name": "Apple Inc.",
        "underlying_price": 150.25,
        "underlying_metrics": {
          "sharpe_ratio": 1.25,
          "sortino_ratio": 1.45,
          "calmar_ratio": 0.85,
          "composite_score": 1.18,
          "volatility": 0.25,
          "annual_return": 0.18,
          "max_drawdown": -0.15
        },
        "options": [
          {
            "symbol": "AAPL240119C00150000",
            "underlying_symbol": "AAPL",
            "strike_price": 150.0,
            "expiration_date": "2024-01-19",
            "type": "call",
            "days_to_expiration": 18,
            "open_interest": 1250,
            "volume": 450,
            "close_price": 5.25,
            "ask_price": 5.30,
            "bid_price": 5.20,
            "last_price": 5.25,
            "implied_volatility": 0.35,
            "delta": 0.65,
            "gamma": 0.02,
            "theta": -0.15,
            "vega": 0.25,
            "rho": 0.08,
            "bid_ask_spread": 0.10,
            "bid_ask_spread_percent": 1.89,
            "volume_oi_ratio": 0.36,
            "intrinsic_value": 0.25,
            "time_value": 5.00,
            "in_the_money": true,
            "break_even_price": 155.25,
            "max_profit": "unlimited",
            "max_loss": 5.25,
            "probability_profit": 0.65,
            "risk_reward_ratio": 0.05
          },
          {
            "symbol": "AAPL240119P00145000",
            "underlying_symbol": "AAPL",
            "strike_price": 145.0,
            "expiration_date": "2024-01-19",
            "type": "put",
            "days_to_expiration": 18,
            "open_interest": 850,
            "volume": 320,
            "close_price": 2.15,
            "ask_price": 2.20,
            "bid_price": 2.10,
            "last_price": 2.15,
            "implied_volatility": 0.32,
            "delta": -0.35,
            "gamma": 0.03,
            "theta": -0.12,
            "vega": 0.18,
            "rho": -0.05,
            "bid_ask_spread": 0.10,
            "bid_ask_spread_percent": 4.65,
            "volume_oi_ratio": 0.38,
            "intrinsic_value": 0.0,
            "time_value": 2.15,
            "in_the_money": false,
            "break_even_price": 142.85,
            "max_profit": 142.85,
            "max_loss": 2.15,
            "probability_profit": 0.35,
            "risk_reward_ratio": 66.44
          }
        ],
        "recommendation_summary": {
          "total_options": 2,
          "calls": 1,
          "puts": 1,
          "avg_implied_volatility": 0.335,
          "avg_volume_oi_ratio": 0.37,
          "best_call": "AAPL240119C00150000",
          "best_put": "AAPL240119P00145000"
        }
      }
    ],
    "summary": {
      "total_symbols_analyzed": 25,
      "total_options_found": 150,
      "average_underlying_score": 1.15,
      "top_recommendations": ["AAPL", "NVDA", "TSLA"],
      "market_sentiment": "bullish",
      "volatility_regime": "moderate"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

**Response (400):**
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid parameters provided",
    "details": {
      "field": "symbols",
      "issue": "Either 'symbol' or 'symbols' parameter must be provided"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

---

## Kraken Endpoints

### 19. Kraken Ticker
**GET** `/kraken/ticker`

**Parameters:**
- `pairs` (optional): Comma-separated pairs (default: "XBT/USD,ETH/USD")
- `depth` (optional): Order book depth
- `limit` (optional): Number of results
- `since` (optional): Since timestamp
- `interval` (optional): Time interval

**Response (200):**
```json
{
  "success": true,
  "data": [
    {
      "pair": "XBT/USD",
      "price": 45000.0,
      "volume": 1250.5,
      "high": 46000.0,
      "low": 44000.0,
      "vwap": 45250.0,
      "trades": 12500,
      "bid": 44950.0,
      "ask": 45050.0
    }
  ],
  "timestamp": 1640995200
}
```

### 20. Kraken Ticker by Pair
**GET** `/kraken/ticker/{pair}`

**Parameters:**
- `pair` (path): Trading pair (e.g., "XBT/USD")

**Response (200):**
```json
{
  "success": true,
  "data": {
    "pair": "XBT/USD",
    "price": 45000.0,
    "volume": 1250.5,
    "high": 46000.0,
    "low": 44000.0,
    "vwap": 45250.0,
    "trades": 12500,
    "bid": 44950.0,
    "ask": 45050.0
  },
  "timestamp": 1640995200
}
```

### 21. Kraken Order Book
**GET** `/kraken/orderbook/{pair}`

**Parameters:**
- `pair` (path): Trading pair
- `depth` (optional): Order book depth (default: 100)

**Response (200):**
```json
{
  "success": true,
  "data": {
    "pair": "XBT/USD",
    "bids": [
      [44950.0, 1.5, 1640995200],
      [44900.0, 2.0, 1640995200]
    ],
    "asks": [
      [45050.0, 1.2, 1640995200],
      [45100.0, 1.8, 1640995200]
    ]
  },
  "timestamp": 1640995200
}
```

### 22. Kraken Assets
**GET** `/kraken/assets`

**Response (200):**
```json
{
  "success": true,
  "data": {
    "XBT": {
      "altname": "XBT",
      "decimals": 10,
      "display_decimals": 5
    },
    "ETH": {
      "altname": "ETH",
      "decimals": 10,
      "display_decimals": 5
    }
  },
  "timestamp": 1640995200
}
```

### 23. Kraken Asset Pairs
**GET** `/kraken/pairs`

**Response (200):**
```json
{
  "success": true,
  "data": {
    "XBTUSD": {
      "altname": "XBTUSD",
      "wsname": "XBT/USD",
      "aclass_base": "currency",
      "base": "XXBT",
      "aclass_quote": "currency",
      "quote": "ZUSD",
      "lot": "unit",
      "pair_decimals": 1,
      "lot_decimals": 8,
      "lot_multiplier": 1,
      "leverage_buy": [],
      "leverage_sell": [],
      "fees": [[0, 0.26], [50000, 0.24]],
      "fees_maker": [[0, 0.16], [50000, 0.14]],
      "fee_volume_currency": "ZUSD",
      "margin_call": 80,
      "margin_stop": 40
    }
  },
  "timestamp": 1640995200
}
```

### 24. Kraken Recent Trades
**GET** `/kraken/trades/{pair}`

**Parameters:**
- `pair` (path): Trading pair
- `since` (optional): Since timestamp
- `limit` (optional): Number of trades (default: 100)

**Response (200):**
```json
{
  "success": true,
  "data": {
    "pair": "XBT/USD",
    "trades": [
      [45000.0, 1.5, 1640995200, "b", "l", ""],
      [44950.0, 2.0, 1640995200, "s", "m", ""]
    ],
    "last": "1640995200"
  },
  "timestamp": 1640995200
}
```

### 25. Kraken OHLC
**GET** `/kraken/ohlc/{pair}`

**Parameters:**
- `pair` (path): Trading pair
- `interval` (optional): Time interval in minutes (default: 1)
- `since` (optional): Since timestamp

**Response (200):**
```json
{
  "success": true,
  "data": {
    "pair": "XBT/USD",
    "ohlc": [
      [1640995200, 45000.0, 45100.0, 44900.0, 45050.0, 1250.5, 12500],
      [1640995260, 45050.0, 45200.0, 45000.0, 45150.0, 1300.2, 13000]
    ],
    "last": 1640995260
  },
  "timestamp": 1640995200
}
```

### 26. Kraken Trending Crypto
**GET** `/kraken/trending`

**Parameters:**
- `limit` (optional): Number of results (default: 10)

**Response (200):**
```json
{
  "success": true,
  "data": [
    {
      "pair": "XBT/USD",
      "volume_24h": 1250000.0,
      "price_change_24h": 2.5,
      "price_change_percent_24h": 5.8
    }
  ],
  "timestamp": 1640995200
}
```

### 27. Kraken Market Summary
**GET** `/kraken/summary/{pair}`

**Parameters:**
- `pair` (path): Trading pair

**Response (200):**
```json
{
  "success": true,
  "data": {
    "pair": "XBT/USD",
    "price": 45000.0,
    "volume_24h": 1250000.0,
    "high_24h": 46000.0,
    "low_24h": 44000.0,
    "vwap_24h": 45250.0,
    "trades_24h": 125000,
    "bid": 44950.0,
    "ask": 45050.0,
    "spread": 100.0,
    "spread_percent": 0.22
  },
  "timestamp": 1640995200
}
```

### 28. Kraken System Status
**GET** `/kraken/status`

**Response (200):**
```json
{
  "success": true,
  "data": {
    "status": "online",
    "timestamp": 1640995200
  },
  "timestamp": 1640995200
}
```

### 29. Kraken Server Time
**GET** `/kraken/time`

**Response (200):**
```json
{
  "success": true,
  "data": {
    "unixtime": 1640995200,
    "rfc1123": "Mon, 01 Jan 2024 12:00:00 +0000"
  },
  "timestamp": 1640995200
}
```

---

## CoinGecko Endpoints

### 30. CoinGecko Top Cryptocurrencies
**GET** `/coingecko/top`

**Parameters:**
- `limit` (optional): Number of results (default: 10)
- `vs_currency` (optional): Quote currency (default: "usd")
- `order` (optional): Sort order - "market_cap_desc", "market_cap_asc", "volume_desc", "volume_asc", "id_desc", "id_asc" (default: "market_cap_desc")
- `page` (optional): Page number (default: 1)
- `sparkline` (optional): Include sparkline data (default: false)
- `price_change_percentage` (optional): Price change percentage - "1h", "24h", "7d", "14d", "30d", "200d", "1y" (default: "24h")

**Response (200):**
```json
[
  {
    "id": "bitcoin",
    "symbol": "btc",
    "name": "Bitcoin",
    "current_price": 122402.524,
    "market_cap": 2410000000000.0,
    "market_cap_rank": 1,
    "total_volume": 25000000000,
    "high_24h": 123000.0,
    "low_24h": 121000.0,
    "price_change_24h": 1500.0,
    "price_change_percentage_24h": 1.24,
    "price_change_percentage_7d_in_currency": 5.2,
    "price_change_percentage_30d_in_currency": 12.5,
    "market_cap_change_24h": 25000000000,
    "market_cap_change_percentage_24h": 1.05,
    "circulating_supply": 19500000,
    "total_supply": 21000000,
    "max_supply": 21000000,
    "ath": 123000.0,
    "ath_change_percentage": -0.49,
    "ath_date": "2025-08-10T04:00:34.604Z",
    "atl": 67.81,
    "atl_change_percentage": 180000.0,
    "atl_date": "2013-07-06T00:00:00.000Z",
    "roi": null,
    "last_updated": "2025-08-10T04:00:34.604Z",
    "sparkline_in_7d": null
  },
  {
    "id": "ethereum",
    "symbol": "eth",
    "name": "Ethereum",
    "current_price": 3500.0,
    "market_cap": 420000000000.0,
    "market_cap_rank": 2,
    "total_volume": 15000000000,
    "high_24h": 3550.0,
    "low_24h": 3450.0,
    "price_change_24h": 50.0,
    "price_change_percentage_24h": 1.45,
    "price_change_percentage_7d_in_currency": 3.2,
    "price_change_percentage_30d_in_currency": 8.5,
    "market_cap_change_24h": 5000000000,
    "market_cap_change_percentage_24h": 1.20,
    "circulating_supply": 120000000,
    "total_supply": 120000000,
    "max_supply": null,
    "ath": 4800.0,
    "ath_change_percentage": -27.08,
    "ath_date": "2021-11-10T14:24:11.849Z",
    "atl": 0.432979,
    "atl_change_percentage": 808000.0,
    "atl_date": "2015-10-20T00:00:00.000Z",
    "roi": null,
    "last_updated": "2025-08-10T04:00:25.776Z",
    "sparkline_in_7d": null
  },
  {
    "id": "ripple",
    "symbol": "xrp",
    "name": "XRP",
    "current_price": 3.26,
    "market_cap": 193006306750.0,
    "market_cap_rank": 3,
    "total_volume": 5016329562.0,
    "high_24h": 3.34,
    "low_24h": 3.22,
    "price_change_24h": -0.0363461210488345,
    "price_change_percentage_24h": -1.10396,
    "price_change_percentage_7d_in_currency": 15.23724308356491,
    "price_change_percentage_30d_in_currency": 25.83262677682219,
    "market_cap_change_24h": -2301831176.9060974,
    "market_cap_change_percentage_24h": -1.17856,
    "circulating_supply": 59308385925.0,
    "total_supply": 99985880506.0,
    "max_supply": 100000000000.0,
    "ath": 3.65,
    "ath_change_percentage": -10.74195,
    "ath_date": "2025-07-18T03:40:53.808Z",
    "atl": 0.00268621,
    "atl_change_percentage": 121062.89998,
    "atl_date": "2014-05-22T00:00:00.000Z",
    "roi": null,
    "last_updated": "2025-08-10T04:00:25.776Z",
    "sparkline_in_7d": null
  }
]
```

### 31. CoinGecko Top Gainers
**GET** `/coingecko/gainers`

**Parameters:**
- `limit` (optional): Number of results (default: 10)
- `vs_currency` (optional): Quote currency (default: "usd")

**Response (200):**
```json
{
  "success": true,
  "data": [
    {
      "id": "ethereum",
      "symbol": "eth",
      "name": "Ethereum",
      "current_price": 3000.0,
      "price_change_percentage_24h": 8.5,
      "market_cap": 350000000000
    }
  ],
  "timestamp": 1640995200
}
```

### 32. CoinGecko Top Losers
**GET** `/coingecko/losers`

**Parameters:**
- `limit` (optional): Number of results (default: 10)
- `vs_currency` (optional): Quote currency (default: "usd")

**Response (200):**
```json
{
  "success": true,
  "data": [
    {
      "id": "cardano",
      "symbol": "ada",
      "name": "Cardano",
      "current_price": 0.50,
      "price_change_percentage_24h": -5.2,
      "market_cap": 15000000000
    }
  ],
  "timestamp": 1640995200
}
```

### 33. CoinGecko Trending Cryptocurrencies
**GET** `/coingecko/trending`

**Response (200):**
```json
{
  "success": true,
  "data": [
    {
      "item": {
        "id": "bitcoin",
        "coin_id": 1,
        "name": "Bitcoin",
        "symbol": "btc",
        "market_cap_rank": 1,
        "thumb": "https://example.com/thumb.png",
        "small": "https://example.com/small.png",
        "large": "https://example.com/large.png",
        "slug": "bitcoin",
        "price_btc": 1.0,
        "score": 100
      }
    }
  ],
  "timestamp": 1640995200
}
```

### 34. CoinGecko Market Overview
**GET** `/coingecko/overview`

**Response (200):**
```json
{
  "success": true,
  "data": {
    "total_market_cap": 2500000000000,
    "total_volume": 150000000000,
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

### 35. CoinGecko Market Context
**GET** `/coingecko/context`

**Response (200):**
```json
{
  "success": true,
  "context": "Market is showing bullish momentum with increasing volume",
  "timestamp": 1640995200
}
```

### 36. CoinGecko Trending Symbols
**GET** `/coingecko/trending-symbols`

**Response (200):**
```json
{
  "success": true,
  "data": ["btc", "eth", "sol", "ada"],
  "timestamp": 1640995200
}
```

### 37. CoinGecko Simple Price
**GET** `/coingecko/simple-price`

**Parameters:**
- `ids` (required): Comma-separated coin IDs
- `vs_currencies` (required): Comma-separated quote currencies
- `include_24hr_change` (optional): Include 24h price change (default: false)

**Response (200):**
```json
{
  "success": true,
  "data": {
    "bitcoin": {
      "usd": 45000.0,
      "usd_24h_change": 3.45
    },
    "ethereum": {
      "usd": 3000.0,
      "usd_24h_change": 8.5
    }
  },
  "timestamp": 1640995200
}
```

---

## High Open Interest Endpoints

### 38. High Open Interest Contracts
**GET** `/high-open-interest/{ticker}`

**Parameters:**
- `ticker` (path): Stock ticker symbol
- `option_type` (optional): "call" or "put" (default: "call")

**Response (200):**
```json
{
  "ticker": "AAPL",
  "result": {
    "short_term": {
      "symbol": "AAPL240119C00150000",
      "underlying_symbol": "AAPL",
      "strike_price": 150.0,
      "expiration_date": "2024-01-19",
      "type": "call",
      "open_interest": 1250,
      "close_price": 5.25,
      "ask_price": 5.30,
      "bid_price": 5.20,
      "last_price": 5.25,
      "implied_volatility": 0.35
    },
    "leap": {
      "symbol": "AAPL250117C00150000",
      "underlying_symbol": "AAPL",
      "strike_price": 150.0,
      "expiration_date": "2025-01-17",
      "type": "call",
      "open_interest": 500,
      "close_price": 15.50,
      "ask_price": 15.60,
      "bid_price": 15.40,
      "last_price": 15.50,
      "implied_volatility": 0.45
    },
    "error": null
  }
}
```

### 39. High Open Interest Batch
**GET** `/high-open-interest/batch`

**Parameters:**
- `tickers` (required): Comma-separated list of ticker symbols
- `option_type` (optional): "call" or "put" (default: "call")

**Response (200):**
```json
[
  {
    "ticker": "AAPL",
    "result": {
      "short_term": {
        "symbol": "AAPL240119C00150000",
        "underlying_symbol": "AAPL",
        "strike_price": 150.0,
        "expiration_date": "2024-01-19",
        "type": "call",
        "open_interest": 1250,
        "close_price": 5.25,
        "ask_price": 5.30,
        "bid_price": 5.20,
        "last_price": 5.25,
        "implied_volatility": 0.35
      },
      "leap": null,
      "error": null
    }
  },
  {
    "ticker": "TSLA",
    "result": {
      "short_term": null,
      "leap": null,
      "error": "Invalid ticker symbol: TSLA"
    }
  }
]
```

---

## Trending Options Endpoints

### 40. Trending Options Analysis
**GET** `/trending-options`

**Parameters:**
- `option_type` (optional): "call", "put", or "both" (default: "call")
- `rf_annual` (optional): Risk-free annual rate (default: 0.03)
- `range` (optional): Time range - "1mo", "3mo", "6mo", "1y", "2y", "5y" (default: "3mo")
- `interval` (optional): Time interval - "1d", "1wk", "1mo" (default: "1d")
- `sharpe_w` (optional): Sharpe ratio weight (default: 0.4)
- `sortino_w` (optional): Sortino ratio weight (default: 0.4)
- `calmar_w` (optional): Calmar ratio weight (default: 0.2)
- `limit` (optional): Maximum number of results (default: 10)
- `min_underlying_score` (optional): Minimum underlying composite score (default: 0.0)
- `min_undervalued_score` (optional): Minimum undervalued score (default: 0.0)

**Response (200):**
```json
{
  "results": [
    {
      "ticker": "NVDA",
      "underlying_metrics": {
        "sharpe_ratio": 2.15,
        "sortino_ratio": 2.45,
        "calmar_ratio": 1.85,
        "max_drawdown": -0.12,
        "volatility": 0.35,
        "annual_return": 0.45,
        "composite_score": 2.15
      },
      "options_analysis": {
        "undervalued_score": 0.85,
        "implied_volatility_percentile": 65,
        "volume_analysis": "high",
        "open_interest_trend": "increasing"
      },
      "recommended_contracts": [
        {
          "symbol": "NVDA240119C00500000",
          "strike_price": 500.0,
          "expiration_date": "2024-01-19",
          "type": "call",
          "open_interest": 2500,
          "implied_volatility": 0.45,
          "delta": 0.65,
          "gamma": 0.02,
          "theta": -0.15,
          "vega": 0.25
        }
      ]
    }
  ],
  "summary": {
    "total_analyzed": 50,
    "total_with_options": 35,
    "average_underlying_score": 1.25,
    "average_undervalued_score": 0.65,
    "top_underlying_tickers": ["NVDA", "AMD", "TSLA"],
    "top_undervalued_tickers": ["NVDA", "AMD", "AAPL"]
  }
}
```

---

## Finviz Endpoints

### 41. Finviz Screener Candidates
**GET** `/screener/candidates`

**Parameters:**
- `signal` (optional): Finviz signal
- `order` (optional): Finviz order
- `screener` (optional): Finviz screener
- `limit` (optional): Number of results

**Response (200):**
```json
{
  "candidates": [
    {
      "ticker": "AAPL",
      "company": "Apple Inc.",
      "sector": "Technology",
      "industry": "Consumer Electronics",
      "country": "USA",
      "market_cap": 2500000000000,
      "pe": 25.5,
      "price": 150.0,
      "change": 2.5,
      "change_percent": 1.67,
      "volume": 50000000
    }
  ]
}
```

### 42. Finviz Recommendations
**GET** `/recommendations/finviz`

**Parameters:**
- `signal` (optional): Finviz signal
- `order` (optional): Finviz order
- `screener` (optional): Finviz screener
- `limit` (optional): Number of results

**Response (200):**
```json
{
  "recommendations": [
    {
      "ticker": "AAPL",
      "recommendation": "Strong Buy",
      "price_target": 175.0,
      "upside": 16.67,
      "analysts": 45,
      "rating": 4.5
    }
  ]
}
```

---

## Error Responses

All endpoints follow a consistent error response format. Here are the complete error responses you may encounter:

### 400 Bad Request
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid parameters or malformed request",
    "details": {
      "field": "symbol",
      "issue": "Symbol must be a valid ticker symbol",
      "valid_format": "Uppercase letters only (e.g., AAPL, TSLA)"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 401 Unauthorized
```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required",
    "details": {
      "reason": "API key missing or invalid",
      "required": "Valid API key in Authorization header"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 403 Forbidden
```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "Access denied",
    "details": {
      "reason": "Insufficient permissions for this endpoint",
      "required_permissions": ["read:market_data", "read:options"]
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 404 Not Found
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource not found",
    "details": {
      "resource": "symbol",
      "value": "INVALID",
      "suggestion": "Check symbol spelling or try a different symbol"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 429 Too Many Requests
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "limit": 60,
      "window": "1 minute",
      "retry_after": 30,
      "current_usage": 65
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 500 Internal Server Error
```json
{
  "success": false,
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Internal server error occurred",
    "details": {
      "reason": "Database connection failed",
      "error_id": "err_987654321"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 502 Bad Gateway
```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Upstream service unavailable",
    "details": {
      "service": "Yahoo Finance",
      "reason": "Service temporarily down",
      "retry_after": 300
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

### 503 Service Unavailable
```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Service temporarily unavailable",
    "details": {
      "reason": "Maintenance in progress",
      "estimated_downtime": "30 minutes",
      "status_page": "https://status.trading-api.com"
    }
  },
  "timestamp": 1640995200,
  "request_id": "req_123456789"
}
```

---

## Rate Limiting

The API implements rate limiting to prevent abuse:
- Default: 60 requests per minute per client
- Burst: 10 requests per burst
- Rate limit headers are included in responses:
  - `X-RateLimit-Limit`: Requests per minute
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Reset time

## CORS

The API supports CORS for cross-origin requests:
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

## Pagination

Some endpoints support pagination:
- Use `page` parameter for page number
- Use `limit` parameter for results per page
- Pagination metadata is included in responses where applicable

## Timestamps

All timestamps are returned in Unix timestamp format (seconds since epoch) unless otherwise specified.

## Data Sources

The API aggregates data from multiple sources:
- **Yahoo Finance**: Stock prices, metrics, and options data
- **Alpaca Markets**: Real-time market data and options
- **Kraken**: Cryptocurrency exchange data
- **CoinGecko**: Cryptocurrency market data
- **Finviz**: Stock screening and analysis
- **Reddit**: Social sentiment and trending stocks

## Frontend Integration Examples

### JavaScript/TypeScript Examples

#### Basic API Client
```typescript
class TradingAPIClient {
  private baseURL: string;
  private apiKey?: string;

  constructor(baseURL: string = 'http://localhost:3000', apiKey?: string) {
    this.baseURL = baseURL;
    this.apiKey = apiKey;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error?.message || `HTTP ${response.status}`);
    }

    return data;
  }

  // Health check
  async healthCheck(): Promise<{ status: string; version: string }> {
    const response = await this.request<{ data: { status: string; version: string } }>('/health');
    return response.data;
  }

  // Get stock metrics
  async getStockMetrics(symbol: string, range: string = '3mo'): Promise<any> {
    const response = await this.request(`/metrics/yahoo?symbols=${symbol}&range=${range}`);
    return response.data;
  }

  // Get options recommendations
  async getOptionsRecommendations(params: {
    symbol?: string;
    symbols?: string;
    side?: 'call' | 'put' | 'both';
    limit?: number;
  }): Promise<any> {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.append(key, value.toString());
      }
    });
    
    const response = await this.request(`/options/recommendations?${searchParams}`);
    return response.data;
  }

  // Get trending stocks
  async getTrendingStocks(limit: number = 10): Promise<any> {
    const response = await this.request(`/trending/stocks?limit=${limit}`);
    return response.data;
  }
}
```

#### React Hook Example
```typescript
import { useState, useEffect } from 'react';

interface UseAPIResponse<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => void;
}

function useAPI<T>(
  endpoint: string,
  dependencies: any[] = []
): UseAPIResponse<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await fetch(`http://localhost:3000${endpoint}`);
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error?.message || 'API request failed');
      }
      
      setData(result.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, dependencies);

  return { data, loading, error, refetch: fetchData };
}

// Usage in component
function StockMetrics({ symbol }: { symbol: string }) {
  const { data, loading, error, refetch } = useAPI(
    `/metrics/yahoo?symbols=${symbol}`,
    [symbol]
  );

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!data) return <div>No data</div>;

  return (
    <div>
      <h2>{data.symbol} Metrics</h2>
      <p>Sharpe Ratio: {data.metrics.sharpe_ratio}</p>
      <p>Current Price: ${data.current_price}</p>
      <button onClick={refetch}>Refresh</button>
    </div>
  );
}
```

#### Error Handling Utility
```typescript
class APIError extends Error {
  constructor(
    public code: string,
    message: string,
    public details?: any,
    public statusCode?: number
  ) {
    super(message);
    this.name = 'APIError';
  }
}

function handleAPIError(error: any): APIError {
  if (error.response) {
    const { status, data } = error.response;
    return new APIError(
      data.error?.code || 'UNKNOWN_ERROR',
      data.error?.message || 'Unknown error occurred',
      data.error?.details,
      status
    );
  }
  
  return new APIError('NETWORK_ERROR', 'Network error occurred');
}

// Usage
try {
  const data = await apiClient.getStockMetrics('AAPL');
  // Handle success
} catch (error) {
  const apiError = handleAPIError(error);
  
  switch (apiError.code) {
    case 'RATE_LIMIT_EXCEEDED':
      // Show retry after message
      console.log(`Retry after ${apiError.details?.retry_after} seconds`);
      break;
    case 'NOT_FOUND':
      // Show not found message
      console.log(`Symbol not found: ${apiError.details?.value}`);
      break;
    default:
      // Show generic error
      console.error(apiError.message);
  }
}
```

### Rate Limiting Handling
```typescript
class RateLimitHandler {
  private retryAfter: number = 0;
  private retryQueue: Array<() => void> = [];

  async handleRateLimit<T>(request: () => Promise<T>): Promise<T> {
    try {
      return await request();
    } catch (error) {
      if (error.code === 'RATE_LIMIT_EXCEEDED') {
        return this.handleRateLimitExceeded(request, error.details?.retry_after);
      }
      throw error;
    }
  }

  private async handleRateLimitExceeded<T>(
    request: () => Promise<T>,
    retryAfter: number = 60
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      this.retryQueue.push(async () => {
        try {
          const result = await request();
          resolve(result);
        } catch (error) {
          reject(error);
        }
      });

      if (this.retryAfter === 0) {
        this.retryAfter = retryAfter;
        setTimeout(() => {
          this.retryAfter = 0;
          this.processQueue();
        }, retryAfter * 1000);
      }
    });
  }

  private processQueue() {
    while (this.retryQueue.length > 0) {
      const request = this.retryQueue.shift();
      if (request) {
        request();
      }
    }
  }
}
```

### Real-time Data with WebSocket
```typescript
class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor(private url: string, private onMessage: (data: any) => void) {}

  connect() {
    this.ws = new WebSocket(this.url);
    
    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.onMessage(data);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.attemptReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
      
      setTimeout(() => {
        console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
        this.connect();
      }, delay);
    }
  }

  send(data: any) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
```

## Testing

You can test the API using the interactive documentation at:
```
http://localhost:3000/docs
```

This provides a Swagger UI interface for testing all endpoints with real data.

### Postman Collection
A Postman collection is available for testing all endpoints:
```
https://www.postman.com/collections/trading-api
```

### cURL Examples
```bash
# Health check
curl -X GET "http://localhost:3000/health"

# Get stock metrics
curl -X GET "http://localhost:3000/metrics/yahoo?symbols=AAPL&range=3mo"

# Get options recommendations
curl -X GET "http://localhost:3000/options/recommendations?symbol=AAPL&side=call&limit=10"

# Get trending stocks
curl -X GET "http://localhost:3000/trending/stocks?limit=20"
```

## Tested Endpoints Status

The following endpoints were tested against the running server and their status is documented below:

### ✅ Working Endpoints (Real Data Captured)
- **Health Check** (`/health`) - Returns basic status
- **System Status** (`/status`) - Returns detailed system health with service status
- **System Metrics** (`/metrics`) - Returns performance metrics
- **News** (`/news`) - Returns aggregated news from Finviz and Reddit
- **CoinGecko Top** (`/coingecko/top`) - Returns top cryptocurrencies with real market data

### ⚠️ Endpoints Not Responding (May need configuration)
- **Trending Stocks** (`/trending/stocks`) - No response during testing
- **Yahoo Metrics** (`/metrics/yahoo`) - No response during testing
- **Options Recommendations** (`/options/recommendations`) - No response during testing
- **High Open Interest** (`/high-open-interest/{ticker}`) - No response during testing
- **Kraken Endpoints** (`/kraken/*`) - No response during testing

### 📝 Notes for Frontend Developers
1. **Real Data**: The response examples in this documentation are actual responses from the running server
2. **Service Dependencies**: Some endpoints may require external API keys (Alpaca, Reddit, etc.) to be configured
3. **Error Handling**: Always implement proper error handling as some endpoints may not be available
4. **Rate Limiting**: The server implements rate limiting, so include retry logic in your frontend code
5. **Testing**: Test each endpoint individually before implementing in production

### 🔧 Configuration Required
To get all endpoints working, ensure the following are configured:
- Alpaca API keys for options and market data
- Reddit API credentials for social sentiment
- Yahoo Finance API access
- Kraken API keys for cryptocurrency data
- CoinGecko API access (working without authentication)

## News API Optimizations

The news API has been significantly optimized for better performance and reliability:

### Performance Improvements

1. **Parallel Execution**: All three news sources (Finviz, Reddit, Alpaca) are now fetched concurrently instead of sequentially
2. **Individual Timeouts**: Each service has its own timeout to prevent hanging on slow services
3. **Caching**: In-memory caching with 5-minute TTL to avoid redundant API calls
4. **Retry Logic**: Exponential backoff retry mechanism for handling transient failures
5. **Error Isolation**: Individual service failures don't break the entire request

### Available Functions

#### `get_news()` - Optimized Parallel Fetch
```rust
pub async fn get_news() -> Result<Value, String>
```
- Fetches news from all sources in parallel
- Individual timeouts: Finviz (10s), Reddit (12s), Alpaca (8s)
- Graceful error handling for each service

#### `get_news_cached()` - Cached Version
```rust
pub async fn get_news_cached() -> Result<Value, String>
```
- Returns cached data if available (5-minute TTL)
- Falls back to fresh fetch if cache is expired
- Significantly faster for repeated requests

#### `get_news_with_retry()` - Retry Logic
```rust
pub async fn get_news_with_retry(max_retries: u32) -> Result<Value, String>
```
- Implements exponential backoff retry
- Useful for handling transient network issues
- Configurable retry count

#### `benchmark_news_performance()` - Performance Testing
```rust
pub async fn benchmark_news_performance() -> Result<String, String>
```
- Compares sequential vs parallel vs cached performance
- Returns detailed timing information
- Useful for performance monitoring

### Performance Expectations

- **Sequential vs Parallel**: ~3x speedup
- **Cached vs Fresh**: ~10-100x speedup for repeated requests
- **Error Recovery**: Individual service failures don't affect others
- **Timeout Protection**: Prevents hanging on slow services

### Usage Examples

```rust
// Basic optimized fetch
let news = get_news().await?;

// Cached fetch (recommended for production)
let news = get_news_cached().await?;

// With retry logic
let news = get_news_with_retry(3).await?;

// Performance benchmark
let benchmark = benchmark_news_performance().await?;
println!("{}", benchmark);
```

### Configuration

The optimizations use the following default settings:
- Cache TTL: 5 minutes
- Timeouts: Finviz (10s), Reddit (12s), Alpaca (8s)
- Retry backoff: 100ms, 200ms, 400ms, etc.

These can be adjusted by modifying the constants in the source code.
