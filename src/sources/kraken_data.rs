use krakenrs::{KrakenRestAPI, KrakenRestConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct KrakenTicker {
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub change_24h: f64,
    pub change_pct_24h: f64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KrakenOrderBook {
    pub pair: String,
    pub bids: Vec<(f64, f64)>, // (price, volume)
    pub asks: Vec<(f64, f64)>, // (price, volume)
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KrakenAsset {
    pub name: String,
    pub altname: String,
    pub aclass: String,
    pub decimals: u32,
    pub display_decimals: u32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KrakenAssetPair {
    pub name: String,
    pub altname: String,
    pub wsname: Option<String>,
    pub aclass_base: String,
    pub base: String,
    pub aclass_quote: String,
    pub quote: String,
    pub pair_decimals: u32,
    pub lot_decimals: u32,
    pub lot_multiplier: u32,
    pub ordermin: Option<String>,
}

// Kraken API response structures
#[derive(Debug, Deserialize)]
struct KrakenOrderBookResponse {
    error: Vec<String>,
    result: Option<OrderBookResult>,
}

#[derive(Debug, Deserialize)]
struct OrderBookResult {
    #[serde(rename = "XXBTZUSD")]
    xbtusd: Option<OrderBookData>,
    #[serde(rename = "XETHZUSD")]
    ethusd: Option<OrderBookData>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, OrderBookData>,
    #[serde(default)]
    last: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct OrderBookData {
    asks: Vec<Vec<serde_json::Value>>,
    bids: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct KrakenTradesResponse {
    error: Vec<String>,
    result: Option<TradesResult>,
}

#[derive(Debug, Deserialize)]
struct TradesResult {
    #[serde(rename = "XXBTZUSD")]
    xbtusd: Option<Vec<serde_json::Value>>,
    #[serde(rename = "XETHZUSD")]
    ethusd: Option<Vec<serde_json::Value>>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, Vec<serde_json::Value>>,
    last: String,
}

#[derive(Debug, Deserialize)]
struct KrakenOHLCResponse {
    error: Vec<String>,
    result: Option<OHLCResult>,
}

#[derive(Debug, Deserialize)]
struct OHLCResult {
    #[serde(rename = "XXBTZUSD")]
    xbtusd: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(rename = "XETHZUSD")]
    ethusd: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, Vec<Vec<serde_json::Value>>>,
    last: u64,
}

pub struct KrakenDataSource {
    http_client: reqwest::Client,
}

impl KrakenDataSource {
    pub async fn new_async() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Create the data source in a blocking task to avoid runtime issues
        tokio::task::spawn_blocking(|| {
            let http_client = reqwest::Client::new();
            
            Ok::<Self, Box<dyn std::error::Error + Send + Sync>>(Self { http_client })
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
    }



    /// Get ticker information for specified pairs
    pub async fn get_tickers_async(&self, pairs: Vec<String>) -> Result<Vec<KrakenTicker>, Box<dyn std::error::Error + Send + Sync>> {
        // If no pairs specified, use default popular pairs
        let pairs_to_fetch = if pairs.is_empty() {
            vec![
                "XXBTZUSD".to_string(),
                "XETHZUSD".to_string(),
                "XXBTZEUR".to_string(),
                "XETHZEUR".to_string(),
                "ADAUSD".to_string(),
                "DOTUSD".to_string(),
                "LINKUSD".to_string(),
                "LTCUSD".to_string(),
            ]
        } else {
            pairs
        };
        
        // Convert pair formats to ensure compatibility with krakenrs library
        let converted_pairs: Vec<String> = pairs_to_fetch.iter().map(|pair| {
            // Handle common pair format conversions
            match pair.as_str() {
                "XXBTZUSD" => "XBTUSD".to_string(),
                "XETHZUSD" => "ETHUSD".to_string(),
                "XXBTZEUR" => "XBTEUR".to_string(),
                "XETHZEUR" => "ETHEUR".to_string(),
                _ => pair.clone(),
            }
        }).collect();
        
        // Use blocking task to avoid runtime issues
        let ticker_response = tokio::task::spawn_blocking(move || {
            let config = KrakenRestConfig::default();
            let rest_api = KrakenRestAPI::try_from(config)
                .map_err(|e| format!("Failed to create Kraken API: {e}"))?;
            rest_api.ticker(converted_pairs)
                .map_err(|e| format!("Failed to get ticker data: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        let mut tickers = Vec::new();

        for (pair_name, ticker_data) in ticker_response {
            // Extract data from the ticker response
            let current_price = ticker_data.c.first()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            
            let volume_24h = ticker_data.a.get(1)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            
            let high_24h = ticker_data.a.get(1)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            
            let low_24h = ticker_data.a.get(1)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            
            let open_price = ticker_data.a.first().unwrap_or(&"0".to_string()).parse::<f64>().unwrap_or(0.0);
            
            let change_24h = current_price - open_price;
            let change_pct_24h = if open_price > 0.0 {
                (change_24h / open_price) * 100.0
            } else {
                0.0
            };

            let ticker = KrakenTicker {
                pair: pair_name,
                price: current_price,
                volume: volume_24h,
                high_24h,
                low_24h,
                change_24h,
                change_pct_24h,
            };
            tickers.push(ticker);
        }

        Ok(tickers)
    }

    /// Get order book for a specific pair
    pub async fn get_order_book(&self, pair: &str, depth: u32) -> Result<KrakenOrderBook, Box<dyn std::error::Error + Send + Sync>> {
        // Convert pair format from "XBT/USD" to "XBTUSD" for Kraken API
        let kraken_pair = pair.replace("/", "");
        
        // Use async HTTP request directly
        let url = format!("https://api.kraken.com/0/public/Depth?pair={kraken_pair}&count={depth}");
        let response = self.http_client.get(&url).send().await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        
        let order_book_response: KrakenOrderBookResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;
        
        if !order_book_response.error.is_empty() {
            return Err(format!("Kraken API error: {:?}", order_book_response.error).into());
        }
        
        let result = order_book_response.result
            .ok_or::<Box<dyn std::error::Error + Send + Sync>>("No result in response".into())?;
        
        // Find the order book data for the requested pair
        let order_book_data = if kraken_pair == "XBTUSD" || kraken_pair == "XXBTZUSD" {
            result.xbtusd
        } else if kraken_pair == "ETHUSD" || kraken_pair == "XETHZUSD" {
            result.ethusd
        } else {
            result.other.get(&kraken_pair).cloned()
        }.ok_or::<Box<dyn std::error::Error + Send + Sync>>(
            format!("No order book data for pair: {pair}").into()
        )?;
        
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        // Parse bids
        for bid in order_book_data.bids {
            if bid.len() >= 2 {
                let price = bid[0].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let volume = bid[1].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                bids.push((price, volume));
            }
        }
        
        // Parse asks
        for ask in order_book_data.asks {
            if ask.len() >= 2 {
                let price = ask[0].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let volume = ask[1].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                asks.push((price, volume));
            }
        }
        
        // Sort bids (descending) and asks (ascending)
        bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        let timestamp = result.last;
        
        Ok(KrakenOrderBook {
            pair: pair.to_string(),
            bids,
            asks,
            timestamp,
        })
    }

    /// Get all available asset pairs
    pub async fn get_asset_pairs_async(&self) -> Result<Vec<KrakenAssetPair>, Box<dyn std::error::Error + Send + Sync>> {
        // Use blocking task to avoid runtime issues
        let pairs_response = tokio::task::spawn_blocking(move || {
            let config = KrakenRestConfig::default();
            let rest_api = KrakenRestAPI::try_from(config)
                .map_err(|e| format!("Failed to create Kraken API: {e}"))?;
            rest_api.asset_pairs(vec![])
                .map_err(|e| format!("Failed to get asset pairs: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        
        let mut asset_pairs = Vec::new();

        for (pair_name, pair_data) in pairs_response {
            let asset_pair = KrakenAssetPair {
                name: pair_name,
                altname: pair_data.alt_name.unwrap_or_default(),
                wsname: pair_data.wsname,
                aclass_base: pair_data.aclass_base,
                base: pair_data.base,
                aclass_quote: pair_data.aclass_quote,
                quote: pair_data.quote,
                pair_decimals: pair_data.pair_decimals as u32,
                lot_decimals: pair_data.lot_decimals as u32,
                lot_multiplier: pair_data.lot_multiplier as u32,
                ordermin: pair_data.ordermin.map(|d| d.to_string()),
            };
            asset_pairs.push(asset_pair);
        }

        Ok(asset_pairs)
    }


    /// Get asset information
    pub async fn get_assets(&self) -> Result<Vec<KrakenAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets_response = tokio::task::spawn_blocking(move || {
            let config = KrakenRestConfig::default();
            let rest_api = KrakenRestAPI::try_from(config)
                .map_err(|e| format!("Failed to create Kraken API: {e}"))?;
            rest_api.assets()
                .map_err(|e| format!("Failed to get assets: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        
        let mut assets = Vec::new();

        for (asset_name, asset_data) in assets_response {
            let asset = KrakenAsset {
                name: asset_name,
                altname: asset_data.altname,
                aclass: asset_data.aclass,
                decimals: asset_data.decimals,
                display_decimals: asset_data.display_decimals,
            };
            assets.push(asset);
        }

        Ok(assets)
    }

    /// Get recent trades for a pair
    pub async fn get_recent_trades(&self, pair: &str, since: Option<u64>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let kraken_pair = pair.replace("/", "");
        
        // Use async HTTP request directly
        let mut url = format!("https://api.kraken.com/0/public/Trades?pair={kraken_pair}");
        if let Some(since_id) = since {
            url.push_str(&format!("&since={since_id}"));
        }
        
        let response = self.http_client.get(&url).send().await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        
        let trades_response: KrakenTradesResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;
        
        if !trades_response.error.is_empty() {
            return Err(format!("Kraken API error: {:?}", trades_response.error).into());
        }
        
        let result = trades_response.result
            .ok_or::<Box<dyn std::error::Error + Send + Sync>>("No result in response".into())?;
        
        // Find the trades data for the requested pair
        let trades_data = if kraken_pair == "XBTUSD" || kraken_pair == "XXBTZUSD" {
            result.xbtusd
        } else if kraken_pair == "ETHUSD" || kraken_pair == "XETHZUSD" {
            result.ethusd
        } else {
            result.other.get(&kraken_pair).cloned()
        }.ok_or::<Box<dyn std::error::Error + Send + Sync>>(
            format!("No trades data for pair: {pair}").into()
        )?;
        
        // Convert to JSON value
        let trades_json = serde_json::json!({
            "trades": trades_data,
            "last": result.last
        });
        
        Ok(trades_json)
    }

    /// Get OHLC data for a pair
    pub async fn get_ohlc(&self, pair: &str, interval: Option<u32>, since: Option<u64>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let kraken_pair = pair.replace("/", "");
        
        // Use async HTTP request directly
        let mut url = format!("https://api.kraken.com/0/public/OHLC?pair={kraken_pair}");
        if let Some(interval_minutes) = interval {
            url.push_str(&format!("&interval={interval_minutes}"));
        }
        if let Some(since_time) = since {
            url.push_str(&format!("&since={since_time}"));
        }
        
        let response = self.http_client.get(&url).send().await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        
        let ohlc_response: KrakenOHLCResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;
        
        if !ohlc_response.error.is_empty() {
            return Err(format!("Kraken API error: {:?}", ohlc_response.error).into());
        }
        
        let result = ohlc_response.result
            .ok_or::<Box<dyn std::error::Error + Send + Sync>>("No result in response".into())?;
        
        // Find the OHLC data for the requested pair
        let ohlc_data = if kraken_pair == "XBTUSD" || kraken_pair == "XXBTZUSD" {
            result.xbtusd
        } else if kraken_pair == "ETHUSD" || kraken_pair == "XETHZUSD" {
            result.ethusd
        } else {
            result.other.get(&kraken_pair).cloned()
        }.ok_or::<Box<dyn std::error::Error + Send + Sync>>(
            format!("No OHLC data for pair: {pair}").into()
        )?;
        
        // Convert to JSON value
        let ohlc_json = serde_json::json!({
            "ohlc": ohlc_data,
            "last": result.last
        });
        
        Ok(ohlc_json)
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let status_response = tokio::task::spawn_blocking(move || {
            let config = KrakenRestConfig::default();
            let rest_api = KrakenRestAPI::try_from(config)
                .map_err(|e| format!("Failed to create Kraken API: {e}"))?;
            rest_api.system_status()
                .map_err(|e| format!("Failed to get system status: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        
        Ok(serde_json::to_value(status_response)?)
    }

    /// Get server time
    pub async fn get_server_time(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let time_response = tokio::task::spawn_blocking(move || {
            let config = KrakenRestConfig::default();
            let rest_api = KrakenRestAPI::try_from(config)
                .map_err(|e| format!("Failed to create Kraken API: {e}"))?;
            rest_api.time()
                .map_err(|e| format!("Failed to get server time: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        
        Ok(serde_json::to_value(time_response)?)
    }
}

use crate::types::TrendingItem;

/// Async function to get trending crypto pairs from Kraken
pub async fn get_trending_crypto_pairs(limit: usize) -> Result<Vec<TrendingItem>, Box<dyn std::error::Error + Send + Sync>> {
    let data_source = KrakenDataSource::new_async().await?;
    
    // Get all asset pairs using async version
    let asset_pairs = data_source.get_asset_pairs_async().await?;
    
    // Filter for crypto pairs (common crypto quote currencies)
    let crypto_quote_currencies = ["USD", "USDT", "EUR", "BTC", "ETH"];
    let crypto_pairs: Vec<String> = asset_pairs
        .into_iter()
        .filter(|pair| {
            crypto_quote_currencies.iter().any(|quote| pair.quote == *quote)
        })
        .map(|pair| pair.name)
        .collect();

    // Get ticker data for crypto pairs
    let tickers = data_source.get_tickers_async(crypto_pairs).await?;
        
    // Sort by volume and return top pairs
    let mut sorted_tickers: Vec<KrakenTicker> = tickers.into_iter().collect();
    sorted_tickers.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap_or(std::cmp::Ordering::Equal));
    
    let trending_items: Vec<TrendingItem> = sorted_tickers
        .into_iter()
        .take(limit)
        .map(|ticker| {
            // Extract base symbol from pair (e.g., "XBTUSD" -> "XBT")
            let base_symbol = if ticker.pair.len() > 3 {
                &ticker.pair[..ticker.pair.len() - 3]
            } else {
                &ticker.pair
            };
            
            TrendingItem {
                id: ticker.pair.clone(),
                symbol: base_symbol.to_string(),
                name: base_symbol.to_string(), // Kraken doesn't provide full names in ticker data
                price: Some(ticker.price),
                price_change_24h: Some(ticker.change_24h),
                price_change_percentage_24h: Some(ticker.change_pct_24h),
                volume: Some(ticker.volume),
                market_cap: None, // Kraken ticker doesn't provide market cap
                market_cap_rank: None,
                score: Some(ticker.volume), // Use volume as score for ranking
                source: "kraken".to_string(),
                image_url: None,
                last_updated: None,
            }
        })
        .collect();

    Ok(trending_items)
}

/// Get market data summary for a specific pair
pub async fn get_market_summary(pair: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let data_source = KrakenDataSource::new_async().await?;
    
    let mut summary = serde_json::Map::new();
    
    // Get ticker data
    if let Ok(tickers) = data_source.get_tickers_async(vec![pair.to_string()]).await {
            if let Some(ticker) = tickers.first() {
                summary.insert("ticker".to_string(), serde_json::to_value(ticker)?);
            }
        }
        
        // Get order book
        if let Ok(order_book) = data_source.get_order_book(pair, 10).await {
            summary.insert("order_book".to_string(), serde_json::to_value(order_book)?);
        }
        
        // Get recent trades
        if let Ok(trades) = data_source.get_recent_trades(pair, None).await {
            summary.insert("recent_trades".to_string(), trades);
        }
        
        Ok(Value::Object(summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kraken_data_source_creation() {
        let result = KrakenDataSource::new_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_tickers_with_empty_pairs() {
        let data_source = KrakenDataSource::new_async().await.unwrap();
        // Test that empty pairs now returns default pairs instead of error
        let result = data_source.get_tickers_async(vec![]).await;
        if let Err(e) = &result {
            println!("Error: {e}");
        }
        assert!(result.is_ok());
        
        let tickers = result.unwrap();
        // Should return default pairs (8 pairs as defined in the method)
        assert!(!tickers.is_empty());
        assert!(!tickers.is_empty()); // At least one default pair should be returned
    }

    #[tokio::test]
    async fn test_get_system_status() {
        let data_source = KrakenDataSource::new_async().await.unwrap();
        let result = data_source.get_system_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_server_time() {
        let data_source = KrakenDataSource::new_async().await.unwrap();
        let result = data_source.get_server_time().await;
        assert!(result.is_ok());
    }
}
