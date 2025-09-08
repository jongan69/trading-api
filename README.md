# Trading API MCP Server

A comprehensive **Model Context Protocol (MCP) server** that exposes powerful market research tools to AI agents. Built on top of a robust Rust trading API that aggregates data from multiple financial sources including traditional markets, cryptocurrencies, DeFi protocols, Solana ecosystem, and meme tokens.

## 🌟 Features

- **40+ Market Research Tools** for AI agents via MCP
- **Multi-Source Data Aggregation**: Stocks, crypto, DeFi, NFTs, meme tokens, news, sentiment, options
- **Real-Time Market Data** from 15+ sources including Reddit, Finviz, CoinGecko, Kraken, Yahoo Finance, Hyperliquid, Pump.fun, Helius
- **Comprehensive Analysis** combining traditional and DeFi markets
- **Solana Ecosystem Integration** with NFT and token analytics
- **DeFi Protocol Support** including Hyperliquid derivatives
- **Meme Token Analysis** via Pump.fun integration
- **Production-Ready** with error handling, caching, rate limiting, and comprehensive documentation
- **Trading Capabilities** for Kraken and Alpaca (paper trading)

## 🚀 Quick Start

```bash
# 1. Setup (one time)
./setup_mcp.sh

# 2. Start services
cargo run                    # Terminal 1: Rust API
source venv/bin/activate     # Terminal 2: Python env
python mcp_server.py         # Terminal 2: MCP server

# 3. Test everything works
python test_mcp.py
```

## 🛠️ MCP Tools Available

### Core Market Analysis Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_comprehensive_market_analysis` | Complete market overview across all sources | Overall market research |
| `get_trending_stocks` | Hot stocks from Reddit, Finviz, and other sources | Trend identification |
| `get_trending_crypto` | Trending cryptocurrencies from multiple sources | Crypto market analysis |
| `get_market_news` | Aggregated financial news from multiple sources | Sentiment analysis |
| `get_reddit_trending_stocks` | Reddit sentiment analysis with social indicators | Social media trends |
| `search_market_data` | Search across all data sources | Targeted research |

### Traditional Finance Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_yahoo_metrics` | Stock metrics, ratios, and financial data | Fundamental analysis |
| `get_finviz_data` | Forex, futures, insider trading data | Traditional markets |
| `get_high_open_interest` | Options contracts with high open interest | Institutional sentiment |

### Cryptocurrency Exchange Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_coingecko_data` | Comprehensive crypto data and market overview | Detailed crypto research |
| `get_coingecko_market_context` | AI-powered market insights from CoinGecko | Market intelligence |
| `get_coingecko_trending_symbols` | Trending crypto symbols | Opportunity identification |
| `get_coingecko_gainers_losers` | Top crypto gainers and losers | Momentum analysis |
| `get_kraken_ticker` | Real-time crypto prices from Kraken | Live market data |
| `get_kraken_orderbook` | Kraken orderbook depth analysis | Liquidity analysis |
| `get_kraken_ohlc` | OHLC candlestick data for technical analysis | Price action analysis |
| `get_kraken_recent_trades` | Recent trading activity | Market momentum |
| `get_kraken_assets` | Available Kraken trading assets | Asset discovery |

### DeFi Protocol Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_hyperliquid_markets` | All Hyperliquid perpetual markets | DeFi derivatives analysis |
| `get_hyperliquid_trending` | Trending DeFi assets on Hyperliquid | Hot DeFi opportunities |
| `get_hyperliquid_top_volume` | Highest volume Hyperliquid markets | Activity analysis |
| `get_hyperliquid_movers` | Top DeFi gainers and losers | Momentum tracking |
| `get_hyperliquid_funding` | Perpetual funding rates | Market sentiment |
| `get_hyperliquid_market_data` | Specific asset market data | Deep asset analysis |
| `get_hyperliquid_overview` | Complete Hyperliquid market overview | DeFi market intelligence |

### Solana Ecosystem Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_solana_asset` | Detailed Solana asset information | Asset research |
| `get_solana_trending_nfts` | Trending Solana NFT collections | NFT market analysis |
| `get_solana_trending_tokens` | Trending Solana tokens | DeFi token discovery |
| `get_solana_top_collections` | Top NFT collections by metrics | NFT investment research |
| `get_solana_wallet_holdings` | Wallet portfolio analysis | Whale watching |
| `search_solana_assets` | Search assets by various criteria | Targeted Solana research |

### Meme Token & Pump.fun Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_pumpfun_trending` | Trending pump.fun meme tokens | Memecoin analysis |
| `get_pumpfun_token_info` | Detailed token information | Token due diligence |
| `get_pumpfun_market_summary` | Pump.fun market overview | Meme market intelligence |
| `search_pumpfun_tokens` | Search for specific meme tokens | Token discovery |
| `get_pumpfun_new_tokens` | Newly launched tokens | Fresh opportunity identification |
| `get_pumpfun_gainers_losers` | Top meme token movers | Momentum analysis |
| `get_pumpfun_bonding_curve` | Token bonding curve economics | Tokenomics analysis |

### System & Monitoring Tools
| Tool | Description | Use Case |
|------|-------------|----------|
| `get_system_health` | API system health status | Monitoring |
| `get_system_metrics` | Performance metrics | System analysis |

## 🤖 AI Agent Integration

### Claude Desktop
Add to your MCP configuration:
```json
{
  "mcpServers": {
    "trading-api": {
      "command": "python",
      "args": ["/path/to/trading-api/mcp_server.py"],
      "transport": "stdio"
    }
  }
}
```

### Research Agents (LangChain)
```python
from langchain_mcp_adapters.client import MultiServerMCPClient

mcp_config = {
    "trading_api": {
        "command": "python",
        "args": ["/path/to/trading-api/mcp_server.py"],
        "transport": "stdio"
    }
}

client = MultiServerMCPClient(mcp_config)
tools = await client.get_tools()
```

## 📊 Example AI Research Queries

### Traditional Markets
- *"Give me a comprehensive analysis of current market conditions"*
- *"What are the top trending stocks and what's driving their movement?"*
- *"Analyze Tesla across all data sources - news, sentiment, technicals"*
- *"What is the options market telling us about institutional positioning?"*
- *"Show me insider trading activity and high open interest options"*

### Cryptocurrency & DeFi
- *"What are the hottest DeFi opportunities on Hyperliquid right now?"*
- *"Compare Bitcoin funding rates across perpetual markets"*
- *"Show me trending Solana tokens with high volume"*
- *"Analyze the correlation between traditional crypto and DeFi markets"*
- *"What's driving the current crypto market sentiment?"*

### Meme Tokens & Emerging Markets
- *"Find newly launched meme tokens on pump.fun with strong momentum"*
- *"Analyze bonding curve economics for trending pump.fun tokens"*
- *"What are the top Solana NFT collections by volume today?"*
- *"Show me whale wallet activity in the Solana ecosystem"*

### Cross-Market Analysis
- *"How are traditional stocks correlating with crypto markets today?"*
- *"Compare meme token performance against traditional crypto"*
- *"Identify arbitrage opportunities between centralized and DeFi markets"*
- *"Analyze market sentiment across Reddit, news, and on-chain data"*

## 🏗️ Architecture

```
AI Agent <--MCP--> Python MCP Server <--HTTP--> Rust Trading API <--> Data Sources
                                                      |
                                                      ├── Traditional Finance
                                                      │   ├── Yahoo Finance
                                                      │   ├── Finviz  
                                                      │   ├── Reddit
                                                      │   └── Alpaca
                                                      │
                                                      ├── Cryptocurrency
                                                      │   ├── CoinGecko
                                                      │   ├── Kraken
                                                      │   └── Helius
                                                      │
                                                      ├── DeFi Protocols
                                                      │   └── Hyperliquid
                                                      │
                                                      ├── Solana Ecosystem
                                                      │   ├── Helius API
                                                      │   ├── Jito
                                                      │   └── Direct RPC
                                                      │
                                                      └── Meme Tokens
                                                          └── Pump.fun
```

### Components
- **Rust API**: High-performance data aggregation with caching, rate limiting, and error handling
- **Python MCP Server**: Standards-compliant MCP interface with 40+ specialized tools
- **Data Sources**: 15+ integrated sources spanning traditional finance, crypto, DeFi, and emerging markets
- **Trading Integration**: Paper and live trading capabilities via Kraken and Alpaca
- **Caching Layer**: Intelligent caching to optimize API calls and reduce latency
- **Monitoring**: Built-in health checks and performance metrics

## 📁 Repository Structure

```
trading-api/
├── src/                          # Rust trading API source code
│   ├── routes/                   # API endpoint handlers
│   │   ├── data.rs              # Traditional market data
│   │   ├── coingecko.rs         # CoinGecko crypto data
│   │   ├── kraken.rs            # Kraken exchange
│   │   ├── hyperliquid.rs       # DeFi derivatives
│   │   ├── solana.rs            # Solana ecosystem
│   │   ├── pumpfun.rs           # Meme token data
│   │   ├── yahoo.rs             # Yahoo Finance
│   │   ├── options.rs           # Options analysis
│   │   ├── high_open_interest.rs # High OI analysis
│   │   └── system.rs            # Health & monitoring
│   ├── sources/                 # Data source integrations
│   │   ├── finviz_data.rs       # Finviz scraping
│   │   ├── reddit_data.rs       # Reddit sentiment
│   │   ├── yahoo_data.rs        # Yahoo Finance API
│   │   ├── coingecko_data.rs    # CoinGecko API
│   │   ├── kraken_data.rs       # Kraken API
│   │   ├── hyperliquid_data.rs  # Hyperliquid API
│   │   ├── helius_data.rs       # Solana data via Helius
│   │   ├── pumpfun_data.rs      # Pump.fun integration
│   │   ├── alpaca_data.rs       # Options data
│   │   └── jito_data.rs         # Jito MEV data
│   ├── helpers/                 # Analysis and aggregation logic
│   │   ├── trending_stocks.rs   # Stock trend analysis
│   │   ├── trending_cryptos.rs  # Crypto trend analysis
│   │   ├── news.rs              # News aggregation
│   │   ├── metrics.rs           # Performance metrics
│   │   └── options.rs           # Options analysis
│   ├── trading/                 # Trading execution modules
│   │   ├── kraken_trading.rs    # Kraken trading
│   │   └── alpaca_trading.rs    # Alpaca trading
│   ├── middleware/              # HTTP middleware
│   ├── types.rs                 # Type definitions
│   ├── errors.rs                # Error handling
│   ├── config.rs                # Configuration management
│   ├── cache.rs                 # Caching layer
│   └── lib.rs                   # Main library
├── examples/                     # Usage examples
│   ├── coingecko_example.rs     # CoinGecko integration
│   ├── kraken_example.rs        # Kraken integration
│   ├── high_open_interest_example.rs # Options analysis
│   └── trending_options_example.rs   # Trending analysis
├── docs/                        # Comprehensive documentation
│   ├── API_DOCUMENTATION.md    # Complete API reference
│   ├── QUICK_START.md           # Quick setup guide
│   ├── MCP_SERVER_README.md     # MCP server details
│   ├── PRODUCTION_READINESS.md  # Production deployment
│   └── TROUBLESHOOTING.md       # Common issues
├── tests/                       # Test suites
│   ├── integration_tests.rs     # Integration tests
│   └── e2e.rs                   # End-to-end tests
├── mcp_server.py                # MCP server implementation (40+ tools)
├── research_agent_example.py    # AI research agent example
├── requirements.txt             # Python dependencies
├── setup_mcp.sh                 # Automated setup script
├── test_mcp.py                  # MCP server test suite
├── Cargo.toml                   # Rust dependencies
└── README.md                    # This comprehensive guide
```

## 🔧 Installation & Setup

### Prerequisites
- Rust 1.70+
- Python 3.8+
- pip and virtual environment support

### Automated Setup
```bash
# Clone and setup everything
git clone <your-repo>
cd trading-api
./setup_mcp.sh
```

### Manual Setup
```bash
# Rust dependencies
cargo build

# Python environment
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## 🧪 Testing

```bash
# Test Rust API
cargo test

# Test MCP server integration
python test_mcp.py

# Test with sample research queries
python research_agent_example.py
```

## 📝 Environment Variables

Create a `.env` file:
```bash
# API Keys (optional, for enhanced data)
REDDIT_CLIENT_ID=your_reddit_client_id
REDDIT_CLIENT_SECRET=your_reddit_client_secret
ALPHAVANTAGE_API_KEY=your_alphavantage_key

# Server Configuration
SERVER_HOST=localhost
SERVER_PORT=3000

# For research agent example
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
```

## 🚦 API Endpoints (Rust Server)

The Rust API provides comprehensive endpoints organized by category:

### System & Health
- `GET /health` - System health check
- `GET /metrics` - Performance metrics
- `GET /docs` - Interactive API documentation (Swagger UI)

### Traditional Markets
- `GET /trending/stocks` - Trending stocks from multiple sources
- `GET /reddit/stocks` - Reddit sentiment analysis
- `GET /news` - Aggregated financial news
- `GET /yahoo/metrics` - Yahoo Finance stock metrics
- `GET /forex` - Forex market data (Finviz)
- `GET /insider` - Insider trading activity
- `GET /high-open-interest` - High open interest options

### Cryptocurrency Exchanges
- `GET /trending/crypto` - Trending cryptocurrencies
- `GET /coingecko/top` - Top cryptocurrencies (CoinGecko)
- `GET /coingecko/trending` - Trending crypto symbols
- `GET /coingecko/gainers` - Top crypto gainers
- `GET /coingecko/losers` - Top crypto losers
- `GET /coingecko/market-context` - AI-powered market insights
- `GET /kraken/ticker` - Kraken ticker data
- `GET /kraken/orderbook/{pair}` - Orderbook depth
- `GET /kraken/ohlc/{pair}` - OHLC candlestick data
- `GET /kraken/trades/{pair}` - Recent trades
- `GET /kraken/assets` - Available trading assets

### DeFi Protocols
- `GET /hyperliquid/markets` - All Hyperliquid markets
- `GET /hyperliquid/trending` - Trending DeFi assets
- `GET /hyperliquid/volume/top` - Highest volume markets
- `GET /hyperliquid/movers` - Top gainers/losers
- `GET /hyperliquid/funding` - Funding rates
- `GET /hyperliquid/market/{coin}` - Specific market data
- `GET /hyperliquid/stats/overview` - Market overview

### Solana Ecosystem
- `GET /solana/asset/{asset_id}` - Asset details
- `GET /solana/nfts/trending` - Trending NFT collections
- `GET /solana/tokens/trending` - Trending tokens
- `GET /solana/collections/top` - Top NFT collections
- `GET /solana/wallet/{address}/holdings` - Wallet analysis
- `GET /solana/assets/search` - Asset search

### Meme Tokens (Pump.fun)
- `GET /pumpfun/trending` - Trending meme tokens
- `GET /pumpfun/token/{mint}` - Token details
- `GET /pumpfun/market-summary` - Market overview
- `GET /pumpfun/search/{query}` - Token search
- `GET /pumpfun/new-tokens` - Newly launched tokens
- `GET /pumpfun/top-gainers` - Top gaining tokens
- `GET /pumpfun/top-losers` - Top losing tokens
- `GET /pumpfun/bonding-curve/{mint}` - Bonding curve data

**📖 Complete Documentation:** Available at `http://localhost:3000/docs` when running the server.
**🔍 OpenAPI Spec:** Available at `http://localhost:3000/openapi.json`

## 🔍 Data Sources

### Traditional Finance
| Source | Data Type | Update Frequency | Features |
|--------|-----------|------------------|----------|
| **Yahoo Finance** | Stock prices, metrics, financial ratios | Real-time/Daily | Comprehensive stock analysis |
| **Finviz** | Stocks, forex, futures, insider trading | Real-time | Market screening, insider activity |
| **Reddit** | Social sentiment, trending discussions | Real-time | Social sentiment analysis |
| **Alpaca** | Options data, high open interest | Real-time | Options market intelligence |

### Cryptocurrency Exchanges  
| Source | Data Type | Update Frequency | Features |
|--------|-----------|------------------|----------|
| **CoinGecko** | Crypto prices, market data, trending | Real-time | Comprehensive crypto analytics |
| **Kraken** | Exchange data, orderbooks, trades | Real-time | Professional trading data |

### DeFi Protocols
| Source | Data Type | Update Frequency | Features |
|--------|-----------|------------------|----------|
| **Hyperliquid** | Perpetual markets, funding rates | Real-time | DeFi derivatives intelligence |

### Solana Ecosystem
| Source | Data Type | Update Frequency | Features |
|--------|-----------|------------------|----------|
| **Helius** | NFTs, tokens, wallet analysis | Real-time | Comprehensive Solana data |
| **Jito** | MEV data, block analysis | Real-time | MEV and validator insights |
| **Direct RPC** | On-chain data, program accounts | Real-time | Raw blockchain data |

### Meme Tokens & Emerging Markets
| Source | Data Type | Update Frequency | Features |
|--------|-----------|------------------|----------|
| **Pump.fun** | Meme tokens, bonding curves | Real-time | Complete meme token ecosystem |

### Data Quality & Reliability
- **Redundancy**: Multiple sources for critical data points
- **Validation**: Cross-source data validation and consistency checks  
- **Fallback**: Graceful degradation when sources are unavailable
- **Rate Limiting**: Intelligent rate limiting to respect API limits
- **Caching**: Smart caching to reduce API calls and improve performance

## 🛡️ Security & Performance

### Performance Optimizations
- **Intelligent Caching**: Multi-layer caching system with TTL-based invalidation
- **Connection Pooling**: Efficient HTTP connection management
- **Concurrent Processing**: Async/await throughout for maximum throughput
- **Rate Limiting**: Adaptive rate limiting per data source
- **Request Batching**: Batch similar requests to reduce API calls
- **Response Compression**: Gzip compression for API responses

### Security Features
- **Input Validation**: Comprehensive validation of all user inputs
- **Sanitization**: Data sanitization to prevent injection attacks
- **CORS Protection**: Configurable CORS policies
- **Error Handling**: Secure error messages without information leakage
- **API Key Management**: Secure handling of external API credentials
- **Request Logging**: Comprehensive request/response logging for audit trails

### Reliability & Monitoring
- **Health Checks**: Built-in health monitoring for all components
- **Circuit Breakers**: Automatic circuit breakers for failing services
- **Graceful Degradation**: Continue operating with reduced functionality
- **Metrics Collection**: Performance metrics and analytics
- **Error Recovery**: Automatic retry logic with exponential backoff
- **Service Discovery**: Dynamic service endpoint management

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support & Troubleshooting

### Common Issues

**"Failed to connect to API"**
- Ensure Rust API is running: `cargo run`
- Check port 3000 is available: `curl http://localhost:3000/health`

**"Tool not found"**
- Verify MCP server is running: `python mcp_server.py`
- Check tool names match exactly

**"Authentication errors"**
- Set API keys in `.env` file
- For research agent example, set `OPENAI_API_KEY` and `ANTHROPIC_API_KEY`

### Getting Help

- 📖 Check the documentation in this README
- 🧪 Run the test suite: `python test_mcp.py`
- 🔍 Check server logs for detailed error information
- 📝 Review the example integration: `research_agent_example.py`

## 🎯 What You Get

✅ **Production-ready MCP server** with 40+ specialized market research tools  
✅ **Comprehensive market coverage** across traditional finance, crypto, DeFi, and emerging markets  
✅ **Multi-source data aggregation** from 15+ major financial platforms  
✅ **Real-time market intelligence** with intelligent caching and rate limiting  
✅ **Solana ecosystem integration** including NFTs, tokens, and wallet analysis  
✅ **DeFi protocol support** with Hyperliquid derivatives and funding rates  
✅ **Meme token analytics** via Pump.fun integration with bonding curve analysis  
✅ **Trading capabilities** for both paper and live trading (Kraken, Alpaca)  
✅ **Easy AI integration** with any MCP-compatible system (Claude, LangChain, etc.)  
✅ **Comprehensive documentation** with examples, troubleshooting, and production guides  
✅ **Automated setup** with testing tools and health monitoring  
✅ **Enterprise-ready** with error handling, logging, metrics, and security features  

### 🚀 Transform Your AI Agents Into:
- **Sophisticated Market Researchers** with access to traditional and emerging markets
- **DeFi Intelligence Analysts** tracking derivatives and funding rates
- **Meme Token Specialists** analyzing bonding curves and social sentiment  
- **Cross-Market Arbitrage Detectors** identifying opportunities across ecosystems
- **Portfolio Analysts** with wallet tracking and whale watching capabilities
- **News & Sentiment Analyzers** combining social media and traditional news sources

**Ready to revolutionize your AI trading and research capabilities?** This comprehensive trading API MCP server provides everything you need to build intelligent, data-driven trading agents that understand both traditional and emerging financial markets.