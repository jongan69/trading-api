use trading_api::sources::coingecko_data::{
    CoinGeckoClient, get_top_coins, get_top_gainers, get_top_losers, 
    get_trending_coins, get_market_overview, get_market_context,
    get_simple_price
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CoinGecko Data Source Example\n");

    // Example 1: Get top cryptocurrencies by market cap
    println!("1. Top 10 Cryptocurrencies by Market Cap:");
    match get_top_coins(10).await {
        Ok(coins) => {
            for (i, coin) in coins.iter().enumerate() {
                println!("   {}. {} ({}): ${:.2} | 24h: {:+.2}%", 
                    i + 1, 
                    coin.name, 
                    coin.symbol.to_uppercase(),
                    coin.current_price.unwrap_or(0.0),
                    coin.price_change_percentage_24h.unwrap_or(0.0)
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 2: Get top gainers
    println!("2. Top 5 Gainers (24h):");
    match get_top_gainers(5).await {
        Ok(coins) => {
            for (i, coin) in coins.iter().enumerate() {
                println!("   {}. {} ({}): +{:.2}% | ${:.4}", 
                    i + 1, 
                    coin.name, 
                    coin.symbol.to_uppercase(),
                    coin.price_change_percentage_24h.unwrap_or(0.0),
                    coin.current_price.unwrap_or(0.0)
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 3: Get top losers
    println!("3. Top 5 Losers (24h):");
    match get_top_losers(5).await {
        Ok(coins) => {
            for (i, coin) in coins.iter().enumerate() {
                println!("   {}. {} ({}): {:.2}% | ${:.4}", 
                    i + 1, 
                    coin.name, 
                    coin.symbol.to_uppercase(),
                    coin.price_change_percentage_24h.unwrap_or(0.0),
                    coin.current_price.unwrap_or(0.0)
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 4: Get trending coins
    println!("4. Trending Cryptocurrencies:");
    match get_trending_coins().await {
        Ok(trending) => {
            for (i, coin) in trending.iter().enumerate() {
                println!("   {}. {} ({}): Score {}", 
                    i + 1, 
                    coin.item.name, 
                    coin.item.symbol.to_uppercase(),
                    coin.item.score
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 5: Get market overview
    println!("5. Market Overview:");
    match get_market_overview().await {
        Ok(overview) => {
            println!("   Total Market Cap: ${:.0}", overview.total_market_cap);
            println!("   24h Trading Volume: ${:.0}", overview.total_volume);
            println!("   Bitcoin Dominance: {:.2}%", overview.bitcoin_dominance);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 6: Get market context (for AI analysis)
    println!("6. Market Context (for AI Analysis):");
    match get_market_context().await {
        Ok(context) => {
            println!("   {}", context.lines().next().unwrap_or("No context available"));
            println!("   ... (truncated for display)");
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 7: Get simple price data
    println!("7. Simple Price Data:");
    let ids = vec!["bitcoin".to_string(), "ethereum".to_string(), "cardano".to_string()];
    let vs_currencies = vec!["usd".to_string(), "eur".to_string()];
    
    match get_simple_price(&ids, &vs_currencies, true).await {
        Ok(data) => {
            println!("   {}", serde_json::to_string_pretty(&data).unwrap());
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 8: Using the client directly
    println!("8. Using CoinGeckoClient directly:");
    let client = CoinGeckoClient::new();
    
    match client.get_top_coins(3).await {
        Ok(coins) => {
            println!("   Top 3 coins using client:");
            for coin in coins {
                println!("   - {} ({}): ${:.2}", 
                    coin.name, 
                    coin.symbol.to_uppercase(),
                    coin.current_price.unwrap_or(0.0)
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n✅ CoinGecko example completed!");
    Ok(())
}
