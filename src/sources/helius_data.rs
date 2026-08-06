use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::types::TrendingItem;

// ── Public response types ──────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
pub struct SolanaAsset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub mint: String,
    pub owner: String,
    pub supply: Option<u64>,
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

// ── Data source ────────────────────────────────────────────────

pub struct HeliusDataSource {
    api_key: String,
    rpc_url: String,
}

impl HeliusDataSource {
    pub fn new_mainnet(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            rpc_url: "https://mainnet.helius-rpc.com".to_string(),
        }
    }

    /// Send a JSON-RPC request to Helius.
    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}?api-key={}", self.rpc_url, self.api_key);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let client = crate::http::shared_client();
        let resp = client.post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Helius RPC request failed: {e}"))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {e}"))?;

        if !status.is_success() {
            return Err(format!("Helius RPC returned {status}: {text}").into());
        }

        let json: Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse Helius response: {e}"))?;

        if let Some(err) = json.get("error") {
            return Err(format!("Helius RPC error: {err}").into());
        }

        Ok(json)
    }

    // ── DAS: getAsset ──────────────────────────────────────────

    pub async fn get_asset(&self, asset_id: &str) -> Result<Option<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("getAsset", serde_json::json!({ "id": asset_id })).await?;
        let result = json.get("result").cloned().unwrap_or(Value::Null);

        if result.is_null() { return Ok(None); }

        let asset = Self::parse_das_asset(&result);
        Ok(Some(asset))
    }

    // ── DAS: getAssetBatch ─────────────────────────────────────

    pub async fn get_assets_batch(&self, asset_ids: Vec<String>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("getAssetBatch", serde_json::json!({ "ids": asset_ids })).await?;
        let results = json.get("result").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        Ok(results.iter().map(|v| Self::parse_das_asset(v)).collect())
    }

    // ── DAS: getAssetsByOwner ──────────────────────────────────

    pub async fn get_assets_by_owner(&self, owner_address: &str, limit: Option<u32>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("getAssetsByOwner", serde_json::json!({
            "ownerAddress": owner_address,
            "page": 1,
            "limit": limit.unwrap_or(50),
            "displayOptions": { "showFungible": true, "showNativeBalance": true },
        })).await?;

        let items = json.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        Ok(items.iter().map(|v| Self::parse_das_asset(v)).collect())
    }

    // ── DAS: searchAssets ──────────────────────────────────────

    pub async fn search_assets(&self, search_criteria: HashMap<String, String>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let mut params = serde_json::Map::new();
        if let Some(owner) = search_criteria.get("owner") {
            params.insert("ownerAddress".into(), owner.clone().into());
        }
        if let Some(creator) = search_criteria.get("creator") {
            params.insert("creatorAddress".into(), creator.clone().into());
        }
        if let Some(collection) = search_criteria.get("collection") {
            params.insert("grouping".into(), serde_json::json!(["collection", collection]));
        }
        params.insert("page".into(), 1.into());
        params.insert("limit".into(), serde_json::json!(search_criteria.get("limit").and_then(|l| l.parse::<u32>().ok()).unwrap_or(20)));

        let json = self.rpc_call("searchAssets", Value::Object(params)).await?;
        let items = json.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        Ok(items.iter().map(|v| Self::parse_das_asset(v)).collect())
    }

    // ── DAS: getAssetsByCreator (via searchAssets) ─────────────

    pub async fn get_assets_by_creator(&self, creator_address: &str, limit: Option<u32>) -> Result<Vec<SolanaAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("searchAssets", serde_json::json!({
            "creatorAddress": creator_address,
            "page": 1,
            "limit": limit.unwrap_or(20),
        })).await?;

        let items = json.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        Ok(items.iter().map(|v| Self::parse_das_asset(v)).collect())
    }

    // ── DAS: getSignaturesForAsset ─────────────────────────────

    pub async fn get_signatures_for_asset(&self, asset_id: &str, limit: Option<u32>) -> Result<Vec<SolanaTransactionSignature>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("getSignaturesForAsset", serde_json::json!({
            "id": asset_id,
            "page": 1,
            "limit": limit.unwrap_or(20),
        })).await?;

        let items = json.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let sigs = items.iter().map(|item| SolanaTransactionSignature {
            signature: item.get("signature").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            slot: item.get("slot").and_then(|v| v.as_u64()).unwrap_or(0),
            block_time: item.get("blockTime").and_then(|v| v.as_i64()),
            memo: item.get("memo").and_then(|v| v.as_str()).map(|s| s.to_string()),
            status: item.get("confirmationStatus").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        }).collect();

        Ok(sigs)
    }

    // ── Solana RPC: getTokenAccountsByOwner ────────────────────

    pub async fn get_token_accounts_by_owner(&self, owner_address: &str, mint: Option<String>, limit: Option<u32>) -> Result<Vec<SolanaTokenAccount>, Box<dyn std::error::Error + Send + Sync>> {
        let mint_filter = if let Some(m) = mint {
            serde_json::json!({ "mint": m })
        } else {
            serde_json::json!({ "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" })
        };

        let json = self.rpc_call("getTokenAccountsByOwner", serde_json::json!([
            owner_address,
            mint_filter,
            { "encoding": "jsonParsed" },
        ])).await?;

        let accounts = json.get("result").and_then(|v| v.get("value")).and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let mut tokens = Vec::new();
        for acc in accounts.iter().take(limit.unwrap_or(50) as usize) {
            let info = &acc.get("account").and_then(|a| a.get("data")).and_then(|d| d.get("parsed")).and_then(|p| p.get("info"));
            tokens.push(SolanaTokenAccount {
                account: acc.get("pubkey").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                mint: info.and_then(|i| i.get("mint")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                owner: owner_address.to_string(),
                amount: info.and_then(|i| i.get("tokenAmount")).and_then(|a| a.get("amount")).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0),
                ui_amount: info.and_then(|i| i.get("tokenAmount")).and_then(|a| a.get("uiAmount")).and_then(|v| v.as_f64()).unwrap_or(0.0),
                decimals: info.and_then(|i| i.get("tokenAmount")).and_then(|a| a.get("decimals")).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                frozen: acc.get("account").and_then(|a| a.get("data")).and_then(|d| d.get("parsed")).and_then(|p| p.get("info")).and_then(|i| i.get("state")).and_then(|v| v.as_str()).map(|s| s == "frozen").unwrap_or(false),
            });
        }

        Ok(tokens)
    }

    // ── Wallet holdings (aggregates DAS + RPC) ─────────────────

    pub async fn get_wallet_holdings(&self, wallet_address: &str) -> Result<Vec<TokenHolding>, Box<dyn std::error::Error + Send + Sync>> {
        // Use DAS getAssetsByOwner with showFungible for rich token data
        let json = self.rpc_call("getAssetsByOwner", serde_json::json!({
            "ownerAddress": wallet_address,
            "page": 1,
            "limit": 100,
            "displayOptions": { "showFungible": true, "showNativeBalance": true, "showInscription": true },
        })).await?;

        let items = json.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let holdings: Vec<TokenHolding> = items.iter().filter_map(|item| {
            let token_info = item.get("token_info");
            let balance = token_info.and_then(|t| t.get("balance")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            if balance == 0.0 { return None; }

            Some(TokenHolding {
                mint: item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                symbol: token_info.and_then(|t| t.get("symbol")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                balance,
                ui_amount_string: token_info.and_then(|t| t.get("balance")).and_then(|v| v.as_str()).unwrap_or("0").to_string(),
                decimals: token_info.and_then(|t| t.get("decimals")).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            })
        }).collect();

        Ok(holdings)
    }

    // ── Trending Solana assets ─────────────────────────────────

    pub async fn get_trending_solana_assets(&self, limit: usize) -> Result<Vec<TrendingItem>, Box<dyn std::error::Error + Send + Sync>> {
        // Get assets from some well-known Solana program addresses to surface active tokens
        let known_addresses = [
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token program
            "JUP6LkbZbjS1jKKwapdHNyWw4E3KFY7eJHwF7SJDLwA7", // Jupiter
        ];

        let mut all_items: Vec<TrendingItem> = Vec::new();

        for addr in &known_addresses {
            if let Ok(assets) = self.get_assets_by_owner(addr, Some(20)).await {
                for asset in assets {
                    all_items.push(TrendingItem {
                        id: asset.id.clone(),
                        symbol: asset.symbol.clone(),
                        name: asset.name.clone(),
                        price: None,
                        price_change_24h: None,
                        price_change_percentage_24h: None,
                        volume: None,
                        market_cap: None,
                        market_cap_rank: None,
                        score: Some(asset.supply.unwrap_or(0) as f64),
                        source: "helius".to_string(),
                        image_url: asset.image.clone(),
                        last_updated: None,
                    });
                }
            }
        }

        all_items.truncate(limit);
        Ok(all_items)
    }

    // ── Solana RPC: getProgramAccounts ─────────────────────────

    pub async fn get_program_accounts(&self, program_id: &str, limit: Option<u32>) -> Result<Vec<ProgramAccountData>, Box<dyn std::error::Error + Send + Sync>> {
        let json = self.rpc_call("getProgramAccounts", serde_json::json!([
            program_id,
            { "encoding": "base64", "filters": [{ "dataSize": 0 }] },
        ])).await?;

        let accounts = json.get("result").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let limit = limit.unwrap_or(20) as usize;

        let result: Vec<ProgramAccountData> = accounts.iter().take(limit).map(|acc| ProgramAccountData {
            pubkey: acc.get("pubkey").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            lamports: acc.get("account").and_then(|a| a.get("lamports")).and_then(|v| v.as_u64()).unwrap_or(0),
            owner: acc.get("account").and_then(|a| a.get("owner")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            executable: acc.get("account").and_then(|a| a.get("executable")).and_then(|v| v.as_bool()).unwrap_or(false),
        }).collect();

        Ok(result)
    }

    // ── Helpers ────────────────────────────────────────────────

    fn parse_das_asset(v: &Value) -> SolanaAsset {
        let content = v.get("content").unwrap_or(v);
        let metadata = content.get("metadata");
        let token_info = v.get("token_info");
        let ownership = v.get("ownership");
        let grouping = v.get("grouping");

        SolanaAsset {
            id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            name: content.get("metadata").and_then(|m| m.get("name")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            symbol: content.get("metadata").and_then(|m| m.get("symbol")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            description: metadata.and_then(|m| m.get("description")).and_then(|x| x.as_str()).map(|s| s.to_string()),
            image: content.get("links").and_then(|l| l.get("image")).and_then(|x| x.as_str()).map(|s| s.to_string()),
            mint: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            owner: ownership.and_then(|o| o.get("owner")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            supply: token_info.and_then(|t| t.get("supply")).and_then(|x| x.as_f64()).map(|s| s as u64),
            decimals: token_info.and_then(|t| t.get("decimals")).and_then(|x| x.as_u64()).unwrap_or(0) as u8,
            is_nft: v.get("interface").and_then(|x| x.as_str()).map(|s| s.contains("NFT")).unwrap_or(false),
            collection: grouping.and_then(|g| g.as_array()).and_then(|arr| arr.first()).and_then(|g| g.get("collection_value")).and_then(|x| x.as_str()).map(|s| s.to_string()),
            attributes: metadata.and_then(|m| m.get("attributes")).cloned().and_then(|v| serde_json::from_value(v).ok()),
        }
    }

    // ── Legacy / simplified wrappers kept for route compatibility ──

    pub fn get_latest_blockhash(&self) -> Result<String, String> {
        // Use a static recent blockhash for demo purposes
        Ok("SIMULATED_BLOCKHASH_FOR_DEMO".to_string())
    }

    pub async fn parse_transactions(&self, _transaction_signatures: Vec<String>) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        // Stub — enhanced transactions require per-transaction fetch
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helius_data_source_creation() {
        let ds = HeliusDataSource::new_mainnet("test-key");
        assert_eq!(ds.rpc_url, "https://mainnet.helius-rpc.com");
        assert_eq!(ds.api_key, "test-key");
    }

    #[tokio::test]
    #[ignore] // requires live API key
    async fn test_get_trending_solana_assets() {
        let key = std::env::var("HELIUS_API_KEY").unwrap_or_default();
        if key.is_empty() { return; }
        let ds = HeliusDataSource::new_mainnet(&key);
        let result = ds.get_trending_solana_assets(5).await;
        assert!(result.is_ok());
    }
}
