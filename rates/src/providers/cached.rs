//! Caching wrapper for rate providers.
//!
//! Wraps any `RateProvider` with an in-memory cache using configurable TTL.
//! Implements stale-while-revalidate: returns stale cached rates immediately
//! while triggering a background refresh.

use async_trait::async_trait;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use crate::provider::{ExchangeRate, RateError, RateProvider};

/// A cached exchange rate entry.
struct CacheEntry {
    rate: ExchangeRate,
    fetched_at: Instant,
}

/// Caching wrapper around any `RateProvider`.
///
/// Caches successful responses keyed by normalized (from, to) pair.
/// Uses stale-while-revalidate: stale entries are served immediately
/// while a background task refreshes the value.
pub struct CachedRateProvider {
    inner: Arc<dyn RateProvider>,
    cache: Arc<RwLock<HashMap<(String, String), CacheEntry>>>,
    ttl: Duration,
    /// Tracks pairs currently being refreshed to prevent duplicate spawns.
    refreshing: Mutex<HashSet<(String, String)>>,
}

impl CachedRateProvider {
    /// Default cache TTL: 30 seconds.
    pub const DEFAULT_TTL_SECS: u64 = 30;

    pub fn new(inner: Arc<dyn RateProvider>, ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            refreshing: Mutex::new(HashSet::new()),
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

                // Stale entry — return it but trigger background refresh
                let stale_rate = entry.rate.clone();
                drop(cache);

                let mut refreshing = self.refreshing.lock().await;
                if !refreshing.contains(&key) {
                    refreshing.insert(key.clone());
                    drop(refreshing);

                    let inner = Arc::clone(&self.inner);
                    let cache = Arc::clone(&self.cache);
                    let from_owned = from.to_string();
                    let to_owned = to.to_string();
                    let provider_name = self.inner.name();

                    tokio::spawn(async move {
                        tracing::debug!(
                            from = %from_owned, to = %to_owned,
                            provider = provider_name,
                            "Stale-while-revalidate: refreshing in background"
                        );
                        match inner.get_rate(&from_owned, &to_owned).await {
                            Ok(fresh_rate) => {
                                let mut cache = cache.write().await;
                                cache.insert(
                                    (from_owned.to_uppercase(), to_owned.to_uppercase()),
                                    CacheEntry {
                                        rate: fresh_rate,
                                        fetched_at: Instant::now(),
                                    },
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    from = %from_owned, to = %to_owned,
                                    error = %e,
                                    "Background refresh failed, keeping stale entry"
                                );
                            }
                        }
                    });
                }

                tracing::debug!(
                    from = %from, to = %to,
                    provider = self.inner.name(),
                    "Serving stale cache entry while refreshing"
                );
                return Ok(stale_rate);
            }
        }

        // Cache miss — fetch synchronously
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

    #[tokio::test]
    async fn test_stale_while_revalidate() {
        let inner = Arc::new(CountingProvider::new());
        // Use a very short TTL so it expires immediately
        let cached = CachedRateProvider::new(inner.clone(), Duration::from_millis(1));

        // First call populates cache
        cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 1);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second call should return stale value and trigger background refresh
        let r2 = cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(r2.rate, Decimal::new(350000, 2)); // still the stale value

        // Give the background task time to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Inner provider should have been called again by the background task
        assert_eq!(inner.call_count(), 2);
    }

    #[tokio::test]
    async fn test_cache_ttl_expiry_miss() {
        let inner = Arc::new(CountingProvider::new());
        // Short TTL
        let cached = CachedRateProvider::new(inner.clone(), Duration::from_millis(1));

        // First call
        cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 1);

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second call returns stale, triggers refresh
        cached.get_rate("ETH", "USD").await.unwrap();

        // Let background refresh complete
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(inner.call_count(), 2);

        // Third call within new TTL should be cached
        cached.get_rate("ETH", "USD").await.unwrap();
        assert_eq!(inner.call_count(), 2);
    }
}
