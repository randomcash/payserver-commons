//! Fallback rate provider.
//!
//! Tries the primary provider first, falls back to a secondary provider
//! on transient errors. Does NOT fall back on `UnsupportedPair` — if the
//! primary says it doesn't support a pair, the secondary won't either
//! (the pair mappings are the same concern).

use async_trait::async_trait;
use std::sync::Arc;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// Fallback rate provider that tries a primary, then a secondary.
pub struct FallbackRateProvider {
    primary: Arc<dyn RateProvider>,
    secondary: Arc<dyn RateProvider>,
}

impl FallbackRateProvider {
    pub fn new(primary: Arc<dyn RateProvider>, secondary: Arc<dyn RateProvider>) -> Self {
        Self { primary, secondary }
    }
}

#[async_trait]
impl RateProvider for FallbackRateProvider {
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
        match self.primary.get_rate(from, to).await {
            Ok(rate) => Ok(rate),
            Err(RateError::UnsupportedPair { .. }) => {
                // Don't fallback for unsupported pairs — secondary won't help
                Err(RateError::UnsupportedPair {
                    from: from.to_string(),
                    to: to.to_string(),
                })
            }
            Err(primary_err) => {
                tracing::warn!(
                    primary = self.primary.name(),
                    secondary = self.secondary.name(),
                    error = %primary_err,
                    from = %from, to = %to,
                    "Primary rate provider failed, trying fallback"
                );
                self.secondary.get_rate(from, to).await
            }
        }
    }

    fn name(&self) -> &'static str {
        // Report as the primary — the fallback is an implementation detail
        self.primary.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    struct FailingProvider;

    #[async_trait]
    impl RateProvider for FailingProvider {
        async fn get_rate(&self, _from: &str, _to: &str) -> Result<ExchangeRate, RateError> {
            Err(RateError::ProviderError("primary down".to_string()))
        }
        fn name(&self) -> &'static str {
            "failing"
        }
    }

    struct UnsupportedProvider;

    #[async_trait]
    impl RateProvider for UnsupportedProvider {
        async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
            Err(RateError::UnsupportedPair {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
        fn name(&self) -> &'static str {
            "unsupported"
        }
    }

    struct FixedProvider;

    #[async_trait]
    impl RateProvider for FixedProvider {
        async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
            Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::new(350000, 2),
                timestamp: Utc::now(),
            })
        }
        fn name(&self) -> &'static str {
            "fixed"
        }
    }

    #[tokio::test]
    async fn test_primary_succeeds() {
        let fb = FallbackRateProvider::new(Arc::new(FixedProvider), Arc::new(FailingProvider));
        let rate = fb.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(rate.rate, Decimal::new(350000, 2));
    }

    #[tokio::test]
    async fn test_fallback_on_error() {
        let fb = FallbackRateProvider::new(Arc::new(FailingProvider), Arc::new(FixedProvider));
        let rate = fb.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(rate.rate, Decimal::new(350000, 2));
    }

    #[tokio::test]
    async fn test_no_fallback_on_unsupported_pair() {
        let fb =
            FallbackRateProvider::new(Arc::new(UnsupportedProvider), Arc::new(FixedProvider));
        let result = fb.get_rate("XYZ", "ABC").await;
        assert!(matches!(result, Err(RateError::UnsupportedPair { .. })));
    }
}
