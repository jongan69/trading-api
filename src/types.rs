use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize, ToSchema, IntoParams, Clone)]
pub struct YahooQuery {
    pub symbols: Option<String>,
    pub range: Option<String>,
    pub interval: Option<String>,
    pub rf_annual: Option<f64>,
    pub target_return_annual: Option<f64>,
    pub periods_per_year: Option<usize>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TickerSymbol(String);

impl TickerSymbol {
    pub fn new(s: String) -> Result<Self, String> {
        let s = s.trim().to_uppercase();
        if s.is_empty() || s.len() > 10 {
            return Err("Invalid ticker symbol".to_string());
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TickerSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for TickerSymbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for TickerSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TickerSymbol::new(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct OptionContract {
    pub symbol: String,
    pub underlying_symbol: String,
    pub strike_price: f64,
    pub expiration_date: String,
    pub r#type: String, // "call" or "put"
    pub open_interest: Option<u64>,
    pub open_interest_date: Option<String>,
    pub close_price: Option<f64>,
    pub close_price_date: Option<String>,
    pub ask_price: Option<f64>,
    pub bid_price: Option<f64>,
    pub last_price: Option<f64>,
    pub implied_volatility: Option<f64>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct OptionPrices {
    pub ask_price: f64,
    pub bid_price: f64,
    pub last_price: f64,
    pub implied_volatility: f64,
    pub open_interest: Option<u64>,
    pub open_interest_date: Option<String>,
    pub close_price_date: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct HighOpenInterestResult {
    pub short_term: Option<OptionContract>,
    pub leap: Option<OptionContract>,
    pub error: Option<String>,
}

#[derive(Clone, Deserialize, ToSchema, IntoParams)]
pub struct OptionsQuery {
    pub symbol: Option<String>,
    pub symbols: Option<String>,
    pub symbols_source: Option<String>,
    pub yahoo_search: Option<String>,
    pub yahoo_limit: Option<usize>,
    pub yahoo_list: Option<String>,
    pub yahoo_region: Option<String>,
    pub side: Option<String>,
    pub min_dte: Option<u32>,
    pub max_dte: Option<u32>,
    pub limit: Option<usize>,
    pub rf_annual: Option<f64>,
    pub range: Option<String>,
    pub interval: Option<String>,
    pub sharpe_w: Option<f64>,
    pub sortino_w: Option<f64>,
    pub calmar_w: Option<f64>,
    pub min_delta: Option<f64>,
    pub max_delta: Option<f64>,
    pub min_premium: Option<f64>,
    pub max_premium: Option<f64>,
    pub min_volume: Option<u64>,
    pub min_strike_ratio: Option<f64>,
    pub max_strike_ratio: Option<f64>,
    pub signal: Option<String>,
    pub order: Option<String>,
    pub screener: Option<String>,
    pub symbols_limit: Option<usize>,
    pub per_symbol_limit: Option<usize>,
    pub max_spread_pct: Option<f64>,
    pub feed: Option<String>,
    #[serde(rename = "type")] pub r#type: Option<String>,
    pub strike_price_gte: Option<f64>,
    pub strike_price_lte: Option<f64>,
    pub expiration_date: Option<String>,
    pub expiration_date_gte: Option<String>,
    pub expiration_date_lte: Option<String>,
    pub root_symbol: Option<String>,
    pub page_token: Option<String>,
    pub alpaca_limit: Option<u32>,
    pub underlying_top: Option<usize>,
    pub debug: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TrendingItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub price: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub volume: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub score: Option<f64>,
    pub source: String, // "coingecko" or "kraken"
    pub image_url: Option<String>,
    pub last_updated: Option<String>,
}


