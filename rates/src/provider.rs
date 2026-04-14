//! Rate provider trait and core types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::env;
use std::sync::Arc;
use thiserror::Error;

use crate::providers::{
    CachedRateProvider, CoinGeckoRateProvider, FallbackRateProvider, KrakenRateProvider,
    NoOpRateProvider,
};

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
    /// Provider name: "kraken", "coingecko", "none".
    pub provider: String,
    /// Custom API URL (optional).
    pub api_url: Option<String>,
    /// Fallback provider name (optional). Used when the primary fails.
    pub fallback_provider: Option<String>,
    /// Cache TTL in seconds (0 = disabled).
    pub cache_ttl_secs: u64,
}

impl Default for RateProviderConfig {
    fn default() -> Self {
        Self {
            provider: "kraken".to_string(),
            api_url: None,
            fallback_provider: Some("coingecko".to_string()),
            cache_ttl_secs: CachedRateProvider::DEFAULT_TTL_SECS,
        }
    }
}

impl RateProviderConfig {
    /// Create a new configuration with the specified provider.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            api_url: None,
            fallback_provider: Some("coingecko".to_string()),
            cache_ttl_secs: CachedRateProvider::DEFAULT_TTL_SECS,
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
    /// - `RATE_FALLBACK_PROVIDER` - Fallback provider (default: "coingecko", set "none" to disable)
    /// - `RATE_CACHE_TTL_SECS` - Cache TTL in seconds (default: 30, 0 = disabled)
    pub fn from_env() -> Self {
        let fallback = env::var("RATE_FALLBACK_PROVIDER")
            .unwrap_or_else(|_| "coingecko".to_string());
        Self {
            provider: env::var("RATE_PROVIDER").unwrap_or_else(|_| "kraken".to_string()),
            api_url: env::var("RATE_PROVIDER_URL").ok(),
            fallback_provider: match fallback.to_lowercase().as_str() {
                "none" | "disabled" | "" => None,
                _ => Some(fallback),
            },
            cache_ttl_secs: env::var("RATE_CACHE_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(CachedRateProvider::DEFAULT_TTL_SECS),
        }
    }

    /// Create a bare provider by name (no caching/fallback).
    fn make_provider(name: &str, api_url: &Option<String>) -> Arc<dyn RateProvider> {
        match name.to_lowercase().as_str() {
            "coingecko" => Arc::new(CoinGeckoRateProvider::new(api_url.clone())),
            "none" | "disabled" => Arc::new(NoOpRateProvider),
            // Default to Kraken
            _ => Arc::new(KrakenRateProvider::new(api_url.clone())),
        }
    }

    /// Create a rate provider based on the configuration.
    ///
    /// Composes: Cache(Fallback(primary, secondary)) when both are configured.
    pub fn create_provider(&self) -> Arc<dyn RateProvider> {
        if self.provider.to_lowercase() == "none" || self.provider.to_lowercase() == "disabled" {
            return Arc::new(NoOpRateProvider);
        }

        let primary = Self::make_provider(&self.provider, &self.api_url);

        // Wrap with fallback if configured
        let provider: Arc<dyn RateProvider> = match &self.fallback_provider {
            Some(fb) if fb.to_lowercase() != self.provider.to_lowercase() => {
                let secondary = Self::make_provider(fb, &None);
                tracing::info!(
                    primary = self.provider,
                    fallback = fb.as_str(),
                    "Rate provider with fallback"
                );
                Arc::new(FallbackRateProvider::new(primary, secondary))
            }
            _ => primary,
        };

        // Wrap with cache if TTL > 0
        if self.cache_ttl_secs > 0 {
            tracing::info!(ttl_secs = self.cache_ttl_secs, "Rate caching enabled");
            Arc::new(CachedRateProvider::new(
                provider,
                std::time::Duration::from_secs(self.cache_ttl_secs),
            ))
        } else {
            provider
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
        assert_eq!(
            config.fallback_provider,
            Some("coingecko".to_string())
        );
        assert_eq!(config.cache_ttl_secs, 30);
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

    #[test]
    fn test_create_provider_disabled() {
        let config = RateProviderConfig::new("none");
        let provider = config.create_provider();
        assert_eq!(provider.name(), "noop");
    }

    #[test]
    fn test_create_provider_coingecko() {
        let mut config = RateProviderConfig::new("coingecko");
        config.fallback_provider = None;
        config.cache_ttl_secs = 0;
        let provider = config.create_provider();
        assert_eq!(provider.name(), "coingecko");
    }
}
