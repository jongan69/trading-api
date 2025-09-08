# Trading API MCP Server

This directory contains a Model Context Protocol (MCP) server that exposes your comprehensive trading API to AI agents for deep market research and analysis.

## Overview

The MCP server acts as a bridge between AI agents and your Rust trading API, providing structured access to:

- **Stock Market Data**: Trending stocks from Reddit, Finviz, and other sources
- **Cryptocurrency Data**: Real-time crypto prices from CoinGecko, Kraken, and aggregated sources  
- **Market News**: Aggregated financial news and sentiment analysis
- **Options Data**: High open interest contracts and derivatives analysis
- **Technical Analysis**: Yahoo Finance metrics, recommendations, and financial ratios
- **Comprehensive Analysis**: Multi-source market overview combining all data types

## Architecture

```
AI Agent <--MCP--> Python MCP Server <--HTTP--> Rust Trading API <--> Data Sources
```

The Python MCP server communicates with AI agents via the Model Context Protocol and makes HTTP requests to your existing Rust API endpoints.

## Setup Instructions

### Prerequisites

- Python 3.8+ 
- Your Rust trading API running on `http://localhost:3000`
- pip and virtual environment support

### Installation

1. **Run the setup script:**
   ```bash
   ./setup_mcp.sh
   ```

   This will:
   - Create a Python virtual environment
   - Install required dependencies
   - Make the MCP server executable

2. **Manual setup (alternative):**
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   ```

### Running the Server

1. **Start your Rust API:**
   ```bash
   cargo run
   ```

2. **In another terminal, start the MCP server:**
   ```bash
   source venv/bin/activate
   python mcp_server.py
   ```

The MCP server will start and communicate via stdin/stdout, which is the standard for MCP servers.

## Available Tools

The MCP server exposes the following tools to AI agents:

### Core Market Data Tools

1. **`get_trending_stocks`**
   - Get trending stocks from Reddit, Finviz, and other sources
   - Parameters: `limit` (default: 20)
   - Useful for identifying market sentiment and popular stocks

2. **`get_trending_crypto`**
   - Get trending cryptocurrencies with price data and market caps
   - Parameters: `limit` (default: 20)
   - Includes trend analysis from multiple sources

3. **`get_market_news`**
   - Get aggregated financial news from multiple sources
   - Parameters: `limit` (default: 50)
   - Essential for understanding market sentiment and events

4. **`get_reddit_trending_stocks`**
   - Get trending stocks from Reddit with sentiment indicators
   - Parameters: `limit` (default: 25)
   - Useful for social media-driven market analysis

### Data Source Specific Tools

5. **`get_coingecko_data`**
   - Get comprehensive cryptocurrency data from CoinGecko
   - Parameters: `limit` (default: 100)
   - Includes top coins, market overview, and price analysis

6. **`get_kraken_ticker`**
   - Get cryptocurrency tickers from Kraken exchange
   - Parameters: `pair` (optional, e.g., "XBTUSD")
   - Real-time price data from major crypto exchange

7. **`get_finviz_data`**
   - Get market data from Finviz
   - Parameters: `category` (forex|crypto|future|insider|group), `limit` (default: 50)
   - Includes forex, futures, and insider trading information

8. **`get_yahoo_metrics`**
   - Get comprehensive stock metrics from Yahoo Finance
   - Parameters: `symbols` (required), `range` (default: "1y"), `interval` (default: "1d")
   - Financial ratios, recommendations, and technical analysis

9. **`get_high_open_interest`**
   - Get option contracts with high open interest
   - Parameters: `limit` (default: 50), `min_open_interest` (optional)
   - Identifies significant market positions

### Analysis Tools

10. **`get_comprehensive_market_analysis`**
    - Get comprehensive analysis combining all data sources
    - Parameters: `include_news`, `include_crypto`, `include_stocks`, `limit` (default: 20)
    - Perfect for overall market research and trend analysis

11. **`search_market_data`**
    - Search for specific symbols or keywords across all sources
    - Parameters: `query` (required), `sources` (default: ["all"]), `limit` (default: 25)
    - Useful for targeted research on specific assets or topics

## Integration with AI Agents

### Example: Using with the Research Agent

The MCP server is designed to work with AI research agents like the one shown in your example. Here's how to integrate it:

1. **Configure the MCP client in your research agent:**
   ```python
   mcp_config = {
       "trading_api": {
           "command": "python",
           "args": ["/path/to/trading-api/mcp_server.py"],
           "transport": "stdio"
       }
   }
   ```

2. **Use the tools in your agent:**
   ```python
   # Get comprehensive market analysis
   market_analysis = await client.call_tool(
       "get_comprehensive_market_analysis",
       {"limit": 30, "include_news": True}
   )
   
   # Search for specific stocks
   tesla_data = await client.call_tool(
       "search_market_data",
       {"query": "TSLA", "sources": ["stocks", "news"]}
   )
   ```

### Example: AI Agent Workflow

A typical AI research workflow might look like:

1. **Market Overview**: Use `get_comprehensive_market_analysis` to get overall market state
2. **Trend Analysis**: Use `get_trending_stocks` and `get_trending_crypto` for trend identification
3. **Sentiment Analysis**: Use `get_reddit_trending_stocks` and `get_market_news` for sentiment
4. **Deep Dive**: Use `search_market_data` for specific assets of interest
5. **Technical Analysis**: Use `get_yahoo_metrics` for detailed financial metrics
6. **Risk Assessment**: Use `get_high_open_interest` for options market positioning

## Configuration

### API Base URL

The MCP server is configured to connect to `http://localhost:3000` by default. To change this:

1. Edit `mcp_server.py`
2. Modify the `API_BASE_URL` constant
3. Restart the MCP server

### Timeout Settings

Default timeout is 30 seconds. Modify `DEFAULT_TIMEOUT` in `mcp_server.py` if needed.

### Logging

The server logs to stdout with INFO level by default. Modify the logging configuration in `mcp_server.py` for different log levels.

## Error Handling

The MCP server includes comprehensive error handling:

- **Connection Errors**: Gracefully handles API unavailability
- **HTTP Errors**: Provides detailed error messages for API failures
- **Timeout Handling**: Configurable timeouts prevent hanging requests
- **Validation**: Input validation for all tool parameters

All errors are returned as structured JSON responses to the AI agent.

## Development

### Adding New Tools

To add new tools to the MCP server:

1. Add the tool definition in `_setup_handlers()` -> `handle_list_tools()`
2. Add the handler in `handle_call_tool()`
3. Implement the handler method (e.g., `_get_new_tool()`)
4. Update this documentation

### Testing

Test the MCP server by:

1. Starting your Rust API
2. Running the MCP server
3. Using an MCP client to call tools
4. Verifying responses match expected format

### Debugging

Enable debug logging by modifying the logging level:
```python
logging.basicConfig(level=logging.DEBUG)
```

## Troubleshooting

### Common Issues

1. **"Failed to connect to API"**
   - Ensure your Rust API is running on the correct port
   - Check firewall settings
   - Verify API_BASE_URL configuration

2. **"Tool not found"**
   - Ensure the tool name matches exactly
   - Check the tool is properly registered in `handle_list_tools()`

3. **"Import errors"**
   - Ensure virtual environment is activated
   - Run `pip install -r requirements.txt`

4. **"JSON decode errors"**
   - Check API response format
   - Verify API endpoints are returning valid JSON

### Logs

Check MCP server logs for detailed error information. The server logs all API requests and responses.

## Performance Considerations

- **Caching**: Consider implementing response caching for frequently requested data
- **Rate Limiting**: The server respects API rate limits automatically
- **Concurrent Requests**: Multiple tools can be called concurrently by AI agents
- **Memory Usage**: Large datasets are streamed rather than loaded entirely into memory

## Security

- **Local Network**: Server connects to localhost API by default
- **No Authentication**: Add authentication if exposing beyond localhost
- **Input Validation**: All inputs are validated before API calls
- **Error Sanitization**: Error messages don't expose sensitive information

## License

This MCP server implementation follows the same license as your trading API project.
