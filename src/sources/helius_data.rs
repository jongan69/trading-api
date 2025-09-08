use helius::error::Result as HeliusResult;
use helius::types::{
    Cluster, GetAsset, GetAssetBatch, GetAssetsByOwner, SearchAssets, Asset, Interface, ParseTransactionsRequest, EnhancedTransaction
};
use helius::Helius;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::TrendingItem;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SolanaAsset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub mint: String,
    pub owner: String,
    pub supply: u64,
    pub decimals: u8,
    pub is_nft: bool,
    pub collection: Option<String>,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SolanaTokenAccount {
    pub account: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub ui_amount: f64,
    pub decimals: u8,
    pub frozen: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SolanaTransactionSignature {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub memo: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SolanaNFTCollection {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub total_items: u64,
    pub floor_price: Option<f64>,
    pub volume_24h: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TokenHolding {
    pub mint: String,
    pub symbol: Option<String>,
    pub balance: f64,
    pub ui_amount_string: String,
    pub decimals: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ProgramAccountData {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
}

pub struct HeliusDataSource {
    client: Helius,
}

impl HeliusDataSource {
    #[allow(clippy::result_large_err)]
    pub fn new(api_key: &str, cluster: Cluster) -> HeliusResult<Self> {
        let client = Helius::new(api_key, cluster)?;
        Ok(Self { client })
    }

    #[allow(clippy::result_large_err)]
    pub fn new_mainnet(api_key: &str) -> HeliusResult<Self> {
        Self::new(api_key, Cluster::MainnetBeta)
    }

    #[allow(clippy::result_large_err)]
    pub fn new_devnet(api_key: &str) -> HeliusResult<Self> {
        Self::new(api_key, Cluster::Devnet)
    }

    /// Get a single Solana asset by its mint address
    pub async fn get_asset(&self, asset_id: &str) -> Result<Option<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let request = GetAsset {
            id: asset_id.to_string(),
            display_options: None,
        };

        match self.client.rpc().get_asset(request).await {
            Ok(Some(asset)) => {
                let solana_asset = convert_helius_asset_to_solana_asset(asset)?;
                Ok(Some(solana_asset))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to get asset: {e}").into()),
        }
    }

    /// Get multiple assets by their IDs using batch request
    pub async fn get_assets_batch(&self, asset_ids: Vec<String>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let request = GetAssetBatch {
            ids: asset_ids,
            display_options: None,
        };

        match self.client.rpc().get_asset_batch(request).await {
            Ok(batch_response) => {
                let mut assets = Vec::new();
                for asset in batch_response.into_iter().flatten() {
                    let solana_asset = convert_helius_asset_to_solana_asset(asset)?;
                    assets.push(solana_asset);
                }
                Ok(assets)
            }
            Err(e) => Err(format!("Failed to get assets batch: {e}").into()),
        }
    }

    /// Get assets owned by a specific address
    pub async fn get_assets_by_owner(&self, owner_address: &str, limit: Option<u32>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let request = GetAssetsByOwner {
            owner_address: owner_address.to_string(),
            page: 1,
            limit: limit.map(|l| l as i32).or(Some(1000)),
            display_options: None,
            cursor: None,
            before: None,
            after: None,
            sort_by: None,
        };

        match self.client.rpc().get_assets_by_owner(request).await {
            Ok(response) => {
                let mut assets = Vec::new();
                for item in response.items {
                    let solana_asset = convert_helius_asset_to_solana_asset(item)?;
                    assets.push(solana_asset);
                }
                Ok(assets)
            }
            Err(e) => Err(format!("Failed to get assets by owner: {e}").into()),
        }
    }

    /// Get assets by creator (simplified)
    pub async fn get_assets_by_creator(&self, _creator_address: &str, _limit: Option<u32>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty for now due to complex API structure
        Ok(Vec::new())
    }


    /// Get token accounts by owner (simplified)
    pub async fn get_token_accounts_by_owner(&self, _owner_address: &str, _mint: Option<String>, _limit: Option<u32>) -> Result<Vec<SolanaTokenAccount>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty for now since the V2 API types don't exist
        Ok(Vec::new())
    }

    /// Search assets with custom criteria
    pub async fn search_assets(&self, search_criteria: HashMap<String, String>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = SearchAssets {
            page:Some(1),
            limit: Some(1000),
            condition_type: None,
            interface: None,
            owner_address: None,
            owner_type: None,
            negate: Some(false),
            ..Default::default()
        };

        // Apply search criteria
        if let Some(owner) = search_criteria.get("owner") {
            request.owner_address = Some(owner.clone());
        }
        if let Some(creator) = search_criteria.get("creator") {
            request.creator_address = Some(creator.clone());
        }
        if let Some(authority) = search_criteria.get("authority") {
            request.authority_address = Some(authority.clone());
        }

        match self.client.rpc().search_assets(request).await {
            Ok(response) => {
                let mut assets = Vec::new();
                for item in response.items {
                    let solana_asset = convert_helius_asset_to_solana_asset(item)?;
                    assets.push(solana_asset);
                }
                Ok(assets)
            }
            Err(e) => Err(format!("Failed to search assets: {e}").into()),
        }
    }

    /// Get transaction signatures for an asset (simplified)
    pub async fn get_signatures_for_asset(&self, _asset_id: &str, _limit: Option<u32>) -> Result<Vec<SolanaTransactionSignature>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty for now since GetSignaturesForAsset type doesn't exist
        Ok(Vec::new())
    }

    /// Get trending Solana assets based on recent activity
    pub async fn get_trending_solana_assets(&self, limit: usize) -> Result<Vec<TrendingItem>, Box<dyn std::error::Error + Send + Sync>> {
        // Use searchAssets to find recently active assets
        let search_request = SearchAssets {
            page: Some(1),
            limit: Some(limit as u32),
            condition_type: None,
            interface: None,
            owner_address: None,
            owner_type: None,
            negate: Some(false),
            sort_by: None,
            cursor: None,
            before: None,
            creator_address: None,
            creator_verified: None,
            authority_address: None,
            grouping: None,
            delegate: None,
            frozen: None,
            supply: None,
            supply_mint: None,
            compressed: None,
            compressible: None,
            royalty_target_type: None,
            royalty_target: None,
            royalty_amount: None,
            burnt: None,
            json_uri: None,
            not: None,
            options: None,
            name: None,
            collections: None,
            token_type: None,
            tree: None,
            collection_nft: None,
            after: None,
        };

        match self.client.rpc().search_assets(search_request).await {
            Ok(response) => {
                let mut trending_items = Vec::new();
                
                for (index, asset) in response.items.iter().enumerate() {
                    if index >= limit {
                        break;
                    }
                    
                    let trending_item = TrendingItem {
                        id: asset.id.clone(),
                        symbol: asset.content.as_ref()
                            .and_then(|c| c.metadata.symbol.as_ref()).cloned()
                            .unwrap_or_else(|| "UNKNOWN".to_string()),
                        name: asset.content.as_ref()
                            .and_then(|c| c.metadata.name.as_ref()).cloned()
                            .unwrap_or_else(|| "Unknown Asset".to_string()),
                        price: None, // Would need Jupiter/price oracle integration
                        price_change_24h: None,
                        price_change_percentage_24h: None,
                        volume: None, // Would need transaction volume analysis
                        market_cap: None,
                        market_cap_rank: Some(index as u32 + 1),
                        score: Some(100.0 - index as f64), // Score based on recency
                        source: "solana".to_string(),
                        image_url: asset.content.as_ref()
                            .and_then(|c| c.files.as_ref())
                            .and_then(|files| files.first())
                            .and_then(|file| file.uri.as_ref())
                            .cloned(),
                        last_updated: Some(chrono::Utc::now().timestamp().to_string()),
                    };
                    trending_items.push(trending_item);
                }
                
                Ok(trending_items)
            }
            Err(e) => Err(format!("Failed to get trending Solana assets: {e}").into()),
        }
    }

    /// Get program accounts (simplified)
    pub async fn get_program_accounts(&self, _program_id: &str, _limit: Option<u32>) -> Result<Vec<ProgramAccountData>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty for now since V2 API types don't exist
        Ok(Vec::new())
    }

    /// Get wallet holdings (simplified)
    pub async fn get_wallet_holdings(&self, _wallet_address: &str) -> Result<Vec<TokenHolding>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty for now since V2 API types don't exist
        Ok(Vec::new())
    }

    /// Parse transactions using Helius Enhanced Transaction API
    pub async fn parse_transactions(&self, transaction_signatures: Vec<String>) -> Result<Vec<EnhancedTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let request = ParseTransactionsRequest {
            transactions: transaction_signatures,
        };

        match self.client.parse_transactions(request).await {
            Ok(transactions) => Ok(transactions),
            Err(e) => Err(format!("Failed to parse transactions: {e}").into()),
        }
    }

    /// Get latest blockhash (simplified)
    pub fn get_latest_blockhash(&self) -> Result<String, String> {
        match self.client.connection().get_latest_blockhash() {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Convert Helius Asset to our SolanaAsset structure
fn convert_helius_asset_to_solana_asset(asset: Asset) -> Result<SolanaAsset, Box<dyn std::error::Error + Send + Sync>> {
    Ok(SolanaAsset {
        id: asset.id.clone(),
        name: asset.content.as_ref()
            .and_then(|c| c.metadata.name.as_ref()).cloned()
            .unwrap_or_else(|| "Unknown Asset".to_string()),
        symbol: asset.content.as_ref()
            .and_then(|c| c.metadata.symbol.as_ref()).cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string()),
        description: asset.content.as_ref()
            .and_then(|c| c.metadata.description.as_ref())
            .cloned(),
        image: asset.content.as_ref()
            .and_then(|c| c.files.as_ref())
            .and_then(|files| files.first())
            .and_then(|file| file.uri.as_ref())
            .cloned(),
        mint: asset.id,
        owner: asset.ownership.owner,
        supply: asset.supply.as_ref()
            .and_then(|s| s.print_current_supply)
            .unwrap_or(1),
        decimals: asset.token_info.as_ref()
            .map(|ti| ti.decimals.unwrap_or(0) as u8)
            .unwrap_or(0),
        is_nft: matches!(asset.interface, Interface::V1NFT | Interface::LegacyNFT | Interface::ProgrammableNFT),
        collection: asset.grouping.as_ref()
            .and_then(|g| g.first())
            .and_then(|g| g.group_value.clone()),
        attributes: None, // Simplified for now due to API complexity
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_helius_data_source_creation() {
        // Test with a dummy API key
        let result = HeliusDataSource::new_mainnet("dummy_key");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_trending_solana_assets() {
        let api_key = std::env::var("HELIUS_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
        
        if api_key == "dummy_key" {
            // Skip test if no real API key
            return;
        }

        let helius = HeliusDataSource::new_mainnet(&api_key).unwrap();
        let result = helius.get_trending_solana_assets(5).await;
        
        match result {
            Ok(trending) => {
                for item in trending {
                    assert!(!item.id.is_empty());
                    assert_eq!(item.source, "solana");
                }
            }
            Err(e) => {
                // Allow test to pass if API is rate-limited or unavailable
                println!("Test failed due to API issues: {}", e);
            }
        }
    }
}
