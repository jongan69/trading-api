#!/usr/bin/env python3
"""
Trading API MCP Server

This server exposes comprehensive market data and analysis tools through the
Model Context Protocol (MCP), enabling AI agents to perform deep market research
across multiple data sources including stocks, cryptocurrencies, options, and news.

The server acts as a bridge between AI agents and the Rust trading API,
providing structured access to market data through standardized MCP tools.
"""

import asyncio
import json
import logging
from typing import Any, Dict, List, Optional, Sequence
from urllib.parse import urljoin

import httpx
from mcp.server import Server, NotificationOptions
from mcp.server.models import InitializationOptions
import mcp.server.stdio
import mcp.types as types

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("trading-api-mcp")

# Configuration
API_BASE_URL = "http://localhost:3000"  # Your Rust API base URL
DEFAULT_TIMEOUT = 30.0

class TradingApiMcpServer:
    """MCP Server for Trading API integration"""
    
    def __init__(self, api_base_url: str = API_BASE_URL):
        self.api_base_url = api_base_url
        self.client = httpx.AsyncClient(timeout=DEFAULT_TIMEOUT)
        self.server = Server("trading-api-mcp-server")
        self._setup_handlers()
    
    def _setup_handlers(self):
        """Setup MCP server handlers"""
        
        @self.server.list_tools()
        async def handle_list_tools() -> List[types.Tool]:
            """List available tools for market research"""
            return [
                types.Tool(
                    name="get_trending_stocks",
                    description="Get trending stocks from Reddit, Finviz, and other sources. Useful for identifying market sentiment and popular stocks.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of stocks to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_trending_crypto",
                    description="Get trending cryptocurrencies from multiple sources. Includes price data, market caps, and trend analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of cryptocurrencies to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_market_news",
                    description="Get aggregated financial news from multiple sources. Essential for understanding market sentiment and events.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of news items to return (default: 50)",
                                "default": 50
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_reddit_trending_stocks",
                    description="Get trending stocks from Reddit with sentiment indicators. Useful for social media-driven market analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of stocks to return (default: 25)",
                                "default": 25
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_coingecko_data",
                    description="Get comprehensive cryptocurrency data from CoinGecko including top coins, market overview, and price analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of cryptocurrencies to return (default: 100)",
                                "default": 100
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_kraken_ticker",
                    description="Get cryptocurrency tickers from Kraken exchange with real-time price data.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "pair": {
                                "type": "string",
                                "description": "Specific trading pair (e.g., XBTUSD). If not provided, returns all tickers.",
                                "optional": True
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_finviz_data",
                    description="Get market data from Finviz including forex, crypto, futures, and insider trading information.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "category": {
                                "type": "string",
                                "description": "Category of data: forex, crypto, future, insider, or group",
                                "enum": ["forex", "crypto", "future", "insider", "group"]
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of items to return (default: 50)",
                                "default": 50
                            }
                        },
                        "required": ["category"]
                    }
                ),
                types.Tool(
                    name="get_yahoo_metrics",
                    description="Get comprehensive stock metrics from Yahoo Finance including financial ratios and analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "symbols": {
                                "type": "string",
                                "description": "Comma-separated list of stock symbols (e.g., 'AAPL,MSFT,GOOGL')"
                            },
                            "range": {
                                "type": "string",
                                "description": "Time range for data (default: 1y)",
                                "default": "1y"
                            },
                            "interval": {
                                "type": "string",
                                "description": "Data interval (default: 1d)",
                                "default": "1d"
                            }
                        },
                        "required": ["symbols"]
                    }
                ),
                types.Tool(
                    name="get_high_open_interest",
                    description="Get option contracts with high open interest for identifying significant market positions.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of contracts to return (default: 50)",
                                "default": 50
                            },
                            "min_open_interest": {
                                "type": "integer",
                                "description": "Minimum open interest threshold",
                                "optional": True
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_comprehensive_market_analysis",
                    description="Get comprehensive market analysis combining data from all sources: stocks, crypto, news, and sentiment. Perfect for overall market research.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "include_news": {
                                "type": "boolean",
                                "description": "Include market news in analysis (default: true)",
                                "default": True
                            },
                            "include_crypto": {
                                "type": "boolean",
                                "description": "Include cryptocurrency data (default: true)",
                                "default": True
                            },
                            "include_stocks": {
                                "type": "boolean",
                                "description": "Include stock data (default: true)",
                                "default": True
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum items per category (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="search_market_data",
                    description="Search for specific symbols, keywords, or market data across all available sources. Useful for targeted research.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query (symbol, company name, or keyword)"
                            },
                            "sources": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Sources to search: stocks, crypto, news, or all (default: all)",
                                "default": ["all"]
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum results per source (default: 25)",
                                "default": 25
                            }
                        },
                        "required": ["query"]
                    }
                ),
                types.Tool(
                    name="get_pumpfun_trending",
                    description="Get trending pump.fun meme tokens with market data, volume, and price movements. Essential for memecoin analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of trending tokens to return (default: 20)",
                                "default": 20
                            },
                            "sort_by": {
                                "type": "string",
                                "description": "Sort by: market_cap, volume, created_at (default: market_cap)",
                                "default": "market_cap"
                            },
                            "time_range": {
                                "type": "string",
                                "description": "Time range: 1h, 24h, 7d, 30d (default: 24h)",
                                "default": "24h"
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_pumpfun_token_info",
                    description="Get detailed information about a specific pump.fun token including price, market cap, and bonding curve data.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "mint_address": {
                                "type": "string",
                                "description": "Token mint address (44-character Solana address)"
                            }
                        },
                        "required": ["mint_address"]
                    }
                ),
                types.Tool(
                    name="get_pumpfun_market_summary",
                    description="Get pump.fun market overview including total market cap, volume, new tokens, and top movers.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                types.Tool(
                    name="search_pumpfun_tokens",
                    description="Search for pump.fun tokens by name or symbol. Useful for finding specific meme tokens.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query (token name or symbol)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results to return (default: 20)",
                                "default": 20
                            }
                        },
                        "required": ["query"]
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_markets",
                    description="Get all available Hyperliquid perpetual markets with pricing and volume data. Essential for DeFi derivatives analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of markets to return (default: 50)",
                                "default": 50
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_trending",
                    description="Get trending DeFi assets on Hyperliquid with volume and price movements. Perfect for identifying hot DeFi derivatives.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of trending assets to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_top_volume",
                    description="Get top Hyperliquid markets by trading volume. Useful for identifying most active DeFi derivatives.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of markets to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_movers",
                    description="Get top gainers and losers on Hyperliquid. Essential for momentum analysis in DeFi derivatives.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of gainers/losers to return (default: 10)",
                                "default": 10
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_funding",
                    description="Get funding rates for all Hyperliquid perpetual contracts. Critical for understanding market sentiment in DeFi derivatives.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_market_data",
                    description="Get detailed market data for a specific Hyperliquid asset including price, volume, and open interest.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "coin": {
                                "type": "string",
                                "description": "Coin symbol (e.g., BTC, ETH, SOL)"
                            }
                        },
                        "required": ["coin"]
                    }
                ),
                types.Tool(
                    name="get_hyperliquid_overview",
                    description="Get comprehensive Hyperliquid market overview with statistics, volume, and key metrics.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                # Solana Ecosystem Tools
                types.Tool(
                    name="get_solana_asset",
                    description="Get detailed information about a specific Solana asset including NFTs and tokens by mint address.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "asset_id": {
                                "type": "string",
                                "description": "Solana asset mint address (44-character base58 string)"
                            }
                        },
                        "required": ["asset_id"]
                    }
                ),
                types.Tool(
                    name="get_solana_trending_nfts",
                    description="Get trending Solana NFT collections with volume, floor prices, and market activity. Essential for NFT market analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of NFT collections to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_solana_trending_tokens",
                    description="Get trending Solana tokens with price movements and trading volume. Perfect for Solana DeFi analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of tokens to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_solana_top_collections",
                    description="Get top Solana NFT collections by market cap, volume, and holder count. Critical for NFT investment research.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of collections to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_solana_wallet_holdings",
                    description="Analyze Solana wallet holdings including tokens and NFTs. Useful for portfolio analysis and whale watching.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "address": {
                                "type": "string",
                                "description": "Solana wallet address (44-character base58 string)"
                            }
                        },
                        "required": ["address"]
                    }
                ),
                types.Tool(
                    name="search_solana_assets",
                    description="Search Solana assets by various criteria including owner, creator, or collection. Perfect for targeted Solana research.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "owner": {
                                "type": "string",
                                "description": "Owner wallet address to search by"
                            },
                            "creator": {
                                "type": "string",
                                "description": "Creator address to search by"
                            },
                            "collection": {
                                "type": "string",
                                "description": "Collection address to search by"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of assets to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                # Advanced Kraken Tools
                types.Tool(
                    name="get_kraken_orderbook",
                    description="Get Kraken orderbook depth for specific trading pairs. Essential for analyzing market liquidity and support/resistance levels.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "pair": {
                                "type": "string",
                                "description": "Trading pair (e.g., XBTUSD, ETHUSD)"
                            },
                            "depth": {
                                "type": "integer",
                                "description": "Orderbook depth (default: 100)",
                                "default": 100
                            }
                        },
                        "required": ["pair"]
                    }
                ),
                types.Tool(
                    name="get_kraken_ohlc",
                    description="Get Kraken OHLC (candlestick) data for technical analysis. Critical for price action and trend analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "pair": {
                                "type": "string",
                                "description": "Trading pair (e.g., XBTUSD, ETHUSD)"
                            },
                            "interval": {
                                "type": "integer",
                                "description": "Time interval in minutes (1, 5, 15, 30, 60, 240, 1440, 10080, 21600)",
                                "default": 60
                            }
                        },
                        "required": ["pair"]
                    }
                ),
                types.Tool(
                    name="get_kraken_recent_trades",
                    description="Get recent trades from Kraken for specific pairs. Useful for analyzing trading activity and market momentum.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "pair": {
                                "type": "string",
                                "description": "Trading pair (e.g., XBTUSD, ETHUSD)"
                            },
                            "since": {
                                "type": "integer",
                                "description": "Return trades since this timestamp"
                            }
                        },
                        "required": ["pair"]
                    }
                ),
                types.Tool(
                    name="get_kraken_assets",
                    description="Get all available trading assets on Kraken with details. Useful for discovering new trading opportunities.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                # Enhanced CoinGecko Tools
                types.Tool(
                    name="get_coingecko_market_context",
                    description="Get AI-powered market context and insights from CoinGecko. Provides intelligent market analysis and commentary.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                types.Tool(
                    name="get_coingecko_trending_symbols",
                    description="Get trending cryptocurrency symbols from CoinGecko. Perfect for identifying emerging opportunities.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                types.Tool(
                    name="get_coingecko_gainers_losers",
                    description="Get top crypto gainers and losers from CoinGecko. Essential for momentum analysis and opportunity identification.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of gainers/losers to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                # System Health & Monitoring Tools
                types.Tool(
                    name="get_system_health",
                    description="Get trading API system health status including uptime, connectivity, and service availability.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                types.Tool(
                    name="get_system_metrics",
                    description="Get system performance metrics including response times, error rates, and resource usage.",
                    inputSchema={
                        "type": "object",
                        "properties": {}
                    }
                ),
                # Enhanced PumpFun Tools
                types.Tool(
                    name="get_pumpfun_new_tokens",
                    description="Get newly launched tokens on pump.fun. Critical for identifying fresh memecoin opportunities.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of new tokens to return (default: 20)",
                                "default": 20
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_pumpfun_gainers_losers",
                    description="Get top gaining and losing pump.fun tokens. Essential for memecoin momentum analysis.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of gainers/losers to return (default: 10)",
                                "default": 10
                            }
                        }
                    }
                ),
                types.Tool(
                    name="get_pumpfun_bonding_curve",
                    description="Get bonding curve data for specific pump.fun tokens. Critical for understanding token economics and price dynamics.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "mint_address": {
                                "type": "string",
                                "description": "Token mint address (44-character Solana address)"
                            }
                        },
                        "required": ["mint_address"]
                    }
                )
            ]
        
        @self.server.call_tool()
        async def handle_call_tool(
            name: str, arguments: Dict[str, Any]
        ) -> List[types.TextContent]:
            """Handle tool calls by routing to appropriate API endpoints"""
            
            try:
                if name == "get_trending_stocks":
                    return await self._get_trending_stocks(arguments)
                elif name == "get_trending_crypto":
                    return await self._get_trending_crypto(arguments)
                elif name == "get_market_news":
                    return await self._get_market_news(arguments)
                elif name == "get_reddit_trending_stocks":
                    return await self._get_reddit_trending_stocks(arguments)
                elif name == "get_coingecko_data":
                    return await self._get_coingecko_data(arguments)
                elif name == "get_kraken_ticker":
                    return await self._get_kraken_ticker(arguments)
                elif name == "get_finviz_data":
                    return await self._get_finviz_data(arguments)
                elif name == "get_yahoo_metrics":
                    return await self._get_yahoo_metrics(arguments)
                elif name == "get_high_open_interest":
                    return await self._get_high_open_interest(arguments)
                elif name == "get_comprehensive_market_analysis":
                    return await self._get_comprehensive_market_analysis(arguments)
                elif name == "search_market_data":
                    return await self._search_market_data(arguments)
                elif name == "get_pumpfun_trending":
                    return await self._get_pumpfun_trending(arguments)
                elif name == "get_pumpfun_token_info":
                    return await self._get_pumpfun_token_info(arguments)
                elif name == "get_pumpfun_market_summary":
                    return await self._get_pumpfun_market_summary(arguments)
                elif name == "search_pumpfun_tokens":
                    return await self._search_pumpfun_tokens(arguments)
                elif name == "get_hyperliquid_markets":
                    return await self._get_hyperliquid_markets(arguments)
                elif name == "get_hyperliquid_trending":
                    return await self._get_hyperliquid_trending(arguments)
                elif name == "get_hyperliquid_top_volume":
                    return await self._get_hyperliquid_top_volume(arguments)
                elif name == "get_hyperliquid_movers":
                    return await self._get_hyperliquid_movers(arguments)
                elif name == "get_hyperliquid_funding":
                    return await self._get_hyperliquid_funding(arguments)
                elif name == "get_hyperliquid_market_data":
                    return await self._get_hyperliquid_market_data(arguments)
                elif name == "get_hyperliquid_overview":
                    return await self._get_hyperliquid_overview(arguments)
                # Solana Tools
                elif name == "get_solana_asset":
                    return await self._get_solana_asset(arguments)
                elif name == "get_solana_trending_nfts":
                    return await self._get_solana_trending_nfts(arguments)
                elif name == "get_solana_trending_tokens":
                    return await self._get_solana_trending_tokens(arguments)
                elif name == "get_solana_top_collections":
                    return await self._get_solana_top_collections(arguments)
                elif name == "get_solana_wallet_holdings":
                    return await self._get_solana_wallet_holdings(arguments)
                elif name == "search_solana_assets":
                    return await self._search_solana_assets(arguments)
                # Kraken Tools
                elif name == "get_kraken_orderbook":
                    return await self._get_kraken_orderbook(arguments)
                elif name == "get_kraken_ohlc":
                    return await self._get_kraken_ohlc(arguments)
                elif name == "get_kraken_recent_trades":
                    return await self._get_kraken_recent_trades(arguments)
                elif name == "get_kraken_assets":
                    return await self._get_kraken_assets(arguments)
                # CoinGecko Tools
                elif name == "get_coingecko_market_context":
                    return await self._get_coingecko_market_context(arguments)
                elif name == "get_coingecko_trending_symbols":
                    return await self._get_coingecko_trending_symbols(arguments)
                elif name == "get_coingecko_gainers_losers":
                    return await self._get_coingecko_gainers_losers(arguments)
                # System Tools
                elif name == "get_system_health":
                    return await self._get_system_health(arguments)
                elif name == "get_system_metrics":
                    return await self._get_system_metrics(arguments)
                # Enhanced PumpFun Tools
                elif name == "get_pumpfun_new_tokens":
                    return await self._get_pumpfun_new_tokens(arguments)
                elif name == "get_pumpfun_gainers_losers":
                    return await self._get_pumpfun_gainers_losers(arguments)
                elif name == "get_pumpfun_bonding_curve":
                    return await self._get_pumpfun_bonding_curve(arguments)
                else:
                    raise ValueError(f"Unknown tool: {name}")
                    
            except Exception as e:
                logger.error(f"Error calling tool {name}: {e}")
                error_response = {
                    "success": False,
                    "error": str(e),
                    "tool": name,
                    "timestamp": asyncio.get_event_loop().time()
                }
                return [types.TextContent(
                    type="text",
                    text=json.dumps(error_response, indent=2)
                )]
    
    async def _make_api_request(self, endpoint: str, params: Optional[Dict] = None) -> Dict[str, Any]:
        """Make a request to the Rust API"""
        url = urljoin(self.api_base_url, endpoint)
        
        try:
            response = await self.client.get(url, params=params or {})
            response.raise_for_status()
            return response.json()
        except httpx.RequestError as e:
            logger.error(f"Request error for {url}: {e}")
            raise Exception(f"Failed to connect to API: {e}")
        except httpx.HTTPStatusError as e:
            logger.error(f"HTTP error for {url}: {e.response.status_code}")
            raise Exception(f"API returned error {e.response.status_code}: {e.response.text}")
    
    async def _get_trending_stocks(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending stocks from multiple sources"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/trending/stocks", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_trending_stocks",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_trending_crypto(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending cryptocurrencies"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/trending/crypto", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_trending_crypto",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_market_news(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get aggregated market news"""
        limit = args.get("limit", 50)
        data = await self._make_api_request("/news", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_market_news",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_reddit_trending_stocks(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Reddit trending stocks"""
        limit = args.get("limit", 25)
        data = await self._make_api_request("/reddit/stocks", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_reddit_trending_stocks",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_coingecko_data(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get CoinGecko cryptocurrency data"""
        limit = args.get("limit", 100)
        params = {
            "limit": limit,
            "vs_currency": "usd",
            "order": "market_cap_desc"
        }
        data = await self._make_api_request("/coingecko/top", params)
        
        response = {
            "success": True,
            "tool": "get_coingecko_data",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_kraken_ticker(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Kraken ticker data"""
        pair = args.get("pair")
        endpoint = f"/kraken/ticker/{pair}" if pair else "/kraken/ticker"
        data = await self._make_api_request(endpoint)
        
        response = {
            "success": True,
            "tool": "get_kraken_ticker",
            "data": data,
            "parameters": {"pair": pair} if pair else {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_finviz_data(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Finviz market data"""
        category = args["category"]
        limit = args.get("limit", 50)
        
        endpoint = f"/{category}"
        data = await self._make_api_request(endpoint, {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_finviz_data",
            "data": data,
            "parameters": {"category": category, "limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_yahoo_metrics(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Yahoo Finance metrics"""
        symbols = args["symbols"]
        range_param = args.get("range", "1y")
        interval = args.get("interval", "1d")
        
        params = {
            "symbols": symbols,
            "range": range_param,
            "interval": interval
        }
        data = await self._make_api_request("/yahoo/metrics", params)
        
        response = {
            "success": True,
            "tool": "get_yahoo_metrics",
            "data": data,
            "parameters": params,
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_high_open_interest(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get high open interest options"""
        limit = args.get("limit", 50)
        min_oi = args.get("min_open_interest")
        
        params = {"limit": limit}
        if min_oi:
            params["min_open_interest"] = min_oi
            
        data = await self._make_api_request("/high-open-interest", params)
        
        response = {
            "success": True,
            "tool": "get_high_open_interest",
            "data": data,
            "parameters": params,
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_comprehensive_market_analysis(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get comprehensive market analysis from multiple sources"""
        include_news = args.get("include_news", True)
        include_crypto = args.get("include_crypto", True)
        include_stocks = args.get("include_stocks", True)
        limit = args.get("limit", 20)
        
        analysis_data = {
            "success": True,
            "tool": "get_comprehensive_market_analysis",
            "analysis_type": "comprehensive_market_overview",
            "timestamp": asyncio.get_event_loop().time(),
            "data": {},
            "parameters": {
                "include_news": include_news,
                "include_crypto": include_crypto,
                "include_stocks": include_stocks,
                "limit": limit
            }
        }
        
        # Gather data from multiple sources
        try:
            if include_stocks:
                # Get trending stocks
                stocks_data = await self._make_api_request("/trending/stocks", {"limit": limit})
                analysis_data["data"]["trending_stocks"] = stocks_data
                
                # Get Reddit sentiment stocks
                reddit_data = await self._make_api_request("/reddit/stocks", {"limit": limit})
                analysis_data["data"]["reddit_sentiment_stocks"] = reddit_data
            
            if include_crypto:
                # Get trending crypto
                crypto_data = await self._make_api_request("/trending/crypto", {"limit": limit})
                analysis_data["data"]["trending_cryptocurrencies"] = crypto_data
                
                # Get CoinGecko data
                try:
                    coingecko_data = await self._make_api_request("/coingecko/top", {"limit": limit})
                    analysis_data["data"]["coingecko_data"] = coingecko_data
                except Exception as e:
                    logger.warning(f"Failed to get CoinGecko data: {e}")
                
                # Get pump.fun trending tokens
                try:
                    pumpfun_data = await self._make_api_request("/pumpfun/trending", {"limit": limit})
                    analysis_data["data"]["pumpfun_trending"] = pumpfun_data
                except Exception as e:
                    logger.warning(f"Failed to get pump.fun data: {e}")
                
                # Get pump.fun market summary
                try:
                    pumpfun_summary = await self._make_api_request("/pumpfun/market-summary")
                    analysis_data["data"]["pumpfun_market_summary"] = pumpfun_summary
                except Exception as e:
                    logger.warning(f"Failed to get pump.fun market summary: {e}")
                
                # Get Hyperliquid trending DeFi assets
                try:
                    hyperliquid_trending = await self._make_api_request("/hyperliquid/trending", {"limit": limit})
                    analysis_data["data"]["hyperliquid_trending"] = hyperliquid_trending
                except Exception as e:
                    logger.warning(f"Failed to get Hyperliquid trending data: {e}")
                
                # Get Hyperliquid top volume markets
                try:
                    hyperliquid_volume = await self._make_api_request("/hyperliquid/volume/top", {"limit": limit})
                    analysis_data["data"]["hyperliquid_top_volume"] = hyperliquid_volume
                except Exception as e:
                    logger.warning(f"Failed to get Hyperliquid volume data: {e}")
                
                # Get Hyperliquid movers (gainers/losers)
                try:
                    hyperliquid_movers = await self._make_api_request("/hyperliquid/movers", {"limit": limit})
                    analysis_data["data"]["hyperliquid_movers"] = hyperliquid_movers
                except Exception as e:
                    logger.warning(f"Failed to get Hyperliquid movers data: {e}")
                
                # Get Solana trending tokens
                try:
                    solana_trending = await self._make_api_request("/solana/tokens/trending", {"limit": limit})
                    analysis_data["data"]["solana_trending_tokens"] = solana_trending
                except Exception as e:
                    logger.warning(f"Failed to get Solana trending tokens: {e}")
                
                # Get Solana trending NFTs
                try:
                    solana_nfts = await self._make_api_request("/solana/nfts/trending", {"limit": limit})
                    analysis_data["data"]["solana_trending_nfts"] = solana_nfts
                except Exception as e:
                    logger.warning(f"Failed to get Solana trending NFTs: {e}")
            
            if include_news:
                # Get market news
                news_data = await self._make_api_request("/news", {"limit": limit})
                analysis_data["data"]["market_news"] = news_data
                
        except Exception as e:
            logger.error(f"Error gathering comprehensive analysis: {e}")
            analysis_data["data"]["error"] = str(e)
        
        return [types.TextContent(type="text", text=json.dumps(analysis_data, indent=2))]
    
    async def _search_market_data(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Search market data across sources"""
        query = args["query"]
        sources = args.get("sources", ["all"])
        limit = args.get("limit", 25)
        
        search_results = {
            "success": True,
            "tool": "search_market_data",
            "query": query,
            "sources_searched": sources,
            "timestamp": asyncio.get_event_loop().time(),
            "results": {},
            "parameters": {"query": query, "sources": sources, "limit": limit}
        }
        
        try:
            # This is a simplified search - in practice, you'd implement
            # more sophisticated search across your API endpoints
            if "stocks" in sources or "all" in sources:
                # Search trending stocks
                stocks_data = await self._make_api_request("/trending/stocks", {"limit": 100})
                # Filter results based on query (simplified)
                search_results["results"]["stocks"] = {
                    "note": "Search functionality requires API-side implementation",
                    "query": query,
                    "available_data": stocks_data
                }
            
            if "crypto" in sources or "all" in sources:
                # Search crypto data
                crypto_data = await self._make_api_request("/trending/crypto", {"limit": 100})
                search_results["results"]["crypto"] = {
                    "note": "Search functionality requires API-side implementation",
                    "query": query,
                    "available_data": crypto_data
                }
            
            if "news" in sources or "all" in sources:
                # Search news
                news_data = await self._make_api_request("/news", {"limit": 100})
                search_results["results"]["news"] = {
                    "note": "Search functionality requires API-side implementation",
                    "query": query,
                    "available_data": news_data
                }
                
        except Exception as e:
            logger.error(f"Error in search: {e}")
            search_results["results"]["error"] = str(e)
        
        return [types.TextContent(type="text", text=json.dumps(search_results, indent=2))]
    
    async def _get_pumpfun_trending(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending pump.fun tokens"""
        limit = args.get("limit", 20)
        sort_by = args.get("sort_by", "market_cap")
        time_range = args.get("time_range", "24h")
        
        params = {
            "limit": limit,
            "sort_by": sort_by,
            "time_range": time_range
        }
        data = await self._make_api_request("/pumpfun/trending", params)
        
        response = {
            "success": True,
            "tool": "get_pumpfun_trending",
            "data": data,
            "parameters": params,
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_pumpfun_token_info(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get detailed pump.fun token information"""
        mint_address = args["mint_address"]
        
        # Validate mint address format
        if len(mint_address) != 44:
            error_response = {
                "success": False,
                "error": "Invalid mint address format (must be 44 characters)",
                "tool": "get_pumpfun_token_info",
                "timestamp": asyncio.get_event_loop().time()
            }
            return [types.TextContent(type="text", text=json.dumps(error_response, indent=2))]
        
        data = await self._make_api_request(f"/pumpfun/token/{mint_address}")
        
        response = {
            "success": True,
            "tool": "get_pumpfun_token_info",
            "data": data,
            "parameters": {"mint_address": mint_address},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_pumpfun_market_summary(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get pump.fun market summary"""
        data = await self._make_api_request("/pumpfun/market-summary")
        
        response = {
            "success": True,
            "tool": "get_pumpfun_market_summary",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _search_pumpfun_tokens(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Search pump.fun tokens"""
        query = args["query"]
        limit = args.get("limit", 20)
        
        params = {"limit": limit}
        data = await self._make_api_request(f"/pumpfun/search/{query}", params)
        
        response = {
            "success": True,
            "tool": "search_pumpfun_tokens",
            "data": data,
            "parameters": {"query": query, "limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_markets(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get all Hyperliquid markets"""
        limit = args.get("limit", 50)
        data = await self._make_api_request("/hyperliquid/markets")
        
        # Limit results if specified
        if isinstance(data, dict) and "data" in data and isinstance(data["data"], list):
            if limit and len(data["data"]) > limit:
                data["data"] = data["data"][:limit]
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_markets",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_trending(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending Hyperliquid DeFi assets"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/hyperliquid/trending", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_trending",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_top_volume(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get top Hyperliquid markets by volume"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/hyperliquid/volume/top", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_top_volume",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_movers(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get top Hyperliquid gainers and losers"""
        limit = args.get("limit", 10)
        data = await self._make_api_request("/hyperliquid/movers", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_movers",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_funding(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Hyperliquid funding rates"""
        data = await self._make_api_request("/hyperliquid/funding")
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_funding",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_market_data(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get specific Hyperliquid market data"""
        coin = args["coin"].upper()
        data = await self._make_api_request(f"/hyperliquid/market/{coin}")
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_market_data",
            "data": data,
            "parameters": {"coin": coin},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_hyperliquid_overview(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Hyperliquid market overview"""
        data = await self._make_api_request("/hyperliquid/stats/overview")
        
        response = {
            "success": True,
            "tool": "get_hyperliquid_overview",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    # ===== SOLANA ECOSYSTEM TOOLS =====
    
    async def _get_solana_asset(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Solana asset details"""
        asset_id = args["asset_id"]
        data = await self._make_api_request(f"/solana/asset/{asset_id}")
        
        response = {
            "success": True,
            "tool": "get_solana_asset",
            "data": data,
            "parameters": {"asset_id": asset_id},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_solana_trending_nfts(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending Solana NFTs"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/solana/nfts/trending", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_solana_trending_nfts",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_solana_trending_tokens(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get trending Solana tokens"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/solana/tokens/trending", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_solana_trending_tokens",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_solana_top_collections(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get top Solana NFT collections"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/solana/collections/top", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_solana_top_collections",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_solana_wallet_holdings(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Solana wallet holdings"""
        address = args["address"]
        data = await self._make_api_request(f"/solana/wallet/{address}/holdings")
        
        response = {
            "success": True,
            "tool": "get_solana_wallet_holdings",
            "data": data,
            "parameters": {"address": address},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _search_solana_assets(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Search Solana assets"""
        params = {}
        if "owner" in args:
            params["owner"] = args["owner"]
        if "creator" in args:
            params["creator"] = args["creator"]
        if "collection" in args:
            params["collection"] = args["collection"]
        params["limit"] = args.get("limit", 20)
        
        data = await self._make_api_request("/solana/assets/search", params)
        
        response = {
            "success": True,
            "tool": "search_solana_assets",
            "data": data,
            "parameters": params,
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    # ===== ADVANCED KRAKEN TOOLS =====
    
    async def _get_kraken_orderbook(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Kraken orderbook"""
        pair = args["pair"]
        depth = args.get("depth", 100)
        data = await self._make_api_request(f"/kraken/orderbook/{pair}", {"depth": depth})
        
        response = {
            "success": True,
            "tool": "get_kraken_orderbook",
            "data": data,
            "parameters": {"pair": pair, "depth": depth},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_kraken_ohlc(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Kraken OHLC data"""
        pair = args["pair"]
        interval = args.get("interval", 60)
        data = await self._make_api_request(f"/kraken/ohlc/{pair}", {"interval": interval})
        
        response = {
            "success": True,
            "tool": "get_kraken_ohlc",
            "data": data,
            "parameters": {"pair": pair, "interval": interval},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_kraken_recent_trades(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Kraken recent trades"""
        pair = args["pair"]
        params = {"pair": pair}
        if "since" in args:
            params["since"] = args["since"]
        
        data = await self._make_api_request(f"/kraken/trades/{pair}", params)
        
        response = {
            "success": True,
            "tool": "get_kraken_recent_trades",
            "data": data,
            "parameters": params,
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_kraken_assets(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get Kraken assets"""
        data = await self._make_api_request("/kraken/assets")
        
        response = {
            "success": True,
            "tool": "get_kraken_assets",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    # ===== ENHANCED COINGECKO TOOLS =====
    
    async def _get_coingecko_market_context(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get CoinGecko market context"""
        data = await self._make_api_request("/coingecko/market-context")
        
        response = {
            "success": True,
            "tool": "get_coingecko_market_context",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_coingecko_trending_symbols(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get CoinGecko trending symbols"""
        data = await self._make_api_request("/coingecko/trending-symbols")
        
        response = {
            "success": True,
            "tool": "get_coingecko_trending_symbols",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_coingecko_gainers_losers(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get CoinGecko gainers and losers"""
        limit = args.get("limit", 20)
        
        # Get both gainers and losers
        try:
            gainers_data = await self._make_api_request("/coingecko/gainers", {"limit": limit})
            losers_data = await self._make_api_request("/coingecko/losers", {"limit": limit})
            
            combined_data = {
                "gainers": gainers_data,
                "losers": losers_data
            }
        except Exception as e:
            logger.warning(f"Failed to get gainers/losers separately, trying combined: {e}")
            combined_data = await self._make_api_request("/coingecko/movers", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_coingecko_gainers_losers",
            "data": combined_data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    # ===== SYSTEM HEALTH & MONITORING TOOLS =====
    
    async def _get_system_health(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get system health status"""
        data = await self._make_api_request("/health")
        
        response = {
            "success": True,
            "tool": "get_system_health",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_system_metrics(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get system metrics"""
        data = await self._make_api_request("/metrics")
        
        response = {
            "success": True,
            "tool": "get_system_metrics",
            "data": data,
            "parameters": {},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    # ===== ENHANCED PUMPFUN TOOLS =====
    
    async def _get_pumpfun_new_tokens(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get new PumpFun tokens"""
        limit = args.get("limit", 20)
        data = await self._make_api_request("/pumpfun/new-tokens", {"limit": limit})
        
        response = {
            "success": True,
            "tool": "get_pumpfun_new_tokens",
            "data": data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_pumpfun_gainers_losers(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get PumpFun gainers and losers"""
        limit = args.get("limit", 10)
        
        try:
            gainers_data = await self._make_api_request("/pumpfun/top-gainers", {"limit": limit})
            losers_data = await self._make_api_request("/pumpfun/top-losers", {"limit": limit})
            
            combined_data = {
                "gainers": gainers_data,
                "losers": losers_data
            }
        except Exception as e:
            logger.warning(f"Failed to get PumpFun gainers/losers: {e}")
            combined_data = {"error": "Failed to fetch gainers/losers data"}
        
        response = {
            "success": True,
            "tool": "get_pumpfun_gainers_losers",
            "data": combined_data,
            "parameters": {"limit": limit},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def _get_pumpfun_bonding_curve(self, args: Dict[str, Any]) -> List[types.TextContent]:
        """Get PumpFun bonding curve data"""
        mint_address = args["mint_address"]
        data = await self._make_api_request(f"/pumpfun/bonding-curve/{mint_address}")
        
        response = {
            "success": True,
            "tool": "get_pumpfun_bonding_curve",
            "data": data,
            "parameters": {"mint_address": mint_address},
            "timestamp": asyncio.get_event_loop().time()
        }
        
        return [types.TextContent(type="text", text=json.dumps(response, indent=2))]
    
    async def run(self):
        """Run the MCP server"""
        async with mcp.server.stdio.stdio_server() as (read_stream, write_stream):
            await self.server.run(
                read_stream,
                write_stream,
                InitializationOptions(
                    server_name="trading-api-mcp-server",
                    server_version="1.0.0",
                    capabilities=self.server.get_capabilities(
                        notification_options=NotificationOptions(),
                        experimental_capabilities={},
                    ),
                ),
            )


async def main():
    """Main entry point"""
    server = TradingApiMcpServer()
    await server.run()


if __name__ == "__main__":
    asyncio.run(main())
