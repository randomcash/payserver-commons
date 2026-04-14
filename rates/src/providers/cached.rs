//! Caching wrapper for rate providers.
//!
//! Wraps any `RateProvider` with an in-memory cache using configurable TTL.
//! Returns cached rates when fresh, fetches from the inner provider when stale.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// A cached exchange rate entry.
struct CacheEntry {
    rate: ExchangeRate,
    fetched_at: Instant,
}

/// Caching wrapper around any `RateProvider`.
///
/// Caches successful responses keyed by (from, to) pair.
/// Stale entries are refreshed on next access.
pub struct CachedRateProvider {
    inner: Arc<dyn RateProvider>,
    cache: RwLock<HashMap<(String, String), CacheEntry>>,
    ttl: Duration,
}

impl CachedRateProvider {
    /// Default cache TTL: 30 seconds.
    pub const DEFAULT_TTL_SECS: u64 = 30;

    pub fn new(inner: Arc<dyn RateProvider>, ttl: Duration) -> Self {
        Self {
            inner,
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }
}

#[async_trait]
impl RateProvider for CachedRateProvider {
    async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
        let key = (from.to_uppercase(), to.to_uppercase());

        // Check cache under read lock
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                if entry.fetched_at.elapsed() < self.ttl {
                    tracing::trace!(
                        from = %from, to = %to,
                        provider = self.inner.name(),
                        "Cache hit"
                    );
                    return Ok(entry.rate.clone());
                }
            }
        }

        // Cache miss or stale — fetch fresh rate
        tracing::debug!(
            from = %from, to = %to,
            provider = self.inner.name(),
            "Cache miss, fetching"
        );
        let rate = self.inner.get_rate(from, to).await?;

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                key,
                CacheEntry {
                    rate: rate.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(rate)
    }

    fn name(&self) -> &'static str {
        // Delegate to inner — the cache is transparent
        self.inner.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Test provider that counts how many times it was called.
    struct CountingProvider {
        calls: AtomicU32,
    }

    impl CountingProvider {
        fn new() -> Self {
            Self {
                calls: AtomicU32::new(0),
            }
        }

        fn call_count(&self) -> u32 {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl RateProvider for CountingProvider {
        async fn get_rate(&self, from: &str, to: &str) -> Result<ExchangeRate, RateError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(ExchangeRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::new(350000, 2), // 3500.00
                timestamp: Utc::now(),
            })
        }

        fn name(&self) -> &'static str {
            "counting"
        }
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let inner = Arc::new(CountingProvider::new());
        let cached = CachedRateProvider::new(inner.clone(), Duration::from_secs(60));

        // First call fetches
        let r1 = cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 1);
        assert_eq!(r1.rate, Decimal::new(350000, 2));

        // Second call should be cached
        let r2 = cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 1); // not incremented
        assert_eq!(r2.rate, Decimal::new(350000, 2));
    }

    #[tokio::test]
    async fn test_cache_miss_different_pair() {
        let inner = Arc::new(CountingProvider::new());
        let cached = CachedRateProvider::new(inner.clone(), Duration::from_secs(60));

        cached.get_rate("ETH", "USD").await.unwrap();
        cached.get_rate("BTC", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 2);
    }

    #[tokio::test]
    async fn test_cache_case_insensitive() {
        let inner = Arc::new(CountingProvider::new());
        let cached = CachedRateProvider::new(inner.clone(), Duration::from_secs(60));

        cached.get_rate("eth", "usd").await.unwrap();
        cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 1); // same pair, different case
    }
}
