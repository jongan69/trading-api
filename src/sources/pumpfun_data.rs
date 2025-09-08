use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use pumpfun::{
    common::types::{Cluster, PriorityFee},
    utils::CreateTokenMetadata,
    PumpFun,
    common::stream::PumpFunEvent as SdkPumpFunEvent,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    signature::Keypair,
    pubkey::Pubkey,
};
use crate::errors::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PumpFunConfig {
    pub rpc_url: String,
    pub commitment: String,
    pub private_key: Option<String>,
    pub priority_fee: PumpFunPriorityFee,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PumpFunPriorityFee {
    pub unit_limit: Option<u32>,
    pub unit_price: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenCreateRequest {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub file: String, // Path to image file
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenBuyRequest {
    pub mint_address: String,
    pub sol_amount: f64, // SOL amount
    pub slippage_bps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenSellRequest {
    pub mint_address: String,
    pub token_amount: Option<u64>, // if None, sell all tokens
    pub slippage_bps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenCreateAndBuyRequest {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub file: String, // Path to image file
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub sol_amount: f64, // SOL amount to buy
    pub slippage_bps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionResult {
    pub signature: String,
    pub success: bool,
    pub error: Option<String>,
    pub slot: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenInfo {
    pub mint_address: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_url: Option<String>,
    pub creator: String,
    pub created_at: Option<String>,
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TradeInfo {
    pub signature: String,
    pub mint_address: String,
    pub trader: String,
    pub is_buy: bool,
    pub sol_amount: f64,
    pub token_amount: u64,
    pub price: f64,
    pub timestamp: Option<String>,
    pub trade_type: String, // "dev", "user", "bot"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BondingCurveInfo {
    pub mint_address: String,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PumpFunEvent {
    pub event_type: String,
    pub token_info: Option<TokenInfo>,
    pub trade_info: Option<TradeInfo>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl Default for PumpFunConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "processed".to_string(),
            private_key: None,
            priority_fee: PumpFunPriorityFee {
                unit_limit: Some(100_000),
                unit_price: Some(100_000_000),
            },
        }
    }
}

impl PumpFunConfig {
    pub fn to_cluster(&self) -> Cluster {
        let commitment = match self.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::processed(),
        };

        let priority_fee = PriorityFee {
            unit_limit: self.priority_fee.unit_limit,
            unit_price: self.priority_fee.unit_price,
        };

        if self.rpc_url.contains("localhost") || self.rpc_url.contains("127.0.0.1") {
            Cluster::localnet(commitment, priority_fee)
        } else if self.rpc_url.contains("devnet") {
            Cluster::devnet(commitment, priority_fee)
        } else {
            Cluster::mainnet(commitment, priority_fee)
        }
    }
}

pub struct PumpFunService {
    config: PumpFunConfig,
    pumpfun: Option<PumpFun>,
}

impl PumpFunService {
    pub fn new(config: PumpFunConfig) -> Self {
        Self {
            config,
            pumpfun: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), ApiError> {
        // Initialize PumpFun client if private key is provided
        if let Some(private_key) = &self.config.private_key {
            let keypair = Keypair::from_base58_string(private_key);
            let cluster = self.config.to_cluster();
            
            let pumpfun = PumpFun::new(Arc::new(keypair), cluster);
            self.pumpfun = Some(pumpfun);
        }

        Ok(())
    }

    pub async fn create_token(&self, request: TokenCreateRequest) -> Result<TransactionResult, ApiError> {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        // Generate a new mint keypair
        let mint_keypair = Keypair::new();
        
        // Create token metadata
        let metadata = CreateTokenMetadata {
            name: request.name,
            symbol: request.symbol,
            description: request.description,
            file: request.file,
            twitter: request.twitter,
            telegram: request.telegram,
            website: request.website,
        };

        // Create priority fee
        let fee = Some(PriorityFee {
            unit_limit: self.config.priority_fee.unit_limit,
            unit_price: self.config.priority_fee.unit_price,
        });

        match pumpfun.create(mint_keypair.insecure_clone(), metadata, fee).await {
            Ok(signature) => Ok(TransactionResult {
                signature: signature.to_string(),
                success: true,
                error: None,
                slot: None,
            }),
            Err(e) => Ok(TransactionResult {
                signature: String::new(),
                success: false,
                error: Some(format!("Token creation failed: {e}")),
                slot: None,
            }),
        }
    }

    pub async fn create_and_buy_token(&self, request: TokenCreateAndBuyRequest) -> Result<TransactionResult, ApiError> {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        // Generate a new mint keypair
        let mint_keypair = Keypair::new();
        
        // Create token metadata
        let metadata = CreateTokenMetadata {
            name: request.name,
            symbol: request.symbol,
            description: request.description,
            file: request.file,
            twitter: request.twitter,
            telegram: request.telegram,
            website: request.website,
        };

        // Create priority fee
        let fee = Some(PriorityFee {
            unit_limit: self.config.priority_fee.unit_limit,
            unit_price: self.config.priority_fee.unit_price,
        });

        // Convert SOL to lamports
        let lamports = (request.sol_amount * LAMPORTS_PER_SOL as f64) as u64;

        match pumpfun.create_and_buy(mint_keypair.insecure_clone(), metadata, lamports, request.slippage_bps, fee).await {
            Ok(signature) => Ok(TransactionResult {
                signature: signature.to_string(),
                success: true,
                error: None,
                slot: None,
            }),
            Err(e) => Ok(TransactionResult {
                signature: String::new(),
                success: false,
                error: Some(format!("Token creation and buy failed: {e}")),
                slot: None,
            }),
        }
    }

    pub async fn buy_token(&self, request: TokenBuyRequest) -> Result<TransactionResult, ApiError> {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        let mint_pubkey = request.mint_address.parse::<Pubkey>()
            .map_err(|e| ApiError::InvalidInput(format!("Invalid mint address: {e}")))?;

        // Create priority fee
        let fee = Some(PriorityFee {
            unit_limit: self.config.priority_fee.unit_limit,
            unit_price: self.config.priority_fee.unit_price,
        });

        // Convert SOL to lamports
        let lamports = (request.sol_amount * LAMPORTS_PER_SOL as f64) as u64;

        match pumpfun.buy(mint_pubkey, lamports, request.slippage_bps, fee).await {
            Ok(signature) => Ok(TransactionResult {
                signature: signature.to_string(),
                success: true,
                error: None,
                slot: None,
            }),
            Err(e) => Ok(TransactionResult {
                signature: String::new(),
                success: false,
                error: Some(format!("Buy failed: {e}")),
                slot: None,
            }),
        }
    }

    pub async fn sell_token(&self, request: TokenSellRequest) -> Result<TransactionResult, ApiError> {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        let mint_pubkey = request.mint_address.parse::<Pubkey>()
            .map_err(|e| ApiError::InvalidInput(format!("Invalid mint address: {e}")))?;

        // Create priority fee
        let fee = Some(PriorityFee {
            unit_limit: self.config.priority_fee.unit_limit,
            unit_price: self.config.priority_fee.unit_price,
        });

        match pumpfun.sell(mint_pubkey, request.token_amount, request.slippage_bps, fee).await {
            Ok(signature) => Ok(TransactionResult {
                signature: signature.to_string(),
                success: true,
                error: None,
                slot: None,
            }),
            Err(e) => Ok(TransactionResult {
                signature: String::new(),
                success: false,
                error: Some(format!("Sell failed: {e}")),
                slot: None,
            }),
        }
    }

    pub async fn get_bonding_curve(&self, mint_address: &str) -> Result<BondingCurveInfo, ApiError> {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        let mint_pubkey = mint_address.parse::<Pubkey>()
            .map_err(|e| ApiError::InvalidInput(format!("Invalid mint address: {e}")))?;

        match pumpfun.get_bonding_curve_account(&mint_pubkey).await {
            Ok(curve) => Ok(BondingCurveInfo {
                mint_address: mint_address.to_string(),
                virtual_token_reserves: curve.virtual_token_reserves,
                virtual_sol_reserves: curve.virtual_sol_reserves,
                real_token_reserves: curve.real_token_reserves,
                real_sol_reserves: curve.real_sol_reserves,
                token_total_supply: curve.token_total_supply,
                complete: curve.complete,
            }),
            Err(e) => Err(ApiError::External(format!("Failed to get bonding curve: {e}"))),
        }
    }

    pub async fn subscribe_events<F>(&self, callback: F) -> Result<(), ApiError>
    where
        F: Fn(PumpFunEvent) + Send + Sync + 'static,
    {
        let pumpfun = self.pumpfun.as_ref()
            .ok_or_else(|| ApiError::Configuration("PumpFun client not initialized".to_string()))?;

        let subscription_callback = move |signature: String, event: Option<SdkPumpFunEvent>, error: Option<Box<dyn std::error::Error>>, _response| {
            let api_event = Self::convert_sdk_event_to_api_event(signature, event, error.map(|e| e.to_string()));
            callback(api_event);
        };

        match pumpfun.subscribe(None, subscription_callback).await {
            Ok(_subscription) => {
                // The subscription is now active and will continue until dropped
                Ok(())
            },
            Err(e) => Err(ApiError::External(format!("Failed to subscribe to events: {e}"))),
        }
    }

    // Helper method to convert SDK events to API events
    pub fn convert_sdk_event_to_api_event(
        signature: String,
        event: Option<SdkPumpFunEvent>,
        error: Option<String>,
    ) -> PumpFunEvent {
        match event {
            Some(SdkPumpFunEvent::Create(create_event)) => PumpFunEvent {
                event_type: "create".to_string(),
                token_info: Some(TokenInfo {
                    mint_address: create_event.mint.to_string(),
                    name: create_event.name,
                    symbol: create_event.symbol,
                    description: "".to_string(), // Description not available in CreateEvent
                    image_url: Some(create_event.uri),
                    creator: create_event.creator.to_string(),
                    created_at: Some(chrono::Utc::now().to_rfc3339()),
                    market_cap: None,
                    price: None,
                    volume_24h: None,
                }),
                trade_info: None,
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(SdkPumpFunEvent::Trade(trade_event)) => PumpFunEvent {
                event_type: "trade".to_string(),
                token_info: None,
                trade_info: Some(TradeInfo {
                    signature: signature.clone(),
                    mint_address: trade_event.mint.to_string(),
                    trader: trade_event.user.to_string(),
                    is_buy: trade_event.is_buy,
                    sol_amount: trade_event.sol_amount as f64 / LAMPORTS_PER_SOL as f64,
                    token_amount: trade_event.token_amount,
                    price: 0.0, // Would need to be calculated
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    trade_type: "user".to_string(),
                }),
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(_other_event) => PumpFunEvent {
                event_type: "other".to_string(),
                token_info: None,
                trade_info: None,
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            None => PumpFunEvent {
                event_type: "error".to_string(),
                token_info: None,
                trade_info: None,
                error,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}