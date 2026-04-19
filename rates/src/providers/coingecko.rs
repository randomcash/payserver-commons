//! CoinGecko exchange rate provider.
//!
//! Uses the free CoinGecko API (`/simple/price`) to fetch exchange rates.
//! No API key required. Serves as a fallback to Kraken.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// CoinGecko exchange rate provider.
pub struct CoinGeckoRateProvider {
    client: reqwest::Client,
    api_url: String,
}

impl CoinGeckoRateProvider {
    pub const DEFAULT_API_URL: &'static str = "https://api.coingecko.com/api/v3";

    pub fn new(api_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_url: api_url.unwrap_or_else(|| Self::DEFAULT_API_URL.to_string()),
        }
    }

    /// Map common ticker symbols to CoinGecko IDs.
    fn to_coingecko_id(symbol: &str) -> Option<&'static str> {
        match symbol.to_uppercase().as_str() {
            "BTC" => Some("bitcoin"),
            "ETH" => Some("ethereum"),
            "USDT" => Some("tether"),
            "USDC" => Some("usd-coin"),
            "DAI" => Some("dai"),
            "MATIC" | "POL" => Some("matic-network"),
            "BNB" => Some("binancecoin"),
            "AVAX" => Some("avalanche-2"),
            "FTM" => Some("fantom"),
            "GNO" => Some("gnosis"),
            "ARB" => Some("arbitrum"),
            "OP" => Some("optimism"),
            "SOL" => Some("solana"),
            "LINK" => Some("chainlink"),
            _ => None,
        }
    }

    /// Map common fiat symbols to CoinGecko vs_currencies IDs (lowercase).
    fn to_coingecko_fiat(symbol: &str) -> Option<&'static str> {
        match symbol.to_uppercase().as_str() {
            "USD" => Some("usd"),
            "EUR" => Some("eur"),
            "GBP" => Some("gbp"),
            "JPY" => Some("jpy"),
            "CHF" => Some("chf"),
            "CAD" => Some("cad"),
            "AUD" => Some("aud"),
            "CNY" => Some("cny"),
            "SGD" => Some("sgd"),
            "HKD" => Some("hkd"),
            _ => None,
        }
    }
}

/// CoinGecko simple/price response: `{ "ethereum": { "usd": 3500.12 } }`
type CoinGeckoResponse = HashMap<String, HashMap<String, Decimal>>;

/// CoinGecko error response.
#[derive(Debug, Deserialize)]
struct CoinGeckoError {
    #[serde(default)]
    error: String,
}

#[async_trait]
impl RateProvider for CoinGeckoRateProvider {
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();

        if from_upper == to_upper {
            return Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::ONE,
                timestamp: Utc::now(),
            });
        }

        // Determine which is crypto and which is fiat
        let (crypto_id, fiat_id, needs_invert) = if let Some(id) =
            Self::to_coingecko_id(&from_upper)
        {
            // from=crypto, to=fiat  (e.g., ETH -> USD)
            let fiat =
                Self::to_coingecko_fiat(&to_upper).ok_or_else(|| RateError::UnsupportedPair {
                    from: from.to_string(),
                    to: to.to_string(),
                })?;
            (id, fiat, false)
        } else if let Some(id) = Self::to_coingecko_id(&to_upper) {
            // from=fiat, to=crypto  (e.g., USD -> ETH)
            let fiat =
                Self::to_coingecko_fiat(&from_upper).ok_or_else(|| RateError::UnsupportedPair {
                    from: from.to_string(),
                    to: to.to_string(),
                })?;
            (id, fiat, true)
        } else {
            return Err(RateError::UnsupportedPair {
                from: from.to_string(),
                to: to.to_string(),
            });
        };

        let url = format!(
            "{}/simple/price?ids={}&vs_currencies={}",
            self.api_url, crypto_id, fiat_id
        );

        tracing::debug!(from = %from, to = %to, url = %url, "Fetching rate from CoinGecko");

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let data: CoinGeckoResponse = serde_json::from_str(&text).map_err(|_| {
            // Try parsing as error response
            if let Ok(err) = serde_json::from_str::<CoinGeckoError>(&text) {
                RateError::ProviderError(err.error)
            } else {
                RateError::InvalidResponse(format!("Failed to parse CoinGecko response: {}", text))
            }
        })?;

        let prices = data.get(crypto_id).ok_or_else(|| {
            RateError::InvalidResponse(format!("No data for {} in response", crypto_id))
        })?;

        let price = prices.get(fiat_id).ok_or_else(|| {
            RateError::InvalidResponse(format!("No {} price for {}", fiat_id, crypto_id))
        })?;

        if price.is_zero() {
            return Err(RateError::InvalidResponse(
                "Received zero price from CoinGecko".to_string(),
            ));
        }

        let rate = if needs_invert {
            Decimal::ONE / price
        } else {
            *price
        };

        Ok(ExchangeRate {
            from: from.to_string(),
            to: to.to_string(),
            rate,
            timestamp: Utc::now(),
        })
    }

    fn name(&self) -> &'static str {
        "coingecko"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coingecko_id_mapping() {
        assert_eq!(
            CoinGeckoRateProvider::to_coingecko_id("ETH"),
            Some("ethereum")
        );
        assert_eq!(
            CoinGeckoRateProvider::to_coingecko_id("BTC"),
            Some("bitcoin")
        );
        assert_eq!(
            CoinGeckoRateProvider::to_coingecko_id("USDT"),
            Some("tether")
        );
        assert_eq!(CoinGeckoRateProvider::to_coingecko_id("XYZ"), None);
    }

    #[test]
    fn test_coingecko_fiat_mapping() {
        assert_eq!(CoinGeckoRateProvider::to_coingecko_fiat("USD"), Some("usd"));
        assert_eq!(CoinGeckoRateProvider::to_coingecko_fiat("EUR"), Some("eur"));
        assert_eq!(CoinGeckoRateProvider::to_coingecko_fiat("XYZ"), None);
    }

    #[test]
    fn test_coingecko_provider_name() {
        let provider = CoinGeckoRateProvider::new(None);
        assert_eq!(provider.name(), "coingecko");
    }
}
