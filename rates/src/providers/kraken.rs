//! Kraken exchange rate provider.
//!
//! Uses the Kraken public API to fetch real-time exchange rates.
//! No API key required for public ticker data.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// Kraken exchange rate provider.
///
/// Uses the Kraken public API to fetch exchange rates.
/// Supported pairs depend on Kraken's available markets.
///
/// # Example
///
/// ```rust,ignore
/// let provider = KrakenRateProvider::new(None);
/// let rate = provider.get_rate("USD", "ETH").await?;
/// println!("1 USD = {} ETH", rate.rate);
/// ```
pub struct KrakenRateProvider {
    client: reqwest::Client,
    api_url: String,
}

impl KrakenRateProvider {
    /// Default Kraken API URL.
    pub const DEFAULT_API_URL: &'static str = "https://api.kraken.com/0/public";

    /// Create a new Kraken rate provider.
    ///
    /// # Arguments
    /// * `api_url` - Optional custom API URL. Uses default if None.
    pub fn new(api_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_url: api_url.unwrap_or_else(|| Self::DEFAULT_API_URL.to_string()),
        }
    }

    /// Map currency symbols to Kraken's format.
    ///
    /// Kraken uses some non-standard symbols:
    /// - BTC -> XBT
    fn to_kraken_symbol(symbol: &str) -> String {
        match symbol.to_uppercase().as_str() {
            "BTC" => "XBT".to_string(),
            other => other.to_string(),
        }
    }

    /// Build the Kraken pair name for a trading pair.
    ///
    /// Kraken pairs are typically CRYPTO/FIAT (e.g., ETHUSD, XBTUSD).
    fn build_pair_name(from: &str, to: &str) -> String {
        let from_kr = Self::to_kraken_symbol(from);
        let to_kr = Self::to_kraken_symbol(to);
        format!("{}{}", to_kr, from_kr)
    }
}

/// Kraken API response for ticker endpoint.
#[derive(Debug, Deserialize)]
struct KrakenTickerResponse {
    error: Vec<String>,
    result: Option<HashMap<String, KrakenTickerData>>,
}

/// Ticker data from Kraken.
#[derive(Debug, Deserialize)]
struct KrakenTickerData {
    /// Last trade: [price, volume]
    c: Vec<String>,
}

#[async_trait]
impl RateProvider for KrakenRateProvider {
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
        // Normalize symbols
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();

        // If same currency, rate is 1
        if from_upper == to_upper {
            return Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::ONE,
                timestamp: Utc::now(),
            });
        }

        // Check if this is a crypto-to-same-crypto conversion
        let from_normalized = Self::to_kraken_symbol(&from_upper);
        let to_normalized = Self::to_kraken_symbol(&to_upper);
        if from_normalized == to_normalized {
            return Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::ONE,
                timestamp: Utc::now(),
            });
        }

        // Determine if we need to invert the rate
        // Kraken typically has pairs like ETHUSD, BTCUSD (crypto/fiat)
        // If from is fiat (USD, EUR) and to is crypto, we query CRYPTO/FIAT and invert
        let is_fiat_to_crypto = crate::is_fiat_currency(&from_upper);

        let (pair_name, needs_invert) = if is_fiat_to_crypto {
            // Query TO/FROM (e.g., ETHUSD for USD->ETH) and invert
            (Self::build_pair_name(from, to), true)
        } else {
            // Query FROM/TO directly
            (format!("{}{}", from_normalized, to_normalized), false)
        };

        let url = format!("{}/Ticker?pair={}", self.api_url, pair_name);

        tracing::debug!(
            from = %from,
            to = %to,
            pair = %pair_name,
            needs_invert = %needs_invert,
            "Fetching rate from Kraken"
        );

        let response: KrakenTickerResponse = self.client.get(&url).send().await?.json().await?;

        // Check for API errors
        if !response.error.is_empty() {
            let error_msg = response.error.join(", ");
            if error_msg.contains("Unknown asset pair") {
                return Err(RateError::UnsupportedPair {
                    from: from.to_string(),
                    to: to.to_string(),
                });
            }
            return Err(RateError::ProviderError(error_msg));
        }

        // Extract ticker data
        let result = response
            .result
            .ok_or_else(|| RateError::InvalidResponse("Missing result in response".to_string()))?;

        // Kraken returns data with a key that might differ from our query
        // (e.g., "XETHZUSD" instead of "ETHUSD")
        let ticker = result
            .values()
            .next()
            .ok_or_else(|| RateError::InvalidResponse("No ticker data in response".to_string()))?;

        // Use the last trade price
        let price_str = ticker
            .c
            .first()
            .ok_or_else(|| RateError::InvalidResponse("No price in ticker data".to_string()))?;

        let price: Decimal = price_str
            .parse()
            .map_err(|e| RateError::InvalidResponse(format!("Invalid price format: {}", e)))?;

        // Validate price is positive
        if price.is_zero() {
            return Err(RateError::InvalidResponse(
                "Received zero price from exchange".to_string(),
            ));
        }

        // Calculate final rate
        let rate = if needs_invert {
            // 1 USD = 1/price ETH (if price is ETH/USD)
            Decimal::ONE / price
        } else {
            price
        };

        tracing::debug!(
            from = %from,
            to = %to,
            raw_price = %price,
            final_rate = %rate,
            "Rate fetched successfully"
        );

        Ok(ExchangeRate {
            from: from.to_string(),
            to: to.to_string(),
            rate,
            timestamp: Utc::now(),
        })
    }

    fn name(&self) -> &'static str {
        "kraken"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kraken_symbol_mapping() {
        assert_eq!(KrakenRateProvider::to_kraken_symbol("BTC"), "XBT");
        assert_eq!(KrakenRateProvider::to_kraken_symbol("ETH"), "ETH");
        assert_eq!(KrakenRateProvider::to_kraken_symbol("USD"), "USD");
    }

    #[test]
    fn test_kraken_pair_name() {
        // USD -> ETH should query ETHUSD
        assert_eq!(KrakenRateProvider::build_pair_name("USD", "ETH"), "ETHUSD");
        // USD -> BTC should query XBTUSD
        assert_eq!(KrakenRateProvider::build_pair_name("USD", "BTC"), "XBTUSD");
    }

    #[test]
    fn test_kraken_provider_name() {
        let provider = KrakenRateProvider::new(None);
        assert_eq!(provider.name(), "kraken");
    }
}
