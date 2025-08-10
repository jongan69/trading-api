use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use std::collections::HashSet;
// use yahoo_finance_api;
use serde_json::Value;
use reqwest::Client;

pub async fn get_trending_from_yahoo() -> Vec<String> {
    let mut trending_stocks: HashSet<String> = HashSet::new();

    // Common headers
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    if let Ok(resp) = client
        .get("https://finance.yahoo.com/trending-tickers")
        .send()
        .await
    {
        if let Ok(body) = resp.text().await {
            let document = Html::parse_document(&body);
            let selector = Selector::parse("td, a, span, div").unwrap();
            for element in document.select(&selector) {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty()
                    && text.len() <= 5
                    && text.chars().all(|c| c.is_ascii_uppercase())
                {
                    trending_stocks.insert(text);
                }
            }
        }
    }

    trending_stocks.into_iter().collect()
}

pub async fn yahoo_predefined_list(scr_id: &str, count: usize) -> Result<Vec<String>, String> {
    let url = format!(
        "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count}&scrIds={scr_id}"
    );
    let resp = Client::new()
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("yahoo predefined req error: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("yahoo predefined status {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("yahoo predefined json error: {e}"))?;
    let mut out = Vec::new();
    if let Some(quotes) = v
        .get("finance")
        .and_then(|f| f.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|o| o.get("quotes"))
        .and_then(|q| q.as_array())
    {
        for q in quotes {
            if let Some(sym) = q.get("symbol").and_then(|s| s.as_str()) {
                if !sym.is_empty() {
                    out.push(sym.to_string());
                }
            }
        }
    }
    Ok(out)
}

pub async fn yahoo_trending(region: &str, count: usize) -> Result<Vec<String>, String> {
    let url = format!(
        "https://query1.finance.yahoo.com/v1/finance/trending/{region}?count={count}"
    );
    let resp = Client::new()
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("yahoo trending req error: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("yahoo trending status {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("yahoo trending json error: {e}"))?;
    let mut out = Vec::new();
    if let Some(quotes) = v
        .get("finance")
        .and_then(|f| f.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|o| o.get("quotes"))
        .and_then(|q| q.as_array())
    {
        for q in quotes {
            if let Some(sym) = q.get("symbol").and_then(|s| s.as_str()) {
                if !sym.is_empty() {
                    out.push(sym.to_string());
                }
            }
        }
    }
    Ok(out)
}

pub async fn fetch_yahoo_options_chain(symbol: &str) -> Result<Value, String> {
    let url = format!("https://query2.finance.yahoo.com/v7/finance/options/{symbol}");
    let resp = Client::new()
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("yahoo options req error: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("yahoo options status {}", resp.status()));
    }
    resp.json::<Value>().await.map_err(|e| format!("yahoo options json error: {e}"))
}

// pub async fn get_news() -> Result<Value, String> {
    
// }

