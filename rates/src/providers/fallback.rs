//! Fallback rate provider.
//!
//! Wraps a primary and secondary provider. If the primary fails with a
//! network/provider error, the secondary is tried. `UnsupportedPair` errors
//! are NOT retried (the pair genuinely does not exist).

use async_trait::async_trait;
use std::sync::Arc;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// A rate provider with automatic fallback.
///
/// Tries the primary provider first. On network or provider errors,
/// falls back to the secondary provider. `UnsupportedPair` errors
/// propagate immediately (the pair won't exist on the fallback either).
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
            Err(RateError::UnsupportedPair { from, to }) => {
                // Don't retry unsupported pairs on fallback
                Err(RateError::UnsupportedPair { from, to })
            }
            Err(e) => {
                tracing::warn!(
                    primary = self.primary.name(),
                    fallback = self.secondary.name(),
                    error = %e,
                    from = %from, to = %to,
                    "Primary rate provider failed, trying fallback"
                );
                self.secondary.get_rate(from, to).await
            }
        }
    }

    fn name(&self) -> &'static str {
        // The fallback is transparent — report the primary's name
        self.primary.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    struct SuccessProvider;

    #[async_trait]
    impl RateProvider for SuccessProvider {
        async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
            Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::new(350000, 2),
                timestamp: Utc::now(),
            })
        }
        fn name(&self) -> &'static str {
            "success"
        }
    }

    struct FailProvider;

    #[async_trait]
    impl RateProvider for FailProvider {
        async fn get_rate(&self, _from: &str, _to: &str) -> Result<ExchangeRate, RateError> {
            Err(RateError::ProviderError("connection refused".to_string()))
        }
        fn name(&self) -> &'static str {
            "fail"
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

    #[tokio::test]
    async fn test_primary_success() {
        let fb = FallbackRateProvider::new(Arc::new(SuccessProvider), Arc::new(FailProvider));
        let rate = fb.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(rate.rate, Decimal::new(350000, 2));
    }

    #[tokio::test]
    async fn test_fallback_on_error() {
        let fb = FallbackRateProvider::new(Arc::new(FailProvider), Arc::new(SuccessProvider));
        let rate = fb.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(rate.rate, Decimal::new(350000, 2));
    }

    #[tokio::test]
    async fn test_no_fallback_on_unsupported_pair() {
        let fb =
            FallbackRateProvider::new(Arc::new(UnsupportedProvider), Arc::new(SuccessProvider));
        let result = fb.get_rate("XYZ", "ABC").await;
        assert!(matches!(result, Err(RateError::UnsupportedPair { .. })));
    }
}
