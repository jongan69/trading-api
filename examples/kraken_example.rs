use trading_api::sources::kraken_data::{
    KrakenDataSource, get_trending_crypto_pairs, get_market_summary
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Kraken Data Source Example ===\n");

    // Initialize the Kraken data source
    println!("✅ Kraken data source initialized successfully\n");

    // Get system status
    println!("📊 System Status:");
    let status = tokio::task::spawn_blocking(|| {
        let data_source = KrakenDataSource::new()?;
        data_source.get_system_status()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    println!("{:?}\n", status);

    // Get server time
    println!("🕐 Server Time:");
    let time = tokio::task::spawn_blocking(|| {
        let data_source = KrakenDataSource::new()?;
        data_source.get_server_time()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    println!("{:?}\n", time);

    // Get ticker information for popular pairs
    println!("📈 Ticker Information:");
    let pairs = vec!["XXBTZUSD".to_string(), "XETHZUSD".to_string(), "LOCKINUSD".to_string()];
    let tickers = tokio::task::spawn_blocking(move || {
        let data_source = KrakenDataSource::new()?;
        data_source.get_tickers(pairs)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    
    for ticker in &tickers {
        println!("{}: ${:.2} (24h: {:.2}%)", 
            ticker.pair, 
            ticker.price, 
            ticker.change_pct_24h
        );
    }
    println!();

    // Get order book for Bitcoin
    println!("📚 Order Book (XBT/USD):");
    let order_book = tokio::task::spawn_blocking(|| {
        let data_source = KrakenDataSource::new()?;
        data_source.get_order_book("XBT/USD", 5)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    println!("Bids:");
    for (price, volume) in &order_book.bids[..std::cmp::min(5, order_book.bids.len())] {
        println!("  ${:.2} - {:.4}", price, volume);
    }
    println!("Asks:");
    for (price, volume) in &order_book.asks[..std::cmp::min(5, order_book.asks.len())] {
        println!("  ${:.2} - {:.4}", price, volume);
    }
    println!();

    // Get trending crypto pairs
    println!("🔥 Trending Crypto Pairs (by volume):");
    let trending = get_trending_crypto_pairs(10).await?;
    for (i, pair) in trending.iter().enumerate() {
        println!("  {}. {}", i + 1, pair);
    }
    println!();

    // Get market summary for Ethereum
    println!("📋 Market Summary (ETH/USD):");
    let summary = get_market_summary("ETH/USD").await?;
    println!("{:?}\n", summary);

    // Get recent trades for Bitcoin
    println!("💱 Recent Trades (XBT/USD):");
    let trades = tokio::task::spawn_blocking(|| {
        let data_source = KrakenDataSource::new()?;
        data_source.get_recent_trades("XBT/USD", None)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    
    // Format trades output nicely
    if let Some(trades_obj) = trades.as_object() {
        if let Some(trades_array) = trades_obj.get("trades") {
            if let Some(trades_vec) = trades_array.as_array() {
                println!("Found {} recent trades:", trades_vec.len());
                for (i, trade) in trades_vec.iter().take(5).enumerate() {
                    println!("  {}. {:?}", i + 1, trade);
                }
            } else {
                println!("No trades array found");
            }
        } else {
            println!("No trades data found");
        }
    } else {
        println!("{:?}", trades);
    }
    println!();

    // Get OHLC data for Bitcoin
    println!("📊 OHLC Data (XBT/USD):");
    let ohlc = tokio::task::spawn_blocking(|| {
        let data_source = KrakenDataSource::new()?;
        data_source.get_ohlc("XBT/USD", Some(1), None)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    
    // Format OHLC output nicely
    if let Some(ohlc_obj) = ohlc.as_object() {
        if let Some(ohlc_array) = ohlc_obj.get("ohlc") {
            if let Some(ohlc_vec) = ohlc_array.as_array() {
                println!("Found {} OHLC candles:", ohlc_vec.len());
                for (i, candle) in ohlc_vec.iter().take(3).enumerate() {
                    println!("  {}. {:?}", i + 1, candle);
                }
            } else {
                println!("No OHLC array found");
            }
        } else {
            println!("No OHLC data found");
        }
    } else {
        println!("{:?}", ohlc);
    }
    println!();

    println!("✅ Example completed successfully!");
    Ok(())
}
