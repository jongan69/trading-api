use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use crate::types::TrendingItem;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CoinGeckoCoin {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub total_volume: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_7d_in_currency: Option<f64>,
    pub price_change_percentage_30d_in_currency: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub market_cap_change_percentage_24h: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_change_percentage: Option<f64>,
    pub ath_date: Option<String>,
    pub atl: Option<f64>,
    pub atl_change_percentage: Option<f64>,
    pub atl_date: Option<String>,
    pub roi: Option<Value>,
    pub last_updated: Option<String>,
    pub sparkline_in_7d: Option<SparklineData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SparklineData {
    pub price: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MarketOverview {
    pub total_market_cap: f64,
    pub total_volume: f64,
    pub bitcoin_dominance: f64,
    pub market_cap_percentage: HashMap<String, f64>,
    pub volume_percentage: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TrendingCoin {
    pub item: CoinGeckoTrendingItem,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CoinGeckoTrendingItem {
    pub id: String,
    pub coin_id: u32,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: u32,
    pub thumb: String,
    pub small: String,
    pub large: String,
    pub slug: String,
    pub price_btc: f64,
    pub score: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinGeckoResponse<T> {
    pub data: T,
}

pub struct CoinGeckoClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for CoinGeckoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinGeckoClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://api.coingecko.com/api/v3".to_string(),
        }
    }

    /// Fetch current market data for cryptocurrencies
    pub async fn get_market_data(
        &self,
        vs_currency: &str,
        order: &str,
        per_page: usize,
        page: usize,
        sparkline: bool,
        price_change_percentage: &str,
    ) -> Result<Vec<CoinGeckoCoin>, String> {
        let url = format!("{}/coins/markets", self.base_url);
        
        let params = [
            ("vs_currency", vs_currency),
            ("order", order),
            ("per_page", &per_page.to_string()),
            ("page", &page.to_string()),
            ("sparkline", &sparkline.to_string()),
            ("price_change_percentage", price_change_percentage),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("CoinGecko API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "CoinGecko API returned status {}",
                response.status()
            ));
        }

        let coins: Vec<CoinGeckoCoin> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse CoinGecko response: {e}"))?;

        Ok(coins)
    }

    /// Get top cryptocurrencies by market cap
    pub async fn get_top_coins(&self, limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
        self.get_market_data("usd", "market_cap_desc", limit, 1, false, "24h,7d,30d")
            .await
    }

    /// Get top gainers in the last 24 hours
    pub async fn get_top_gainers(&self, limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
        let coins = self
            .get_market_data("usd", "market_cap_desc", 100, 1, false, "24h")
            .await?;

        let mut gainers: Vec<CoinGeckoCoin> = coins
            .into_iter()
            .filter(|coin| coin.price_change_percentage_24h.is_some())
            .collect();

        gainers.sort_by(|a, b| {
            b.price_change_percentage_24h
                .unwrap_or(0.0)
                .partial_cmp(&a.price_change_percentage_24h.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(gainers.into_iter().take(limit).collect())
    }

    /// Get top losers in the last 24 hours
    pub async fn get_top_losers(&self, limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
        let coins = self
            .get_market_data("usd", "market_cap_desc", 100, 1, false, "24h")
            .await?;

        let mut losers: Vec<CoinGeckoCoin> = coins
            .into_iter()
            .filter(|coin| coin.price_change_percentage_24h.is_some())
            .collect();

        losers.sort_by(|a, b| {
            a.price_change_percentage_24h
                .unwrap_or(0.0)
                .partial_cmp(&b.price_change_percentage_24h.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(losers.into_iter().take(limit).collect())
    }

    /// Get trending coins
    pub async fn get_trending_coins(&self) -> Result<Vec<TrendingItem>, String> {
        let url = format!("{}/search/trending", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("CoinGecko trending request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "CoinGecko trending API returned status {}",
                response.status()
            ));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse trending response: {e}"))?;

        let coins = data
            .get("coins")
            .and_then(|c| c.as_array())
            .ok_or("No coins data found in response")?;

        let trending_coins: Vec<TrendingCoin> = serde_json::from_value(Value::Array(
            coins.clone(),
        ))
        .map_err(|e| format!("Failed to deserialize trending coins: {e}"))?;

        // Convert to unified TrendingItem structure
        let trending_items: Vec<TrendingItem> = trending_coins
            .into_iter()
            .map(|trending_coin| {
                TrendingItem {
                    id: trending_coin.item.id.clone(),
                    symbol: trending_coin.item.symbol.clone(),
                    name: trending_coin.item.name.clone(),
                    price: None, // CoinGecko trending doesn't provide current price
                    price_change_24h: None,
                    price_change_percentage_24h: None,
                    volume: None,
                    market_cap: None,
                    market_cap_rank: Some(trending_coin.item.market_cap_rank),
                    score: Some(trending_coin.item.score as f64),
                    source: "coingecko".to_string(),
                    image_url: Some(trending_coin.item.thumb.clone()),
                    last_updated: None,
                }
            })
            .collect();

        Ok(trending_items)
    }

    /// Get market overview statistics
    pub async fn get_market_overview(&self) -> Result<MarketOverview, String> {
        let coins = self
            .get_market_data("usd", "market_cap_desc", 100, 1, false, "24h")
            .await?;

        let total_market_cap: f64 = coins
            .iter()
            .filter_map(|coin| coin.market_cap)
            .sum();

        let total_volume: f64 = coins
            .iter()
            .filter_map(|coin| coin.total_volume)
            .sum();

        let bitcoin_data = coins
            .iter()
            .find(|coin| coin.symbol.to_lowercase() == "btc");

        let bitcoin_dominance = if let Some(btc) = bitcoin_data {
            if let Some(btc_market_cap) = btc.market_cap {
                (btc_market_cap / total_market_cap) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let mut market_cap_percentage = HashMap::new();
        let mut volume_percentage = HashMap::new();

        for coin in &coins {
            if let (Some(market_cap), Some(volume)) = (coin.market_cap, coin.total_volume) {
                if total_market_cap > 0.0 {
                    market_cap_percentage.insert(
                        coin.symbol.clone(),
                        (market_cap / total_market_cap) * 100.0,
                    );
                }
                if total_volume > 0.0 {
                    volume_percentage.insert(
                        coin.symbol.clone(),
                        (volume / total_volume) * 100.0,
                    );
                }
            }
        }

        Ok(MarketOverview {
            total_market_cap,
            total_volume,
            bitcoin_dominance,
            market_cap_percentage,
            volume_percentage,
        })
    }

    /// Get simple price data for specific coins
    pub async fn get_simple_price(
        &self,
        ids: &[String],
        vs_currencies: &[String],
        include_24hr_change: bool,
    ) -> Result<Value, String> {
        let url = format!("{}/simple/price", self.base_url);
        
        let ids_str = ids.join(",");
        let vs_currencies_str = vs_currencies.join(",");
        let include_change = if include_24hr_change { "true".to_string() } else { "false".to_string() };
        
        let params = vec![
            ("ids", &ids_str),
            ("vs_currencies", &vs_currencies_str),
            ("include_24hr_change", &include_change),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("CoinGecko simple price request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "CoinGecko simple price API returned status {}",
                response.status()
            ));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse simple price response: {e}"))?;

        Ok(data)
    }

    /// Get comprehensive market context (similar to the Python fetch_mcp_context function)
    pub async fn get_market_context(&self) -> Result<String, String> {
        let coins = self
            .get_market_data("usd", "market_cap_desc", 50, 1, false, "24h,7d,30d")
            .await?;

        let mut context_parts = Vec::new();
        context_parts.push("TOP 10 CRYPTOCURRENCIES BY MARKET CAP:".to_string());

        // Top 10 by market cap
        for (i, coin) in coins.iter().take(10).enumerate() {
            let price = coin.current_price.unwrap_or(0.0);
            let change_24h = coin.price_change_percentage_24h.unwrap_or(0.0);
            let change_7d = coin.price_change_percentage_7d_in_currency.unwrap_or(0.0);
            let change_30d = coin.price_change_percentage_30d_in_currency.unwrap_or(0.0);

            context_parts.push(format!(
                "{}. {} ({}): ${:.2} | 24h: {:+.2}% | 7d: {:+.2}% | 30d: {:+.2}%",
                i + 1,
                coin.name,
                coin.symbol.to_uppercase(),
                price,
                change_24h,
                change_7d,
                change_30d
            ));
        }

        // Top gainers
        let gainers = self.get_top_gainers(5).await?;
        context_parts.push("\nTOP 24H GAINERS:".to_string());
        for (i, coin) in gainers.iter().enumerate() {
            let price = coin.current_price.unwrap_or(0.0);
            let change = coin.price_change_percentage_24h.unwrap_or(0.0);
            context_parts.push(format!(
                "{}. {} ({}): +{:.2}% | ${:.4}",
                i + 1,
                coin.name,
                coin.symbol.to_uppercase(),
                change,
                price
            ));
        }

        // Top losers
        let losers = self.get_top_losers(5).await?;
        context_parts.push("\nTOP 24H LOSERS:".to_string());
        for (i, coin) in losers.iter().enumerate() {
            let price = coin.current_price.unwrap_or(0.0);
            let change = coin.price_change_percentage_24h.unwrap_or(0.0);
            context_parts.push(format!(
                "{}. {} ({}): {:.2}% | ${:.4}",
                i + 1,
                coin.name,
                coin.symbol.to_uppercase(),
                change,
                price
            ));
        }

        // Market overview
        let overview = self.get_market_overview().await?;
        context_parts.push("\nMARKET OVERVIEW:".to_string());
        context_parts.push(format!(
            "Total Market Cap: ${:.0}",
            overview.total_market_cap
        ));
        context_parts.push(format!(
            "24h Trading Volume: ${:.0}",
            overview.total_volume
        ));
        context_parts.push(format!(
            "Bitcoin Dominance: {:.2}%",
            overview.bitcoin_dominance
        ));

        Ok(context_parts.join("\n"))
    }

    /// Get trending cryptocurrencies
    pub async fn get_trending_cryptos(&self) -> Result<Vec<String>, String> {
        let trending = self.get_trending_coins().await?;
        Ok(trending
            .iter()
            .map(|coin| coin.symbol.to_uppercase())
            .collect())
    }
}

// Convenience functions for easy access
pub async fn get_top_coins(limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
    let client = CoinGeckoClient::new();
    client.get_top_coins(limit).await
}

pub async fn get_top_gainers(limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
    let client = CoinGeckoClient::new();
    client.get_top_gainers(limit).await
}

pub async fn get_top_losers(limit: usize) -> Result<Vec<CoinGeckoCoin>, String> {
    let client = CoinGeckoClient::new();
    client.get_top_losers(limit).await
}

pub async fn get_trending_coins() -> Result<Vec<TrendingItem>, String> {
    let client = CoinGeckoClient::new();
    client.get_trending_coins().await
}

pub async fn get_market_overview() -> Result<MarketOverview, String> {
    let client = CoinGeckoClient::new();
    client.get_market_overview().await
}

pub async fn get_market_context() -> Result<String, String> {
    let client = CoinGeckoClient::new();
    client.get_market_context().await
}

pub async fn get_trending_cryptos() -> Result<Vec<String>, String> {
    let client = CoinGeckoClient::new();
    client.get_trending_cryptos().await
}

pub async fn get_simple_price(
    ids: &[String],
    vs_currencies: &[String],
    include_24hr_change: bool,
) -> Result<Value, String> {
    let client = CoinGeckoClient::new();
    client.get_simple_price(ids, vs_currencies, include_24hr_change).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coin_gecko_client_creation() {
        let client = CoinGeckoClient::new();
        assert_eq!(client.base_url, "https://api.coingecko.com/api/v3");
    }

    #[tokio::test]
    async fn test_get_top_coins_function() {
        // Test the convenience function
        let result = get_top_coins(3).await;
        match result {
            Ok(coins) => {
                assert!(!coins.is_empty(), "Should return some coins");
                assert!(coins.len() <= 3, "Should respect the limit");
                
                // Check that coins have required fields
                for coin in coins {
                    assert!(!coin.id.is_empty(), "Coin should have an ID");
                    assert!(!coin.symbol.is_empty(), "Coin should have a symbol");
                    assert!(!coin.name.is_empty(), "Coin should have a name");
                }
            }
            Err(e) => {
                // Allow rate limiting errors
                assert!(e.contains("429") || e.contains("rate limit") || e.contains("Too Many Requests"), 
                        "Unexpected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_trending_coins_function() {
        let result = get_trending_coins().await;
        match result {
            Ok(trending) => {
                assert!(!trending.is_empty(), "Should return some trending coins");
                
                // Check that trending coins have required fields
                for coin in trending {
                    assert!(!coin.id.is_empty(), "Trending coin should have an ID");
                    assert!(!coin.symbol.is_empty(), "Trending coin should have a symbol");
                    assert!(!coin.name.is_empty(), "Trending coin should have a name");
                }
            }
            Err(e) => {
                // Allow rate limiting errors
                assert!(e.contains("429") || e.contains("rate limit") || e.contains("Too Many Requests"), 
                        "Unexpected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_market_overview_function() {
        let result = get_market_overview().await;
        match result {
            Ok(overview) => {
                assert!(overview.total_market_cap > 0.0, "Total market cap should be positive");
                assert!(overview.total_volume >= 0.0, "Total volume should be non-negative");
                assert!(overview.bitcoin_dominance >= 0.0, "Bitcoin dominance should be non-negative");
                assert!(overview.bitcoin_dominance <= 100.0, "Bitcoin dominance should be <= 100%");
            }
            Err(e) => {
                // Allow rate limiting errors
                assert!(e.contains("429") || e.contains("rate limit") || e.contains("Too Many Requests"), 
                        "Unexpected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_simple_price_function() {
        let ids = vec!["bitcoin".to_string(), "ethereum".to_string()];
        let vs_currencies = vec!["usd".to_string()];
        
        let result = get_simple_price(&ids, &vs_currencies, false).await;
        match result {
            Ok(data) => {
                // Check that we got a JSON object
                assert!(data.is_object(), "Should return a JSON object");
                
                // Check that it contains the requested coins
                if let Some(btc_data) = data.get("bitcoin") {
                    assert!(btc_data.is_object(), "Bitcoin data should be an object");
                }
                if let Some(eth_data) = data.get("ethereum") {
                    assert!(eth_data.is_object(), "Ethereum data should be an object");
                }
            }
            Err(e) => {
                // Allow rate limiting errors
                assert!(e.contains("429") || e.contains("rate limit") || e.contains("Too Many Requests"), 
                        "Unexpected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_trending_cryptos_function() {
        let result = get_trending_cryptos().await;
        match result {
            Ok(symbols) => {
                assert!(!symbols.is_empty(), "Should return some trending symbols");
                
                // Check that symbols are uppercase
                for symbol in symbols {
                    assert_eq!(symbol, symbol.to_uppercase(), "Symbols should be uppercase");
                    assert!(!symbol.is_empty(), "Symbol should not be empty");
                }
            }
            Err(e) => {
                // Allow rate limiting errors
                assert!(e.contains("429") || e.contains("rate limit") || e.contains("Too Many Requests"), 
                        "Unexpected error: {e}");
            }
        }
    }

    #[test]
    fn test_coin_gecko_coin_serialization() {
        let coin = CoinGeckoCoin {
            id: "bitcoin".to_string(),
            symbol: "btc".to_string(),
            name: "Bitcoin".to_string(),
            current_price: Some(50000.0),
            market_cap: Some(1000000000000.0),
            market_cap_rank: Some(1),
            total_volume: Some(50000000000.0),
            high_24h: Some(51000.0),
            low_24h: Some(49000.0),
            price_change_24h: Some(1000.0),
            price_change_percentage_24h: Some(2.0),
            price_change_percentage_7d_in_currency: Some(5.0),
            price_change_percentage_30d_in_currency: Some(10.0),
            market_cap_change_24h: Some(20000000000.0),
            market_cap_change_percentage_24h: Some(2.0),
            circulating_supply: Some(19000000.0),
            total_supply: Some(21000000.0),
            max_supply: Some(21000000.0),
            ath: Some(69000.0),
            ath_change_percentage: Some(-27.5),
            ath_date: Some("2021-11-10T14:24:11.849Z".to_string()),
            atl: Some(67.81),
            atl_change_percentage: Some(73680.0),
            atl_date: Some("2013-07-06T00:00:00.000Z".to_string()),
            roi: None,
            last_updated: Some("2024-01-01T00:00:00.000Z".to_string()),
            sparkline_in_7d: None,
        };

        // Test serialization
        let json = serde_json::to_string(&coin).unwrap();
        assert!(json.contains("bitcoin"), "Serialized JSON should contain coin ID");
        assert!(json.contains("Bitcoin"), "Serialized JSON should contain coin name");
        assert!(json.contains("btc"), "Serialized JSON should contain coin symbol");

        // Test deserialization
        let deserialized: CoinGeckoCoin = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "bitcoin");
        assert_eq!(deserialized.symbol, "btc");
        assert_eq!(deserialized.name, "Bitcoin");
        assert_eq!(deserialized.current_price, Some(50000.0));
    }

    #[test]
    fn test_market_overview_serialization() {
        let overview = MarketOverview {
            total_market_cap: 2000000000000.0,
            total_volume: 100000000000.0,
            bitcoin_dominance: 45.5,
            market_cap_percentage: {
                let mut map = std::collections::HashMap::new();
                map.insert("btc".to_string(), 45.5);
                map.insert("eth".to_string(), 18.2);
                map
            },
            volume_percentage: {
                let mut map = std::collections::HashMap::new();
                map.insert("btc".to_string(), 35.0);
                map.insert("eth".to_string(), 25.0);
                map
            },
        };

        // Test serialization
        let json = serde_json::to_string(&overview).unwrap();
        assert!(json.contains("2000000000000"), "Serialized JSON should contain total market cap");
        assert!(json.contains("45.5"), "Serialized JSON should contain bitcoin dominance");

        // Test deserialization
        let deserialized: MarketOverview = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_market_cap, 2000000000000.0);
        assert_eq!(deserialized.bitcoin_dominance, 45.5);
        assert_eq!(deserialized.market_cap_percentage.get("btc"), Some(&45.5));
    }
}
