use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::sources::helius_data::{
    HeliusDataSource, SolanaAsset, SolanaTokenAccount, SolanaTransactionSignature, TokenHolding, ProgramAccountData,
};
use crate::errors::ApiError;
use crate::state::AppState;
use crate::types::TrendingItem;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SolanaQuery {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub verified_only: Option<bool>,
    pub show_fungible: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct AssetSearchQuery {
    pub owner: Option<String>,
    pub creator: Option<String>,
    pub authority: Option<String>,
    pub collection: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SolanaResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
    pub source: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/asset/{asset_id}", get(get_asset))
        .route("/assets/owner/{owner}", get(get_assets_by_owner))
        .route("/assets/creator/{creator}", get(get_assets_by_creator))
        .route("/assets/search", get(search_assets))
        .route("/token-accounts", get(get_token_accounts))
        .route("/signatures/{asset_id}", get(get_asset_signatures))
        .route("/trending", get(get_trending_solana))
        .route("/nfts/trending", get(get_trending_nfts))
        .route("/tokens/trending", get(get_trending_tokens))
        .route("/collections/top", get(get_top_collections))
        .route("/wallet/{address}/holdings", get(get_wallet_holdings))
        .route("/program/{program_id}/accounts", get(get_program_accounts))
        .with_state(state)
}

/// Get a single Solana asset by its mint address
#[utoipa::path(
    get,
    path = "/solana/asset/{asset_id}",
    params(
        ("asset_id" = String, Path, description = "Asset mint address")
    ),
    tag = "solana",
    responses((status = 200, description = "Solana asset details", body = SolanaResponse<Option<SolanaAsset>>))
)]
pub async fn get_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let asset = helius.get_asset(&asset_id).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: asset,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get assets owned by a specific address
#[utoipa::path(
    get,
    path = "/solana/assets/owner/{owner}",
    params(
        ("owner" = String, Path, description = "Owner wallet address"),
        SolanaQuery
    ),
    tag = "solana",
    responses((status = 200, description = "Assets owned by address", body = SolanaResponse<Vec<SolanaAsset>>))
)]
pub async fn get_assets_by_owner(
    State(state): State<AppState>,
    Path(owner): Path<String>,
    Query(query): Query<SolanaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let assets = helius.get_assets_by_owner(&owner, query.limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: assets,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get assets by creator address
#[utoipa::path(
    get,
    path = "/solana/assets/creator/{creator}",
    params(
        ("creator" = String, Path, description = "Creator wallet address"),
        SolanaQuery
    ),
    tag = "solana",
    responses((status = 200, description = "Assets by creator", body = SolanaResponse<Vec<SolanaAsset>>))
)]
pub async fn get_assets_by_creator(
    State(state): State<AppState>,
    Path(creator): Path<String>,
    Query(query): Query<SolanaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let assets = helius.get_assets_by_creator(&creator, query.limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: assets,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Search assets with custom criteria
#[utoipa::path(
    get,
    path = "/solana/assets/search",
    params(AssetSearchQuery),
    tag = "solana",
    responses((status = 200, description = "Search results", body = SolanaResponse<Vec<SolanaAsset>>))
)]
pub async fn search_assets(
    State(state): State<AppState>,
    Query(query): Query<AssetSearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    // Build search criteria
    let mut search_criteria = std::collections::HashMap::new();
    if let Some(owner) = query.owner {
        search_criteria.insert("owner".to_string(), owner);
    }
    if let Some(creator) = query.creator {
        search_criteria.insert("creator".to_string(), creator);
    }
    if let Some(authority) = query.authority {
        search_criteria.insert("authority".to_string(), authority);
    }
    
    let assets = helius.search_assets(search_criteria).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: assets,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get token accounts
#[utoipa::path(
    get,
    path = "/solana/token-accounts",
    params(
        ("mint" = Option<String>, Query, description = "Token mint address"),
        ("owner" = Option<String>, Query, description = "Owner address")
    ),
    tag = "solana",
    responses((status = 200, description = "Token accounts", body = SolanaResponse<Vec<SolanaTokenAccount>>))
)]
pub async fn get_token_accounts(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let mint = params.get("mint").cloned();
    let owner = params.get("owner");
    
    let token_accounts = if let Some(owner_addr) = owner {
        helius.get_token_accounts_by_owner(owner_addr, mint, None).await
    } else {
        // If no owner specified, return empty result
        Ok(Vec::new())
    }
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: token_accounts,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get transaction signatures for an asset
#[utoipa::path(
    get,
    path = "/solana/signatures/{asset_id}",
    params(
        ("asset_id" = String, Path, description = "Asset mint address"),
        ("limit" = Option<u32>, Query, description = "Number of signatures to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Transaction signatures", body = SolanaResponse<Vec<SolanaTransactionSignature>>))
)]
pub async fn get_asset_signatures(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<u32>().ok());
    
    let signatures = helius.get_signatures_for_asset(&asset_id, limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: signatures,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get trending Solana assets
#[utoipa::path(
    get,
    path = "/solana/trending",
    params(
        ("limit" = Option<u32>, Query, description = "Number of trending assets to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Trending Solana assets", body = SolanaResponse<Vec<TrendingItem>>))
)]
pub async fn get_trending_solana(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(20);
    
    let trending = helius.get_trending_solana_assets(limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: trending,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get trending NFTs
#[utoipa::path(
    get,
    path = "/solana/nfts/trending",
    params(
        ("limit" = Option<u32>, Query, description = "Number of trending NFTs to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Trending Solana NFTs", body = SolanaResponse<Vec<TrendingItem>>))
)]
pub async fn get_trending_nfts(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    // For now, use the same trending assets endpoint but filter for NFTs
    get_trending_solana(State(state), Query(params)).await
}

/// Get trending tokens
#[utoipa::path(
    get,
    path = "/solana/tokens/trending",
    params(
        ("limit" = Option<u32>, Query, description = "Number of trending tokens to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Trending Solana tokens", body = SolanaResponse<Vec<TrendingItem>>))
)]
pub async fn get_trending_tokens(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    // For now, use the same trending assets endpoint but filter for tokens
    get_trending_solana(State(state), Query(params)).await
}

/// Get top NFT collections
#[utoipa::path(
    get,
    path = "/solana/collections/top",
    params(
        ("limit" = Option<u32>, Query, description = "Number of top collections to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Top Solana NFT collections", body = SolanaResponse<Vec<TrendingItem>>))
)]
pub async fn get_top_collections(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    // This would require collection-specific logic
    // For now, return trending assets as placeholder
    get_trending_solana(State(state), Query(params)).await
}

/// Get wallet token holdings summary
#[utoipa::path(
    get,
    path = "/solana/wallet/{address}/holdings",
    params(
        ("address" = String, Path, description = "Wallet address")
    ),
    tag = "solana",
    responses((status = 200, description = "Wallet token holdings", body = SolanaResponse<Vec<TokenHolding>>))
)]
pub async fn get_wallet_holdings(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let holdings = helius.get_wallet_holdings(&address).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: holdings,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get program accounts
#[utoipa::path(
    get,
    path = "/solana/program/{program_id}/accounts",
    params(
        ("program_id" = String, Path, description = "Program ID"),
        ("limit" = Option<u32>, Query, description = "Number of accounts to return")
    ),
    tag = "solana",
    responses((status = 200, description = "Program accounts", body = SolanaResponse<Vec<ProgramAccountData>>))
)]
pub async fn get_program_accounts(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.config.helius_api_key.as_deref()
        .ok_or_else(|| ApiError::InternalError("Helius API key not configured".to_string()))?;
    
    let helius = HeliusDataSource::new_mainnet(api_key)
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<u32>().ok());
    
    let accounts = helius.get_program_accounts(&program_id, limit).await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let response = SolanaResponse {
        success: true,
        data: accounts,
        timestamp,
        source: "helius".to_string(),
    };
    
    Ok((StatusCode::OK, Json(response)))
}
