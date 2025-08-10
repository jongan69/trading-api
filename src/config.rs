use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub alpaca: AlpacaConfig,
    pub reddit: Option<RedditConfig>,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub rate_limiting: RateLimitConfig,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditConfig {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_request_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub enabled: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let alpaca_key = env::var("ALPACA_API_KEY_ID")
            .or_else(|_| env::var("APCA_API_KEY_ID"))
            .map_err(|_| "ALPACA_API_KEY_ID/APCA_API_KEY_ID missing".to_string())?;
        
        let alpaca_secret = env::var("ALPACA_API_SECRET_KEY")
            .or_else(|_| env::var("APCA_API_SECRET_KEY"))
            .map_err(|_| "ALPACA_API_SECRET_KEY/APCA_API_SECRET_KEY missing".to_string())?;

        let alpaca = AlpacaConfig {
            api_key: alpaca_key,
            api_secret: alpaca_secret,
            base_url: "https://api.alpaca.markets".to_string(),
        };

        let reddit = if let (Ok(client_id), Ok(client_secret), Ok(username), Ok(password)) = (
            env::var("REDDIT_CLIENT_ID"),
            env::var("REDDIT_CLIENT_SECRET"),
            env::var("REDDIT_USERNAME"),
            env::var("REDDIT_PASSWORD"),
        ) {
            Some(RedditConfig {
                client_id,
                client_secret,
                username,
                password,
            })
        } else {
            None
        };

        let server = ServerConfig {
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        };

        let logging = LoggingConfig {
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            enable_request_logging: env::var("ENABLE_REQUEST_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        };

        let rate_limiting = RateLimitConfig {
            requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            enabled: env::var("RATE_LIMIT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        };

        let retry = RetryConfig {
            max_retries: env::var("RETRY_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            base_delay_ms: env::var("RETRY_BASE_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            max_delay_ms: env::var("RETRY_MAX_DELAY_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            enabled: env::var("RETRY_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        };

        Ok(Config {
            alpaca,
            reddit,
            server,
            logging,
            rate_limiting,
            retry,
        })
    }

    pub fn alpaca_headers(&self) -> (String, String) {
        (self.alpaca.api_key.clone(), self.alpaca.api_secret.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            alpaca: AlpacaConfig {
                api_key: String::new(),
                api_secret: String::new(),
                base_url: "https://api.alpaca.markets".to_string(),
            },
            reddit: None,
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_request_logging: true,
            },
            rate_limiting: RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
                enabled: true,
            },
            retry: RetryConfig {
                max_retries: 3,
                base_delay_ms: 1000,
                max_delay_ms: 10000,
                enabled: true,
            },
        }
    }
}
