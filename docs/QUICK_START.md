# Quick Start: Trading API MCP Server

Get your trading API running as an MCP server for AI agents in just a few steps!

## 🚀 Quick Setup (5 minutes)

### 1. Setup the MCP Server
```bash
# Run the automated setup
./setup_mcp.sh
```

### 2. Start Your Services
```bash
# Terminal 1: Start the Rust trading API
cargo run

# Terminal 2: Activate Python environment and test
source venv/bin/activate
python test_mcp.py
```

### 3. Run the MCP Server
```bash
# In the activated Python environment
python mcp_server.py
```

That's it! Your MCP server is now running and ready to serve AI agents.

## 🧪 Quick Test

Test that everything works:

```bash
# Test the API endpoints
python test_mcp.py

# Should show:
# ✅ Rust API is running and accessible
# ✅ /trending/stocks - OK
# ✅ /trending/crypto - OK
# ... etc
```

## 🤖 Use with AI Agents

### With Claude Desktop (MCP)

Add to your Claude Desktop MCP config:
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

### With Research Agents

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

# Now use tools for market research!
```

## 🛠️ Available Tools

Your MCP server exposes 11 powerful market research tools:

- **`get_comprehensive_market_analysis`** - Complete market overview
- **`get_trending_stocks`** - Hot stocks from multiple sources  
- **`get_trending_crypto`** - Trending cryptocurrencies
- **`get_market_news`** - Aggregated financial news
- **`get_reddit_trending_stocks`** - Reddit sentiment analysis
- **`search_market_data`** - Search across all data sources
- **`get_coingecko_data`** - Comprehensive crypto data
- **`get_kraken_ticker`** - Real-time crypto prices
- **`get_finviz_data`** - Forex, futures, insider data
- **`get_yahoo_metrics`** - Stock metrics and ratios
- **`get_high_open_interest`** - Options market positioning

## 📊 Example Queries for AI Agents

Try these research queries with your AI agent:

1. **Market Overview**: *"Give me a comprehensive analysis of current market conditions across stocks, crypto, and news."*

2. **Trend Analysis**: *"What are the top trending stocks and cryptocurrencies right now? What's driving their movement?"*

3. **Sentiment Analysis**: *"Analyze market sentiment from Reddit and news sources. Are we in a bull or bear market?"*

4. **Asset Research**: *"Research Tesla (TSLA) across all available data sources. Include news, sentiment, and technical analysis."*

5. **Options Analysis**: *"What is the options market telling us about institutional positioning? Look at high open interest contracts."*

6. **Cross-Asset Analysis**: *"How are traditional stocks correlating with cryptocurrency markets today?"*

## 🔧 Troubleshooting

**API not connecting?**
- Ensure Rust API is running: `cargo run`
- Check it's on port 3000: `curl http://localhost:3000/health`

**MCP server issues?**
- Activate virtual environment: `source venv/bin/activate`
- Install dependencies: `pip install -r requirements.txt`

**Tool errors?**
- Check logs in the MCP server output
- Verify API endpoints with: `python test_mcp.py`

## 📚 Next Steps

- Read the full [MCP_SERVER_README.md](MCP_SERVER_README.md) for detailed documentation
- Check out [research_agent_example.py](research_agent_example.py) for integration examples
- Customize the MCP server for your specific use cases

## 🎯 What You've Built

You now have a powerful MCP server that:
- ✅ Exposes your comprehensive trading API to AI agents
- ✅ Provides 11 specialized market research tools
- ✅ Handles errors gracefully and returns structured data
- ✅ Works with any MCP-compatible AI system
- ✅ Enables deep market research and analysis

Your AI agents can now perform sophisticated market research using your trading data!
