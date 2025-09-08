#!/usr/bin/env python3
"""
Research Agent with Trading API MCP Integration

This example demonstrates how to integrate the Trading API MCP server
with a research agent for comprehensive market analysis.

Based on the research agent pattern you provided, this shows how to:
1. Configure the MCP client to connect to the trading API
2. Use trading-specific tools for market research
3. Compress and analyze market data
"""

import os
import asyncio
import time
from datetime import datetime
from typing_extensions import Literal
from pathlib import Path
import tiktoken

from langchain.chat_models import init_chat_model
from langchain_core.messages import SystemMessage, HumanMessage, ToolMessage, filter_messages
from langchain_mcp_adapters.client import MultiServerMCPClient
from langgraph.graph import StateGraph, START, END

from dotenv import load_dotenv
load_dotenv()

# Import your existing state and prompt modules
# from deep_research_from_scratch.prompts import research_agent_prompt_with_mcp, compress_research_system_prompt, compress_research_human_message
# from deep_research_from_scratch.state_research import ResearcherState, ResearcherOutputState
# from deep_research_from_scratch.utils import get_today_str, think_tool, get_current_dir

# ===== CONFIGURATION =====

# MCP server configuration for trading API access
trading_mcp_config = {
    "trading_api": {
        "command": "python3",
        "args": [
            str(Path(__file__).parent / "mcp_server.py")  # Path to trading MCP server
        ],
        "transport": "stdio"
    }
}

# Global client variable - will be initialized lazily
_client = None

# Rate limiting configuration
class RateLimiter:
    def __init__(self, tokens_per_minute=45000, requests_per_minute=45):  # Conservative limits
        self.tokens_per_minute = tokens_per_minute
        self.requests_per_minute = requests_per_minute
        self.token_usage = []
        self.request_times = []
        self.encoding = tiktoken.get_encoding("cl100k_base")  # Approximate for Claude
    
    def count_tokens(self, text):
        """Approximate token count for text."""
        return len(self.encoding.encode(str(text)))
    
    def can_make_request(self, estimated_tokens):
        """Check if we can make a request without exceeding rate limits."""
        now = time.time()
        
        # Remove old entries (older than 1 minute)
        self.token_usage = [(t, tokens) for t, tokens in self.token_usage if now - t < 60]
        self.request_times = [t for t in self.request_times if now - t < 60]
        
        # Check token limit
        current_tokens = sum(tokens for _, tokens in self.token_usage)
        if current_tokens + estimated_tokens > self.tokens_per_minute:
            return False, f"Would exceed token limit: {current_tokens + estimated_tokens}/{self.tokens_per_minute}"
        
        # Check request limit
        if len(self.request_times) >= self.requests_per_minute:
            return False, f"Would exceed request limit: {len(self.request_times)}/{self.requests_per_minute}"
        
        return True, "OK"
    
    def record_request(self, tokens_used):
        """Record a successful request."""
        now = time.time()
        self.token_usage.append((now, tokens_used))
        self.request_times.append(now)
    
    async def wait_if_needed(self, estimated_tokens):
        """Wait if necessary to respect rate limits."""
        can_proceed, reason = self.can_make_request(estimated_tokens)
        
        if not can_proceed:
            print(f"⏳ Rate limit protection: {reason}")
            print("   Waiting 60 seconds to reset limits...")
            await asyncio.sleep(60)
            return await self.wait_if_needed(estimated_tokens)  # Recursive check
        
        return True

# Global rate limiter
rate_limiter = RateLimiter()

def get_mcp_client():
    """Get or initialize MCP client lazily to avoid issues with LangGraph Platform."""
    global _client
    if _client is None:
        _client = MultiServerMCPClient(trading_mcp_config)
    return _client

# Initialize models - using Claude Haiku 3.5 for higher rate limits (50k tokens/min vs 30k)
compress_model = init_chat_model(model="openai:gpt-4o-mini", max_tokens=16000)
model = init_chat_model(model="anthropic:claude-3-5-haiku-20241022", max_tokens=4000)

# ===== TRADING-SPECIFIC PROMPTS =====

trading_research_prompt = """You are a sophisticated financial research agent with access to comprehensive market data tools through the Trading API MCP server.

Your role is to conduct deep market research and analysis using the following tools:

MARKET DATA TOOLS:
- get_trending_stocks: Get trending stocks from Reddit, Finviz, and other sources
- get_trending_crypto: Get trending cryptocurrencies with price data and market caps  
- get_market_news: Get aggregated financial news from multiple sources
- get_reddit_trending_stocks: Get Reddit trending stocks with sentiment indicators
- get_coingecko_data: Get comprehensive cryptocurrency data from CoinGecko
- get_kraken_ticker: Get real-time crypto prices from Kraken exchange
- get_finviz_data: Get forex, futures, and insider trading data from Finviz
- get_yahoo_metrics: Get stock metrics and financial ratios from Yahoo Finance
- get_high_open_interest: Get options with high open interest
- get_comprehensive_market_analysis: Get combined analysis from all sources
- search_market_data: Search for specific symbols or keywords across all sources

RESEARCH APPROACH:
1. Start with comprehensive market analysis to understand overall market conditions
2. Identify trending assets and market themes
3. Analyze sentiment from news and social media
4. Deep dive into specific assets of interest
5. Look for correlations and market inefficiencies
6. Consider both technical and fundamental factors

ANALYSIS FRAMEWORK:
- Market Sentiment: What's the overall mood? Bull/bear indicators?
- Trending Assets: What's moving and why? Volume and momentum analysis
- News Impact: How are current events affecting markets?
- Social Sentiment: What's retail investor sentiment from Reddit/social media?
- Technical Signals: Key support/resistance, trend analysis
- Options Activity: What are institutions positioning for?
- Cross-Asset Analysis: How are different asset classes correlating?

Always provide data-driven insights with specific examples and metrics.
Today's date: {date}

Begin your research by getting a comprehensive market overview, then drill down into specific areas of interest.
"""

compress_trading_research_prompt = """You are an expert financial analyst tasked with compressing and synthesizing market research data.

Take the comprehensive market research data provided and create a concise, actionable summary that includes:

EXECUTIVE SUMMARY:
- Overall market conditions and sentiment
- Key themes and trends identified
- Most significant opportunities or risks

KEY FINDINGS:
- Top trending stocks with reasons for movement
- Cryptocurrency market dynamics
- Important news events and their market impact
- Social sentiment indicators
- Options market signals

ACTIONABLE INSIGHTS:
- Specific assets to watch
- Potential trading opportunities
- Risk factors to monitor
- Market timing considerations

SUPPORTING DATA:
- Key metrics and statistics
- Price movements and volume data
- Sentiment scores and indicators
- Technical levels to watch

Format your response as a professional research report suitable for investment decision-making.
Today's date: {date}
"""

# ===== AGENT NODES =====

async def trading_llm_call(state):
    """Analyze market conditions and decide on tool usage with Trading API MCP integration."""
    
    # Get available tools from MCP server
    client = get_mcp_client()
    mcp_tools = await client.get_tools()
    
    # Initialize model with tool binding
    model_with_tools = model.bind_tools(mcp_tools)
    
    # Build the message chain properly
    date = datetime.now().strftime("%Y-%m-%d")
    messages = [
        SystemMessage(content=trading_research_prompt.format(date=date))
    ] + state["researcher_messages"]
    
    # Estimate token usage
    message_text = " ".join([str(m.content) for m in messages])
    estimated_tokens = rate_limiter.count_tokens(message_text)
    
    # Wait if needed to respect rate limits
    await rate_limiter.wait_if_needed(estimated_tokens)
    
    print(f"🔍 Making LLM call with ~{estimated_tokens} tokens")
    
    try:
        # Process user input with trading-specific system prompt
        response = model_with_tools.invoke(messages)
        
        # Record successful request
        actual_tokens = estimated_tokens + rate_limiter.count_tokens(str(response.content))
        rate_limiter.record_request(actual_tokens)
        
        return {
            "researcher_messages": state["researcher_messages"] + [response]
        }
    except Exception as e:
        if "rate_limit" in str(e).lower():
            print(f"⚠️ Rate limit hit: {e}")
            print("   Waiting 60 seconds before retry...")
            await asyncio.sleep(60)
            return await trading_llm_call(state)  # Retry
        else:
            raise e

async def trading_tool_node(state):
    """Execute trading API tools using MCP tools."""
    
    tool_calls = state["researcher_messages"][-1].tool_calls
    
    async def execute_trading_tools():
        """Execute all trading tool calls."""
        # Get fresh tool references from MCP server
        client = get_mcp_client()
        mcp_tools = await client.get_tools()
        tools_by_name = {tool.name: tool for tool in mcp_tools}
        
        # Execute tool calls
        observations = []
        for tool_call in tool_calls:
            tool = tools_by_name[tool_call["name"]]
            # All MCP tools are async
            observation = await tool.ainvoke(tool_call["args"])
            observations.append(observation)
        
        # Format results as tool messages
        tool_outputs = [
            ToolMessage(
                content=str(observation),
                tool_call_id=tool_call["id"],
            )
            for observation, tool_call in zip(observations, tool_calls)
        ]
        
        return tool_outputs
    
    messages = await execute_trading_tools()
    return {"researcher_messages": state["researcher_messages"] + messages}

async def compress_trading_research(state) -> dict:
    """Compress trading research findings into a concise investment report."""
    date = datetime.now().strftime("%Y-%m-%d")
    system_message = compress_trading_research_prompt.format(date=date)
    
    # Limit the amount of data we compress to stay within token limits
    recent_messages = state.get("researcher_messages", [])[-10:]  # Only last 10 messages
    
    messages = [SystemMessage(content=system_message)] + recent_messages + [
        HumanMessage(content="Please compress this market research into a concise, actionable investment report.")
    ]
    
    # Estimate tokens for OpenAI model (different encoding but similar)
    message_text = " ".join([str(m.content) for m in messages])
    estimated_tokens = rate_limiter.count_tokens(message_text)
    
    print(f"📝 Compressing research with ~{estimated_tokens} tokens")
    
    # Add small delay between requests to spread load
    await asyncio.sleep(2)
    
    try:
        response = compress_model.invoke(messages)
        
        # Extract raw trading data from tool and AI messages
        raw_notes = [
            str(m.content) for m in filter_messages(
                recent_messages, 
                include_types=["tool", "ai"]
            )
        ]
        
        return {
            "compressed_research": str(response.content),
            "raw_notes": ["\n".join(raw_notes)]
        }
    except Exception as e:
        print(f"⚠️ Error in compression: {e}")
        # Fallback to simple summary if compression fails
        return {
            "compressed_research": "Research completed but compression failed due to rate limits. Raw data available in notes.",
            "raw_notes": [str(m.content) for m in recent_messages[-5:]]  # Last 5 messages only
        }

# ===== ROUTING LOGIC =====

def should_continue_trading(state) -> Literal["trading_tool_node", "compress_trading_research"]:
    """Determine whether to continue with tool execution or compress research."""
    messages = state["researcher_messages"]
    last_message = messages[-1]
    
    # Continue to tool execution if tools were called
    if last_message.tool_calls:
        return "trading_tool_node"
    # Otherwise, compress research findings
    return "compress_trading_research"

# ===== TRADING RESEARCH GRAPH =====

# Build the trading research agent workflow
trading_agent_builder = StateGraph(dict)  # Use dict for simplicity in this example

# Add nodes to the graph
trading_agent_builder.add_node("trading_llm_call", trading_llm_call)
trading_agent_builder.add_node("trading_tool_node", trading_tool_node)
trading_agent_builder.add_node("compress_trading_research", compress_trading_research)

# Add edges to connect nodes
trading_agent_builder.add_edge(START, "trading_llm_call")
trading_agent_builder.add_conditional_edges(
    "trading_llm_call",
    should_continue_trading,
    {
        "trading_tool_node": "trading_tool_node",
        "compress_trading_research": "compress_trading_research",
    },
)
trading_agent_builder.add_edge("trading_tool_node", "trading_llm_call")
trading_agent_builder.add_edge("compress_trading_research", END)

# Compile the trading research agent
trading_research_agent = trading_agent_builder.compile()

# ===== EXAMPLE USAGE =====

async def run_market_research_example():
    """Example of running market research with the trading API."""
    
    print("🔍 Starting comprehensive market research...")
    print("⚠️ Using rate-limited approach to respect Anthropic API limits")
    
    # Reduced research queries to minimize token usage
    research_queries = [
        "Get a comprehensive market overview using get_comprehensive_market_analysis. Focus on key trends in stocks and crypto.",
        # Removed additional queries to stay within rate limits
    ]
    
    for i, query in enumerate(research_queries, 1):
        print(f"\n📊 Research Query {i}/{len(research_queries)}: {query}")
        
        # Add delay between queries to spread token usage over time
        if i > 1:
            print("⏳ Waiting 30 seconds between queries to manage rate limits...")
            await asyncio.sleep(30)
        
        try:
            # Run the research agent
            result = await trading_research_agent.ainvoke({
                "researcher_messages": [HumanMessage(content=query)]
            })
            
            print("📋 Research Results:")
            print(result.get("compressed_research", "No results"))
            print("-" * 80)
            
        except Exception as e:
            print(f"❌ Error in research query {i}: {e}")
            if "rate_limit" in str(e).lower():
                print("   This appears to be a rate limit issue. Consider:")
                print("   1. Reducing the scope of your research queries")
                print("   2. Using shorter prompts")
                print("   3. Waiting longer between requests")
            print("-" * 80)
            continue

def run_sync_example():
    """Synchronous wrapper for the async example."""
    asyncio.run(run_market_research_example())

if __name__ == "__main__":
    print("Trading API MCP Research Agent Example")
    print("=" * 50)
    print()
    print("This example shows how to integrate the Trading API MCP server")
    print("with a research agent for comprehensive market analysis.")
    print()
    print("Make sure to:")
    print("1. Start your Rust trading API: cargo run")
    print("2. Have the MCP server dependencies installed: pip install -r requirements.txt")
    print("3. Set your OpenAI and Anthropic API keys")
    print()
    
    # Run the example
    run_sync_example()
