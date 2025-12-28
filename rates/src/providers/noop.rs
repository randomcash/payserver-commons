//! No-op rate provider for testing or disabled rate fetching.

use async_trait::async_trait;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// A no-op rate provider that always returns an error.
///
/// Use this when rate fetching is disabled or for testing scenarios
/// where you want to ensure no external API calls are made.
pub struct NoOpRateProvider;

#[async_trait]
impl RateProvider for NoOpRateProvider {
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
        Err(RateError::UnsupportedPair {
            from: from.to_string(),
            to: to.to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "none"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_provider_returns_error() {
        let provider = NoOpRateProvider;
        let result = provider.get_rate("USD", "ETH").await;
        assert!(result.is_err());

        match result {
            Err(RateError::UnsupportedPair { from, to }) => {
                assert_eq!(from, "USD");
                assert_eq!(to, "ETH");
            }
            _ => panic!("Expected UnsupportedPair error"),
        }
    }

    #[test]
    fn test_noop_provider_name() {
        let provider = NoOpRateProvider;
        assert_eq!(provider.name(), "none");
    }
}
