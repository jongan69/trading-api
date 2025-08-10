use crate::sources::kraken_data::get_trending_crypto_pairs;
use crate::sources::coingecko_data::get_trending_cryptos;

/// Get trending cryptocurrencies from Kraken by volume
pub async fn get_trending_cryptos_kraken(limit: usize) -> Vec<String> {
    match get_trending_crypto_pairs(limit).await {
        Ok(items) => items.into_iter().map(|item| item.symbol).collect(),
        Err(e) => {
            tracing::error!("Failed to get trending cryptos from Kraken: {}", e);
            vec![]
        }
    }
}

/// Get trending cryptos from finviz
pub async fn get_trending_cryptos_finviz() -> Vec<String> {
    // TODO: Implement Finviz crypto trending
    vec![]
}

/// Get trending cryptos from yahoo
pub async fn get_trending_cryptos_yahoo() -> Vec<String> {
    // TODO: Implement Yahoo crypto trending
    vec![]
}

/// Get trending cryptos from CoinGecko
pub async fn get_trending_cryptos_coingecko(limit: usize) -> Vec<String> {
    match get_trending_cryptos().await {
        Ok(cryptos) => {
            let mut result = cryptos;
            result.truncate(limit);
            result
        }
        Err(e) => {
            tracing::error!("Failed to get trending cryptos from CoinGecko: {}", e);
            vec![]
        }
    }
}

/// Get aggregated trending cryptocurrencies from multiple sources
pub async fn get_trending_cryptos_aggregated(limit: usize) -> Vec<String> {
    let mut all_cryptos = std::collections::HashSet::new();
    
    // Get from Kraken
    let kraken_cryptos = get_trending_cryptos_kraken(limit).await;
    for crypto in kraken_cryptos {
        all_cryptos.insert(crypto);
    }
    
    // Get from Finviz
    let finviz_cryptos = get_trending_cryptos_finviz().await;
    for crypto in finviz_cryptos {
        all_cryptos.insert(crypto);
    }
    
    // Get from Yahoo
    let yahoo_cryptos = get_trending_cryptos_yahoo().await;
    for crypto in yahoo_cryptos {
        all_cryptos.insert(crypto);
    }
    
    // Get from CoinGecko
    let coingecko_cryptos = get_trending_cryptos_coingecko(limit).await;
    for crypto in coingecko_cryptos {
        all_cryptos.insert(crypto);
    }
    
    // Convert back to vector and limit
    let mut result: Vec<String> = all_cryptos.into_iter().collect();
    result.truncate(limit);
    result
}