//! Rate provider trait and core types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::env;
use std::sync::Arc;
use thiserror::Error;

use crate::providers::{KrakenRateProvider, NoOpRateProvider};

/// Errors that can occur when fetching exchange rates.
#[derive(Debug, Error)]
pub enum RateError {
    /// The trading pair is not supported by the provider.
    #[error("Unsupported trading pair: {from}/{to}")]
    UnsupportedPair { from: String, to: String },

    /// Network or HTTP error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Invalid response from the rate provider.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Rate provider returned an error.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Rate is stale or unavailable.
    #[error("Rate unavailable")]
    Unavailable,
}

/// An exchange rate between two currencies.
#[derive(Debug, Clone)]
pub struct ExchangeRate {
    /// Source currency (e.g., "USD").
    pub from: String,
    /// Target currency (e.g., "ETH").
    pub to: String,
    /// Exchange rate: 1 `from` = `rate` `to`.
    pub rate: Decimal,
    /// When this rate was fetched.
    pub timestamp: DateTime<Utc>,
}

/// Trait for exchange rate providers.
///
/// Implementations fetch real-time exchange rates from external APIs.
/// The trait is designed to be provider-agnostic, allowing easy swapping
/// between different rate sources (Kraken, CoinGecko, etc.).
#[async_trait]
pub trait RateProvider: Send + Sync {
    /// Get the exchange rate from one currency to another.
    ///
    /// # Arguments
    /// * `from` - Source currency (e.g., "USD", "EUR")
    /// * `to` - Target currency (e.g., "ETH", "BTC")
    ///
    /// # Returns
    /// The exchange rate where 1 `from` = `rate` `to`.
    ///
    /// # Example
    /// ```rust,ignore
    /// let rate = provider.get_rate("USD", "ETH").await?;
    /// // If rate.rate = 0.0005, then 1 USD = 0.0005 ETH
    /// let eth_amount = usd_amount * rate.rate;
    /// ```
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError>;

    /// Get the name of this rate provider.
    fn name(&self) -> &'static str;
}

/// Configuration for rate providers.
#[derive(Debug, Clone)]
pub struct RateProviderConfig {
    /// Provider name: "kraken", "none".
    pub provider: String,
    /// Custom API URL (optional).
    pub api_url: Option<String>,
}

impl Default for RateProviderConfig {
    fn default() -> Self {
        Self {
            provider: "kraken".to_string(),
            api_url: None,
        }
    }
}

impl RateProviderConfig {
    /// Create a new configuration with the specified provider.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            api_url: None,
        }
    }

    /// Set a custom API URL.
    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = Some(url);
        self
    }

    /// Load configuration from environment variables.
    ///
    /// - `RATE_PROVIDER` - Provider name (default: "kraken")
    /// - `RATE_PROVIDER_URL` - Custom API URL (optional)
    pub fn from_env() -> Self {
        Self {
            provider: env::var("RATE_PROVIDER").unwrap_or_else(|_| "kraken".to_string()),
            api_url: env::var("RATE_PROVIDER_URL").ok(),
        }
    }

    /// Create a rate provider based on the configuration.
    pub fn create_provider(&self) -> Arc<dyn RateProvider> {
        match self.provider.to_lowercase().as_str() {
            "none" | "disabled" => Arc::new(NoOpRateProvider),
            _ => Arc::new(KrakenRateProvider::new(self.api_url.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_provider_config_default() {
        let config = RateProviderConfig::default();
        assert_eq!(config.provider, "kraken");
        assert!(config.api_url.is_none());
    }

    #[test]
    fn test_rate_provider_config_new() {
        let config = RateProviderConfig::new("none");
        assert_eq!(config.provider, "none");
    }

    #[test]
    fn test_rate_provider_config_with_url() {
        let config =
            RateProviderConfig::new("kraken").with_api_url("https://custom.api.com".to_string());
        assert_eq!(config.api_url, Some("https://custom.api.com".to_string()));
    }
}
