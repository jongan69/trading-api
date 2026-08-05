use serde::{Deserialize, Serialize};

/// Basic token metadata.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub mint_address: String,
    pub description: String,
    pub image_url: Option<String>,
    pub creator: Option<String>,
    pub created_at: Option<String>,
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
}

/// Bonding-curve state for a token.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct BondingCurveInfo {
    pub mint_address: String,
    pub virtual_token_reserves: f64,
    pub virtual_sol_reserves: f64,
    pub real_token_reserves: f64,
    pub real_sol_reserves: f64,
    pub token_total_supply: f64,
    pub complete: bool,
    pub current_price_sol: Option<f64>,
    pub market_cap_sol: Option<f64>,
    pub bonding_curve_progress: Option<f64>,
}
