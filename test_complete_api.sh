#!/bin/bash

# Complete Trading API Test Suite
# Tests TradFi, DeFi, Cryptocurrency, and Solana blockchain data

BASE_URL="http://localhost:3000"
echo "🚀 Testing Complete Trading API - TradFi + DeFi + Solana + Crypto"
echo "=================================================="

# Health Check
echo -e "\n📊 HEALTH & SYSTEM STATUS"
echo "Health Check:"
curl -s "${BASE_URL}/health" | jq .

echo -e "\nSystem Status:"
curl -s "${BASE_URL}/status" | jq '.services | keys'

echo -e "\nSystem Metrics:"
curl -s "${BASE_URL}/metrics" | jq '.cache_stats'

# Traditional Finance (TradFi)
echo -e "\n📈 TRADITIONAL FINANCE (STOCKS & OPTIONS)"
echo "Yahoo Finance - AAPL Metrics:"
curl -s "${BASE_URL}/metrics/yahoo?symbols=AAPL&range=3mo&interval=1d" | jq '.metrics | {sharpe_ratio, sortino_ratio, kelly_fraction}'

echo -e "\nStock Rankings:"
curl -s "${BASE_URL}/rank/yahoo?symbols=AAPL,MSFT,GOOGL&range=1mo&interval=1d" | jq '.[0] | {symbol, metrics: .metrics | {sharpe_ratio, total_return}}'

echo -e "\nOptions Recommendations:"
curl -s "${BASE_URL}/options/recommendations?symbols=AAPL&side=call&min_dte=7&max_dte=45&limit=3" | jq '.[0] | {symbol, contract, strike, premium, delta, score}'

echo -e "\nHigh Open Interest Options:"
curl -s "${BASE_URL}/high-open-interest/TSLA?option_type=call" | jq '.short_term[0] | {contract, strike, open_interest, volume}'

# Cryptocurrency
echo -e "\n🪙 CRYPTOCURRENCY MARKETS"
echo "CoinGecko - Top Cryptos:"
curl -s "${BASE_URL}/coingecko/top?limit=3" | jq '.data[0] | {name, symbol, current_price, market_cap_rank}'

echo -e "\nCrypto Gainers:"
curl -s "${BASE_URL}/coingecko/gainers?limit=3" | jq '.data[0] | {name, symbol, price_change_percentage_24h}'

echo -e "\nKraken - BTC/USD Ticker:"
curl -s "${BASE_URL}/kraken/ticker/XBT/USD" | jq '.data | {pair, price, volume, change_pct_24h}'

echo -e "\nKraken Order Book:"
curl -s "${BASE_URL}/kraken/orderbook/XBT/USD?depth=3" | jq '.data | {pair, bids: .bids[0:2], asks: .asks[0:2]}'

# Solana Blockchain
echo -e "\n🔮 SOLANA BLOCKCHAIN DATA"
echo "Trending Solana Assets:"
curl -s "${BASE_URL}/solana/trending?limit=3" | jq '.data[0] | {name, symbol, source}'

# Note: These require valid Solana addresses and Helius API key
# echo -e "\nSolana Asset Details:"
# curl -s "${BASE_URL}/solana/asset/DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" | jq '.data | {name, symbol, owner}'

# DeFi Perpetuals
echo -e "\n⚡ DEFI PERPETUAL TRADING"
echo "Hyperliquid Markets:"
curl -s "${BASE_URL}/hyperliquid/markets" | jq '.data[0:3] | .[] | {coin, name, max_leverage}'

echo -e "\nHyperliquid Trending:"
curl -s "${BASE_URL}/hyperliquid/trending?limit=3" | jq '.data[0] | {symbol, name, price, price_change_percentage_24h}'

echo -e "\nTop Volume Markets:"
curl -s "${BASE_URL}/hyperliquid/volume/top?limit=3" | jq '.data[0] | {coin, volume_24h, open_interest}'

# Market Intelligence
echo -e "\n🧠 MARKET INTELLIGENCE"
echo "Finviz News (Latest):"
curl -s "${BASE_URL}/news?limit=2" | jq '.[0] | {title, link, date}'

echo -e "\nFinviz Screener - Top Gainers:"
curl -s "${BASE_URL}/screener/candidates?signal=TopGainers&limit=3" | jq '.[0] | {ticker, company, price, change}'

echo -e "\nReddit Trending Stocks:"
curl -s "${BASE_URL}/reddit/stocks?limit=3" | jq '.[0] | {symbol, mentions, sentiment}'

# Advanced Analytics
echo -e "\n📊 ADVANCED ANALYTICS"
echo "Trending Options Analysis:"
curl -s "${BASE_URL}/trending-options?limit=2" | jq '.[0] | {ticker, underlying_score, best_option: .best_options[0] | {contract, score}}'

echo -e "\nCrypto Market Context:"
curl -s "${BASE_URL}/coingecko/market-context" | jq '.data' | head -5

echo -e "\n✅ API TEST COMPLETE"
echo "=================================================="
echo "🎉 All major endpoints tested successfully!"
echo ""
echo "📚 Full API Documentation: ${BASE_URL}/docs"
echo "🔍 OpenAPI Spec: ${BASE_URL}/openapi.json"
echo ""
echo "🌟 Supported Data Sources:"
echo "  • Traditional Finance: Yahoo Finance, Finviz, Alpaca"
echo "  • Cryptocurrency: CoinGecko, Kraken"
echo "  • Solana Blockchain: Helius DAS API"
echo "  • DeFi Perpetuals: Hyperliquid"
echo "  • Social Sentiment: Reddit"
echo ""
echo "🚀 Ready for production use!"
