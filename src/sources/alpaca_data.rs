use serde_json::Value;
use reqwest::Client;
use crate::types::OptionsQuery;

// Get News from Alpaca
pub async fn get_alpaca_news() -> Result<Value, String> {
    let key = std::env::var("ALPACA_API_KEY_ID")
        .or_else(|_| std::env::var("ALPACA_API_KEY_ID"))
        .or_else(|_| std::env::var("APCA_API_KEY_ID"))
        .map_err(|_| "ALPACA_API_KEY_ID/APCA_API_KEY_ID missing".to_string())?;
    let secret = std::env::var("ALPACA_API_SECRET_KEY")
        .or_else(|_| std::env::var("ALPACA_API_SECRET_KEY"))
        .or_else(|_| std::env::var("APCA_API_SECRET_KEY"))
        .map_err(|_| "ALPACA_API_SECRET_KEY/APCA_API_SECRET_KEY missing".to_string())?;
    let client = Client::new();
    let resp = client.get("https://data.alpaca.markets/v1beta1/news?sort=desc")
        .header("APCA-API-KEY-ID", key)
        .header("APCA-API-SECRET-KEY", secret)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("alpaca news req error: {e}"))?;
    if !resp.status().is_success() { return Err(format!("alpaca news status {}", resp.status())); }
    let v = resp.json::<Value>().await.map_err(|e| format!("alpaca news json error: {e}"))?;
    Ok(v)
}

// Get Options from Alpaca
pub async fn fetch_alpaca_snapshots(symbol: &str, q: &OptionsQuery) -> Result<Value, String> {
    // Accept ALPACA_* or APCA_* (and also ALPCA_* if user typoed in main banner)
    let key = std::env::var("ALPACA_API_KEY_ID")
        .or_else(|_| std::env::var("ALPACA_API_KEY_ID"))
        .or_else(|_| std::env::var("APCA_API_KEY_ID"))
        .map_err(|_| "ALPACA_API_KEY_ID/APCA_API_KEY_ID missing".to_string())?;
    let secret = std::env::var("ALPACA_API_SECRET_KEY")
        .or_else(|_| std::env::var("ALPACA_API_SECRET_KEY"))
        .or_else(|_| std::env::var("APCA_API_SECRET_KEY"))
        .map_err(|_| "ALPACA_API_SECRET_KEY/APCA_API_SECRET_KEY missing".to_string())?;
    // helper to perform a single request with an optional feed override
    async fn do_request(symbol: &str, headers: (&str, &str), q: &OptionsQuery, feed_override: Option<&str>) -> Result<Value, String> {
        let (key, secret) = headers;
        // keys intentionally not logged
        let mut req = Client::new()
            .get(format!("https://data.alpaca.markets/v1beta1/options/snapshots/{}", symbol))
            .header("APCA-API-KEY-ID", key)
            .header("APCA-API-SECRET-KEY", secret)
            .header("accept", "application/json");
        let mut qp: Vec<(String, String)> = Vec::new();
        if let Some(f) = feed_override.or_else(|| q.feed.as_deref()) { qp.push(("feed".into(), f.to_string())); }
        if let Some(v) = &q.r#type { qp.push(("type".into(), v.clone())); }
        qp.push(("limit".into(), q.alpaca_limit.unwrap_or(100).to_string()));
        if let Some(v) = q.strike_price_gte { qp.push(("strike_price_gte".into(), v.to_string())); }
        if let Some(v) = q.strike_price_lte { qp.push(("strike_price_lte".into(), v.to_string())); }
        if let Some(v) = &q.expiration_date { qp.push(("expiration_date".into(), v.clone())); }
        if let Some(v) = &q.expiration_date_gte { qp.push(("expiration_date_gte".into(), v.clone())); }
        if let Some(v) = &q.expiration_date_lte { qp.push(("expiration_date_lte".into(), v.clone())); }
        if let Some(v) = &q.root_symbol { qp.push(("root_symbol".into(), v.clone())); }
        if let Some(v) = &q.page_token { qp.push(("page_token".into(), v.clone())); }
        req = req.query(&qp);
        let resp = req.send().await.map_err(|e| format!("alpaca req error: {e}"))?;
        if !resp.status().is_success() { return Err(format!("alpaca status {}", resp.status())); }
        resp.json::<Value>().await.map_err(|e| format!("alpaca json error: {e}"))
    }

    let headers = (key.as_str(), secret.as_str());
    // Always include feed in the URL: use provided feed or default to indicative
    let feed = q.feed.as_deref().unwrap_or("indicative");
    do_request(symbol, headers, q, Some(feed)).await
}

// Get Stocks from Alpaca

// Get Crypto from Alpaca

// Get Forex from Alpaca

// Get Futures from Alpaca