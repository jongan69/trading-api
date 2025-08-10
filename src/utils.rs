use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, error, info};
use reqwest::Client;
use serde_json::Value;

use crate::errors::ApiError;
use crate::config::RetryConfig;

/// Retry a function with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ApiError>>,
{
    if !config.enabled {
        return f().await;
    }

    let mut last_error = None;
    
    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    info!("{} succeeded after {} retries", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);
                
                if attempt < config.max_retries {
                    let delay = calculate_backoff_delay(attempt, config);
                    warn!(
                        "{} failed (attempt {}/{}), retrying in {:?}: {:?}",
                        operation_name,
                        attempt + 1,
                        config.max_retries + 1,
                        delay,
                        last_error
                    );
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

fn calculate_backoff_delay(attempt: u32, config: &RetryConfig) -> Duration {
    let base_delay = Duration::from_millis(config.base_delay_ms);
    let max_delay = Duration::from_millis(config.max_delay_ms);
    
    let delay = base_delay * 2_u32.pow(attempt);
    delay.min(max_delay)
}

/// Validate a ticker symbol
pub fn validate_ticker_symbol(symbol: &str) -> Result<(), ApiError> {
    if symbol.is_empty() {
        return Err(ApiError::ValidationError("Ticker symbol cannot be empty".to_string()));
    }
    
    if symbol.len() > 10 {
        return Err(ApiError::ValidationError("Ticker symbol too long".to_string()));
    }
    
    // Check for valid characters (letters, numbers, dots, hyphens)
    if !symbol.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
        return Err(ApiError::ValidationError("Invalid characters in ticker symbol".to_string()));
    }
    
    Ok(())
}

/// Clean and normalize a ticker symbol
pub fn normalize_ticker_symbol(symbol: &str) -> String {
    symbol.trim().to_uppercase()
}

/// Check if a response indicates rate limiting
pub fn is_rate_limit_response(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// Check if a response indicates a server error that should be retried
pub fn is_retryable_error(status: reqwest::StatusCode) -> bool {
    status.is_server_error() || 
    status == reqwest::StatusCode::REQUEST_TIMEOUT ||
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// Make an HTTP request with retry logic
pub async fn make_request_with_retry(
    _client: &Client,
    request_builder: reqwest::RequestBuilder,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<reqwest::Response, ApiError> {
    retry_with_backoff(
        || async {
            let response = request_builder.try_clone()
                .ok_or_else(|| ApiError::InternalError("Cannot clone request".to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::Upstream(e.to_string()))?;
            
            if is_retryable_error(response.status()) {
                return Err(ApiError::Upstream(format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))));
            }
            
            Ok(response)
        },
        config,
        operation_name,
    ).await
}

/// Parse JSON response with error handling
pub async fn parse_json_response<T>(response: reqwest::Response) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ApiError::Upstream(format!("HTTP {status}: {error_text}")));
    }
    
    let json: T = response.json().await
        .map_err(|e| ApiError::Upstream(format!("Failed to parse JSON: {e}")))?;
    
    Ok(json)
}

/// Extract error message from various API response formats
pub fn extract_error_message(response_body: &str) -> String {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<Value>(response_body) {
        if let Some(error) = json.get("error") {
            if let Some(msg) = error.as_str() {
                return msg.to_string();
            }
        }
        if let Some(message) = json.get("message") {
            if let Some(msg) = message.as_str() {
                return msg.to_string();
            }
        }
        if let Some(msg) = json.get("msg") {
            if let Some(msg_str) = msg.as_str() {
                return msg_str.to_string();
            }
        }
    }
    
    // Fall back to the raw response body
    response_body.to_string()
}

/// Validate and clean a list of ticker symbols
pub fn validate_and_clean_symbols(symbols: &[String]) -> Result<Vec<String>, ApiError> {
    let mut cleaned_symbols = Vec::new();
    
    for symbol in symbols {
        let normalized = normalize_ticker_symbol(symbol);
        validate_ticker_symbol(&normalized)?;
        cleaned_symbols.push(normalized);
    }
    
    if cleaned_symbols.is_empty() {
        return Err(ApiError::ValidationError("At least one symbol is required".to_string()));
    }
    
    Ok(cleaned_symbols)
}

/// Calculate exponential moving average
pub fn calculate_ema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = prices[0];
    
    for &price in prices.iter().skip(1) {
        ema = (price * multiplier) + (ema * (1.0 - multiplier));
    }
    
    Some(ema)
}

/// Calculate standard deviation
pub fn calculate_std_dev(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;
    
    Some(variance.sqrt())
}

/// Format duration for logging
pub fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// Log request details for debugging
pub fn log_request_details(
    method: &str,
    url: &str,
    status: reqwest::StatusCode,
    duration: Duration,
    error: Option<&str>,
) {
    let status_code = status.as_u16();
    let duration_str = format_duration(duration);
    
    match error {
        Some(err) => {
            error!(
                "{} {} -> {} {} ({}) - Error: {}",
                method, url, status_code, status.canonical_reason().unwrap_or("Unknown"), duration_str, err
            );
        }
        None => {
            info!(
                "{} {} -> {} {} ({})",
                method, url, status_code, status.canonical_reason().unwrap_or("Unknown"), duration_str
            );
        }
    }
}
