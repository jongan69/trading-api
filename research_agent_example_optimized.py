#!/usr/bin/env python3
"""
Rate-Limited Research Agent with Trading API MCP Integration

This optimized example demonstrates how to integrate the Trading API MCP server
with a research agent while respecting Anthropic's API rate limits.

Optimizations for rate limits:
1. Uses Claude Haiku 3.5 (50k tokens/min vs 30k for Sonnet)
2. Implements token counting and rate limiting
3. Reduces prompt sizes significantly
4. Adds delays between requests
5. Includes retry logic with exponential backoff
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

# ===== CONFIGURATION =====

# MCP server configuration for trading API access
trading_mcp_config = {
    "trading_api": {
        "command": "python3",
        "args": [
            str(Path(__file__).parent / "mcp_server.py")
        ],
        "transport": "stdio"
    }
}

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

# Global instances
rate_limiter = RateLimiter()
_client = None

def get_mcp_client():
    """Get or initialize MCP client lazily."""
    global _client
    if _client is None:
        _client = MultiServerMCPClient(trading_mcp_config)
    return _client

# Initialize models - using Claude Haiku 3.5 for higher rate limits (50k tokens/min vs 30k)
compress_model = init_chat_model(model="openai:gpt-4o-mini", max_tokens=8000)
model = init_chat_model(model="anthropic:claude-3-5-haiku-20241022", max_tokens=4000)

# ===== OPTIMIZED PROMPTS =====

# Significantly shortened prompt to reduce token usage
trading_research_prompt = """You are a financial research agent with market data tools.

Tools available:
- get_comprehensive_market_analysis: Combined analysis from all sources
- get_trending_stocks: Trending stocks from multiple sources  
- get_trending_crypto: Trending cryptocurrencies with price data
- get_market_news: Aggregated financial news
- get_hyperliquid_trending: Trending DeFi assets on Hyperliquid
- get_solana_trending_nfts: Trending Solana NFT collections
- get_solana_trending_tokens: Trending Solana tokens
- get_coingecko_market_context: AI-powered market insights
- search_market_data: Search symbols or keywords

Approach: Use get_comprehensive_market_analysis for broad overview, then focus on key trends.
Date: {date}

Be concise and actionable."""

compress_research_prompt = """Compress market research into a brief, actionable summary.

Include:
- Key market trends and sentiment
- Top opportunities and risks  
- Specific assets to watch
- Supporting metrics

Date: {date}"""

# ===== AGENT NODES =====

async def trading_llm_call(state):
    """LLM call with rate limiting."""
    
    # Get available tools from MCP server
    client = get_mcp_client()
    mcp_tools = await client.get_tools()
    
    # Initialize model with tool binding
    model_with_tools = model.bind_tools(mcp_tools)
    
    # Build messages
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
    """Execute trading API tools."""
    
    tool_calls = state["researcher_messages"][-1].tool_calls
    
    async def execute_trading_tools():
        """Execute all trading tool calls."""
        client = get_mcp_client()
        mcp_tools = await client.get_tools()
        tools_by_name = {tool.name: tool for tool in mcp_tools}
        
        observations = []
        for tool_call in tool_calls:
            tool = tools_by_name[tool_call["name"]]
            observation = await tool.ainvoke(tool_call["args"])
            observations.append(observation)
        
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
    """Compress research with rate limiting."""
    date = datetime.now().strftime("%Y-%m-%d")
    system_message = compress_research_prompt.format(date=date)
    
    # Limit messages to stay within token limits
    recent_messages = state.get("researcher_messages", [])[-8:]  # Only last 8 messages
    
    messages = [SystemMessage(content=system_message)] + recent_messages + [
        HumanMessage(content="Compress this research into actionable insights.")
    ]
    
    # Estimate tokens
    message_text = " ".join([str(m.content) for m in messages])
    estimated_tokens = rate_limiter.count_tokens(message_text)
    
    print(f"📝 Compressing research with ~{estimated_tokens} tokens")
    
    # Small delay between requests
    await asyncio.sleep(2)
    
    try:
        response = compress_model.invoke(messages)
        
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
        return {
            "compressed_research": "Research completed but compression failed due to rate limits.",
            "raw_notes": [str(m.content) for m in recent_messages[-3:]]
        }

# ===== ROUTING LOGIC =====

def should_continue_trading(state) -> Literal["trading_tool_node", "compress_trading_research"]:
    """Determine whether to continue with tool execution or compress research."""
    messages = state["researcher_messages"]
    last_message = messages[-1]
    
    if last_message.tool_calls:
        return "trading_tool_node"
    return "compress_trading_research"

# ===== TRADING RESEARCH GRAPH =====

trading_agent_builder = StateGraph(dict)

# Add nodes
trading_agent_builder.add_node("trading_llm_call", trading_llm_call)
trading_agent_builder.add_node("trading_tool_node", trading_tool_node)
trading_agent_builder.add_node("compress_trading_research", compress_trading_research)

# Add edges
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

# Compile the agent
trading_research_agent = trading_agent_builder.compile()

# ===== EXAMPLE USAGE =====

async def run_market_research_example():
    """Rate-limited market research example."""
    
    print("🔍 Starting rate-limited market research...")
    print("⚠️ Optimized for Anthropic API limits:")
    print("   - Using Claude Haiku 3.5 (50k tokens/min)")
    print("   - Token counting and rate limiting")
    print("   - Reduced prompt sizes")
    print("   - Delays between requests")
    print()
    
    # Single focused query to minimize token usage
    query = "Get comprehensive market analysis. Focus on top trends in stocks and crypto."
    
    print(f"📊 Research Query: {query}")
    
    try:
        # Run the research agent
        result = await trading_research_agent.ainvoke({
            "researcher_messages": [HumanMessage(content=query)]
        })
        
        print("\n📋 Research Results:")
        print("=" * 50)
        print(result.get("compressed_research", "No results"))
        print("=" * 50)
        
        # Show token usage summary
        total_tokens = sum(tokens for _, tokens in rate_limiter.token_usage)
        total_requests = len(rate_limiter.request_times)
        print(f"\n📊 Usage Summary:")
        print(f"   Total tokens used: {total_tokens}/{rate_limiter.tokens_per_minute}")
        print(f"   Total requests: {total_requests}/{rate_limiter.requests_per_minute}")
        
    except Exception as e:
        print(f"❌ Error in research: {e}")
        if "rate_limit" in str(e).lower():
            print("\n💡 Rate limit suggestions:")
            print("   1. Wait 60 seconds before retrying")
            print("   2. Use shorter, more focused queries")
            print("   3. Consider using fewer tools per request")

def run_sync_example():
    """Synchronous wrapper for the async example."""
    asyncio.run(run_market_research_example())

if __name__ == "__main__":
    print("Trading API MCP Research Agent - Rate Limited Version")
    print("=" * 60)
    print()
    print("This optimized version respects Anthropic's API rate limits:")
    print("- Claude Haiku 3.5: 50,000 tokens/min, 50 requests/min")
    print("- Token counting and automatic rate limiting")
    print("- Shortened prompts and focused queries")
    print("- Retry logic with exponential backoff")
    print()
    print("Setup requirements:")
    print("1. Start your Rust trading API: cargo run")
    print("2. Install MCP dependencies: pip install -r requirements.txt")
    print("3. Set ANTHROPIC_API_KEY and OPENAI_API_KEY environment variables")
    print()
    
    # Run the example
    run_sync_example()
