//! Exchange rate providers and currency utilities for PayServer.
//!
//! This crate provides:
//! - Fiat currency detection
//! - Exchange rate provider trait
//! - Kraken and CoinGecko rate provider implementations
//! - Caching wrapper with configurable TTL
//! - Fallback provider for automatic failover
//!
//! # Configuration
//!
//! - `RATE_PROVIDER` - Provider: "kraken" (default), "coingecko", "none"
//! - `RATE_PROVIDER_URL` - Custom API URL (optional)
//! - `RATE_FALLBACK_PROVIDER` - Fallback: "coingecko" (default), "kraken", "none"
//! - `RATE_CACHE_TTL_SECS` - Cache TTL in seconds (default: 30, 0 = disabled)
//!
//! # Example
//!
//! ```rust,ignore
//! use rates::{RateProvider, KrakenRateProvider, is_fiat_currency};
//!
//! // Check if currency is fiat
//! assert!(is_fiat_currency("USD"));
//! assert!(!is_fiat_currency("ETH"));
//!
//! // Fetch exchange rate
//! let provider = KrakenRateProvider::new(None);
//! let rate = provider.get_rate("USD", "ETH").await?;
//! println!("1 USD = {} ETH", rate.rate);
//! ```

mod currency;
mod provider;
mod providers;

pub use currency::{FIAT_CURRENCIES, is_crypto_currency, is_fiat_currency};
pub use provider::{ExchangeRate, RateError, RateProvider, RateProviderConfig};
pub use providers::{
    CachedRateProvider, CoinGeckoRateProvider, FallbackRateProvider, KrakenRateProvider,
    NoOpRateProvider,
};
