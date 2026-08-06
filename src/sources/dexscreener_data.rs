use serde::{Deserialize, Serialize};

// ── DexScreener API types ────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct DexPair {
    #[serde(default)]
    chainId: String,
    #[serde(default)]
    dexId: String,
    #[serde(default)]
    pairAddress: String,
    baseToken: DexToken,
    quoteToken: DexToken,
    #[serde(default)]
    priceUsd: Option<String>,
    priceNative: Option<String>,
    priceChange: Option<DexPriceChange>,
    volume: Option<DexVolume>,
    liquidity: Option<DexLiquidity>,
    fdv: Option<f64>,
    #[serde(default)]
    marketCap: Option<f64>,
    #[serde(default)]
    pairCreatedAt: Option<i64>,
    info: Option<DexInfo>,
    txns: Option<DexTxns>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DexToken {
    address: String,
    name: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct DexPriceChange {
    m5: Option<f64>,
    h1: Option<f64>,
    h6: Option<f64>,
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DexVolume {
    h24: Option<f64>,
    h6: Option<f64>,
    h1: Option<f64>,
    m5: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DexLiquidity {
    usd: Option<f64>,
    base: Option<f64>,
    quote: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DexInfo {
    #[serde(default)]
    imageUrl: Option<String>,
    websites: Option<Vec<DexWebsite>>,
    socials: Option<Vec<DexSocial>>,
}

#[derive(Debug, Deserialize)]
struct DexWebsite {
    url: String,
}

#[derive(Debug, Deserialize)]
struct DexSocial {
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    handle: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DexTxns {
    h24: Option<DexTxnCount>,
}

#[derive(Debug, Deserialize)]
struct DexTxnCount {
    buys: Option<i64>,
    sells: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct TokenProfile {
    url: Option<String>,
    chainId: String,
    tokenAddress: String,
    icon: Option<String>,
    description: Option<String>,
    links: Option<Vec<TokenLink>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenLink {
    #[serde(rename = "type")]
    link_type: Option<String>,
    label: Option<String>,
    url: String,
}

// ── Public response types ─────────────────────────────────────

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct SolanaTokenPair {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: Option<f64>,
    pub price_native: Option<String>,
    pub volume_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub price_change_5m: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub price_change_6h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub buys_24h: Option<i64>,
    pub sells_24h: Option<i64>,
    pub dex: String,
    pub created_at: Option<i64>,
    pub image_url: Option<String>,
    pub websites: Vec<String>,
    pub socials: Vec<SocialLink>,
}

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct SocialLink {
    pub platform: String,
    pub handle: String,
}

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct TokenProfileInfo {
    pub address: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub links: Vec<TokenProfileLink>,
}

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct TokenProfileLink {
    pub label: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct SolanaMarketStats {
    pub trending: Vec<SolanaTokenPair>,
    pub top_volume: Vec<SolanaTokenPair>,
    pub top_gainers: Vec<SolanaTokenPair>,
}

// ── API client ────────────────────────────────────────────────

const DEX_BASE: &str = "https://api.dexscreener.com";

fn parse_optional_f64(s: &Option<String>) -> Option<f64> {
    s.as_ref().and_then(|v| v.parse::<f64>().ok())
}

fn pair_to_token_pair(p: &DexPair) -> SolanaTokenPair {
    SolanaTokenPair {
        address: p.pairAddress.clone(),
        symbol: p.baseToken.symbol.clone(),
        name: p.baseToken.name.clone(),
        price_usd: parse_optional_f64(&p.priceUsd),
        price_native: p.priceNative.clone(),
        volume_24h: p.volume.as_ref().and_then(|v| v.h24),
        market_cap: p.marketCap,
        fdv: p.fdv,
        liquidity_usd: p.liquidity.as_ref().and_then(|l| l.usd),
        price_change_5m: p.priceChange.as_ref().and_then(|c| c.m5),
        price_change_1h: p.priceChange.as_ref().and_then(|c| c.h1),
        price_change_6h: p.priceChange.as_ref().and_then(|c| c.h6),
        price_change_24h: p.priceChange.as_ref().and_then(|c| c.h24),
        buys_24h: p.txns.as_ref().and_then(|t| t.h24.as_ref()).and_then(|h| h.buys),
        sells_24h: p.txns.as_ref().and_then(|t| t.h24.as_ref()).and_then(|h| h.sells),
        dex: p.dexId.clone(),
        created_at: p.pairCreatedAt,
        image_url: p.info.as_ref().and_then(|i| i.imageUrl.clone()),
        websites: p.info.as_ref()
            .and_then(|i| i.websites.as_ref())
            .map(|w| w.iter().map(|s| s.url.clone()).collect())
            .unwrap_or_default(),
        socials: p.info.as_ref()
            .and_then(|i| i.socials.as_ref())
            .map(|s| s.iter().map(|soc| SocialLink {
                platform: soc.platform.clone().unwrap_or_default(),
                handle: soc.handle.clone().unwrap_or_default(),
            }).collect())
            .unwrap_or_default(),
    }
}

async fn dex_fetch(path: &str) -> Result<Vec<DexPair>, String> {
    let url = format!("{DEX_BASE}{path}");
    let client = crate::http::shared_client();
    let resp = client.get(&url).send().await.map_err(|e| format!("DexScreener request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("DexScreener returned {}", resp.status()));
    }

    let parsed: serde_json::Value = resp.json().await.map_err(|e| format!("JSON parse error: {e}"))?;
    let pairs: Vec<DexPair> = serde_json::from_value(parsed.get("pairs").cloned().unwrap_or(serde_json::json!([])))
        .unwrap_or_default();
    Ok(pairs)
}

// ── Public API ────────────────────────────────────────────────

/// Search Solana tokens by name or symbol.
pub async fn search_tokens(query: &str, limit: usize) -> Result<Vec<SolanaTokenPair>, String> {
    let pairs = dex_fetch(&format!("/latest/dex/search?q={query}")).await?;
    Ok(pairs.iter()
        .filter(|p| p.chainId == "solana")
        .take(limit)
        .map(|p| pair_to_token_pair(p))
        .collect())
}

/// Get all pairs for a specific token address.
pub async fn get_token_pairs(token_address: &str) -> Result<Vec<SolanaTokenPair>, String> {
    let pairs = dex_fetch(&format!("/latest/dex/tokens/{token_address}")).await?;
    Ok(pairs.iter().map(|p| pair_to_token_pair(p)).collect())
}

/// Get trending Solana tokens — uses DexScreener search sorted by volume.
pub async fn get_trending_tokens(limit: usize) -> Result<Vec<SolanaTokenPair>, String> {
    // DexScreener trending endpoint doesn't filter by chain, so search for SOL pairs
    let pairs = dex_fetch("/latest/dex/search?q=SOL").await?;
    let mut sol_pairs: Vec<SolanaTokenPair> = pairs.iter()
        .filter(|p| p.chainId == "solana")
        .map(|p| pair_to_token_pair(p))
        .collect();
    sol_pairs.sort_by(|a, b| b.volume_24h.unwrap_or(0.0).partial_cmp(&a.volume_24h.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
    sol_pairs.truncate(limit);
    Ok(sol_pairs)
}

/// Top tokens by 24h volume.
pub async fn get_top_by_volume(limit: usize) -> Result<Vec<SolanaTokenPair>, String> {
    let pairs = dex_fetch("/latest/dex/search?q=SOL").await?;
    let mut sol_pairs: Vec<SolanaTokenPair> = pairs.iter()
        .filter(|p| p.chainId == "solana")
        .map(|p| pair_to_token_pair(p))
        .collect();
    sol_pairs.sort_by(|a, b| b.volume_24h.unwrap_or(0.0).partial_cmp(&a.volume_24h.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
    sol_pairs.truncate(limit);
    Ok(sol_pairs)
}

/// Top gainers by 24h price change.
pub async fn get_top_gainers(limit: usize) -> Result<Vec<SolanaTokenPair>, String> {
    let pairs = dex_fetch("/latest/dex/search?q=SOL").await?;
    let mut sol_pairs: Vec<SolanaTokenPair> = pairs.iter()
        .filter(|p| p.chainId == "solana")
        .map(|p| pair_to_token_pair(p))
        .collect();
    sol_pairs.sort_by(|a, b| b.price_change_24h.unwrap_or(f64::MIN).partial_cmp(&a.price_change_24h.unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal));
    sol_pairs.truncate(limit);
    Ok(sol_pairs)
}

/// Combined market stats: trending, top volume, top gainers.
pub async fn get_market_stats(limit: usize) -> Result<SolanaMarketStats, String> {
    let pairs = dex_fetch("/latest/dex/search?q=SOL").await?;
    let sol_pairs: Vec<SolanaTokenPair> = pairs.iter()
        .filter(|p| p.chainId == "solana")
        .map(|p| pair_to_token_pair(p))
        .collect();

    let mut by_volume = sol_pairs.clone();
    by_volume.sort_by(|a, b| b.volume_24h.unwrap_or(0.0).partial_cmp(&a.volume_24h.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
    by_volume.truncate(limit);

    let mut by_gainers = sol_pairs.clone();
    by_gainers.sort_by(|a, b| b.price_change_24h.unwrap_or(f64::MIN).partial_cmp(&a.price_change_24h.unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal));
    by_gainers.truncate(limit);

    Ok(SolanaMarketStats {
        trending: by_volume.clone(),
        top_volume: by_volume,
        top_gainers: by_gainers,
    })
}

/// Get pump.fun tokens from DexScreener (filtered by dexId).
pub async fn get_pumpfun_tokens(limit: usize) -> Result<Vec<SolanaTokenPair>, String> {
    // Search broadly and filter for pump.fun / pumpswap
    let pairs = dex_fetch("/latest/dex/search?q=SOL").await?;
    let mut pump: Vec<SolanaTokenPair> = pairs.iter()
        .filter(|p| p.chainId == "solana" && (p.dexId == "pumpfun" || p.dexId == "pumpswap"))
        .map(|p| pair_to_token_pair(p))
        .collect();
    pump.sort_by(|a, b| b.volume_24h.unwrap_or(0.0).partial_cmp(&a.volume_24h.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
    pump.truncate(limit);
    Ok(pump)
}

/// Get token profile from DexScreener (icon, description, social links).
pub async fn get_token_profile(token_address: &str) -> Result<Option<TokenProfileInfo>, String> {
    let url = format!("{DEX_BASE}/token-profiles/latest/v1");
    let client = crate::http::shared_client();
    let resp = client.get(&url).send().await.map_err(|e| format!("Profile request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Token profiles returned {}", resp.status()));
    }

    let profiles: Vec<TokenProfile> = resp.json().await.map_err(|e| format!("JSON parse error: {e}"))?;

    let profile = profiles.into_iter().find(|p| p.tokenAddress == token_address);
    Ok(profile.map(|p| TokenProfileInfo {
        address: p.tokenAddress,
        icon: p.icon,
        description: p.description,
        links: p.links.unwrap_or_default().into_iter().map(|l| TokenProfileLink {
            label: l.label,
            url: l.url,
        }).collect(),
    }))
}
