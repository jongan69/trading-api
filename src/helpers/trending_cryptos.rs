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

    let kraken_cryptos = get_trending_cryptos_kraken(limit).await;
    for crypto in kraken_cryptos {
        all_cryptos.insert(crypto);
    }

    let coingecko_cryptos = get_trending_cryptos_coingecko(limit).await;
    for crypto in coingecko_cryptos {
        all_cryptos.insert(crypto);
    }

    let mut result: Vec<String> = all_cryptos.into_iter().collect();
    result.truncate(limit);
    result
}