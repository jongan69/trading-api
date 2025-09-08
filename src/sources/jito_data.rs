use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use jito_sdk_rust::{
    JitoRpcClient,
    bundle::{Bundle, BundleStatus},
    transaction::TransactionStatus,
};
use solana_sdk::{
    signature::{Keypair, Signature},
    transaction::Transaction,
    pubkey::Pubkey,
};
use crate::errors::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JitoConfig {
    pub block_engine_url: String,
    pub rpc_url: String,
    pub auth_token: Option<String>,
    pub tip_amount: Option<u64>, // in lamports
    pub max_retries: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BundleRequest {
    pub transactions: Vec<String>, // Base64 encoded transactions
    pub tip_amount: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionRequest {
    pub transaction: String, // Base64 encoded transaction
    pub tip_amount: Option<u64>,
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BundleResult {
    pub bundle_id: String,
    pub status: String,
    pub transactions: Vec<TransactionResult>,
    pub tip_amount: u64,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionResult {
    pub signature: String,
    pub status: String,
    pub slot: Option<u64>,
    pub confirmation_status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BundleStatusResult {
    pub bundle_id: String,
    pub status: String,
    pub landed_slot: Option<u64>,
    pub transactions: Vec<TransactionStatusResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionStatusResult {
    pub signature: String,
    pub confirmation_status: String,
    pub slot: Option<u64>,
    pub err: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TipAccountsResult {
    pub tip_accounts: Vec<String>,
    pub current_tip_lamports: u64,
    pub recommended_tip_lamports: u64,
}

impl Default for JitoConfig {
    fn default() -> Self {
        Self {
            block_engine_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            auth_token: None,
            tip_amount: Some(1000), // 1000 lamports default tip
            max_retries: 3,
            timeout_ms: 30000,
        }
    }
}

pub struct JitoService {
    config: JitoConfig,
    client: Option<JitoRpcClient>,
}

impl JitoService {
    pub fn new(config: JitoConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), ApiError> {
        let client = JitoRpcClient::new(&self.config.block_engine_url)
            .map_err(|e| ApiError::External(format!("Failed to initialize Jito client: {}", e)))?;
        
        self.client = Some(client);
        Ok(())
    }

    pub async fn get_tip_accounts(&self) -> Result<TipAccountsResult, ApiError> {
        let client = self.client.as_ref()
            .ok_or_else(|| ApiError::Configuration("Jito client not initialized".to_string()))?;

        let tip_accounts = client.get_tip_accounts()
            .await
            .map_err(|e| ApiError::External(format!("Failed to get tip accounts: {}", e)))?;

        // Mock data for now - actual implementation would fetch real tip information
        Ok(TipAccountsResult {
            tip_accounts: tip_accounts.into_iter().map(|acc| acc.to_string()).collect(),
            current_tip_lamports: 500,
            recommended_tip_lamports: 1000,
        })
    }

    pub async fn send_bundle(&self, request: BundleRequest) -> Result<BundleResult, ApiError> {
        let client = self.client.as_ref()
            .ok_or_else(|| ApiError::Configuration("Jito client not initialized".to_string()))?;

        // Decode transactions from base64
        let mut transactions = Vec::new();
        for tx_data in &request.transactions {
            let tx_bytes = base64::decode(tx_data)
                .map_err(|e| ApiError::InvalidInput(format!("Invalid transaction encoding: {}", e)))?;
            
            let transaction: Transaction = bincode::deserialize(&tx_bytes)
                .map_err(|e| ApiError::InvalidInput(format!("Invalid transaction format: {}", e)))?;
            
            transactions.push(transaction);
        }

        let tip_amount = request.tip_amount.unwrap_or(self.config.tip_amount.unwrap_or(1000));
        
        let bundle = Bundle::new(transactions, tip_amount);
        
        let bundle_id = client.send_bundle(&bundle)
            .await
            .map_err(|e| ApiError::External(format!("Failed to send bundle: {}", e)))?;

        // Create transaction results
        let transaction_results: Vec<TransactionResult> = request.transactions
            .into_iter()
            .enumerate()
            .map(|(i, _)| TransactionResult {
                signature: format!("pending_{}", i),
                status: "pending".to_string(),
                slot: None,
                confirmation_status: None,
                error: None,
            })
            .collect();

        Ok(BundleResult {
            bundle_id: bundle_id.to_string(),
            status: "submitted".to_string(),
            transactions: transaction_results,
            tip_amount,
            submitted_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn get_bundle_statuses(&self, bundle_ids: Vec<String>) -> Result<Vec<BundleStatusResult>, ApiError> {
        let client = self.client.as_ref()
            .ok_or_else(|| ApiError::Configuration("Jito client not initialized".to_string()))?;

        let mut results = Vec::new();

        for bundle_id in bundle_ids {
            let bundle_uuid = bundle_id.parse()
                .map_err(|e| ApiError::InvalidInput(format!("Invalid bundle ID format: {}", e)))?;

            match client.get_bundle_status(&bundle_uuid).await {
                Ok(status) => {
                    results.push(BundleStatusResult {
                        bundle_id: bundle_id.clone(),
                        status: format!("{:?}", status),
                        landed_slot: None, // Would be extracted from actual status
                        transactions: Vec::new(), // Would be populated from actual status
                    });
                }
                Err(e) => {
                    results.push(BundleStatusResult {
                        bundle_id: bundle_id.clone(),
                        status: "error".to_string(),
                        landed_slot: None,
                        transactions: vec![TransactionStatusResult {
                            signature: "unknown".to_string(),
                            confirmation_status: "error".to_string(),
                            slot: None,
                            err: Some(e.to_string()),
                        }],
                    });
                }
            }
        }

        Ok(results)
    }

    pub async fn get_inflight_bundle_statuses(&self, bundle_ids: Vec<String>) -> Result<Vec<BundleStatusResult>, ApiError> {
        let client = self.client.as_ref()
            .ok_or_else(|| ApiError::Configuration("Jito client not initialized".to_string()))?;

        let bundle_uuids: Result<Vec<_>, _> = bundle_ids
            .iter()
            .map(|id| id.parse())
            .collect();

        let bundle_uuids = bundle_uuids
            .map_err(|e| ApiError::InvalidInput(format!("Invalid bundle ID format: {}", e)))?;

        let statuses = client.get_inflight_bundle_statuses(&bundle_uuids)
            .await
            .map_err(|e| ApiError::External(format!("Failed to get inflight bundle statuses: {}", e)))?;

        let results: Vec<BundleStatusResult> = bundle_ids
            .into_iter()
            .zip(statuses.into_iter())
            .map(|(bundle_id, status)| BundleStatusResult {
                bundle_id,
                status: format!("{:?}", status),
                landed_slot: None,
                transactions: Vec::new(),
            })
            .collect();

        Ok(results)
    }

    pub async fn send_transaction(&self, request: TransactionRequest) -> Result<TransactionResult, ApiError> {
        let client = self.client.as_ref()
            .ok_or_else(|| ApiError::Configuration("Jito client not initialized".to_string()))?;

        // Decode transaction from base64
        let tx_bytes = base64::decode(&request.transaction)
            .map_err(|e| ApiError::InvalidInput(format!("Invalid transaction encoding: {}", e)))?;
        
        let transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| ApiError::InvalidInput(format!("Invalid transaction format: {}", e)))?;

        let signature = client.send_transaction(&transaction)
            .await
            .map_err(|e| ApiError::External(format!("Failed to send transaction: {}", e)))?;

        Ok(TransactionResult {
            signature: signature.to_string(),
            status: "submitted".to_string(),
            slot: None,
            confirmation_status: Some("processed".to_string()),
            error: None,
        })
    }

    pub async fn send_transaction_with_priority(&self, request: TransactionRequest) -> Result<TransactionResult, ApiError> {
        // This would implement priority fee handling
        // For now, delegate to regular send_transaction
        self.send_transaction(request).await
    }
}
