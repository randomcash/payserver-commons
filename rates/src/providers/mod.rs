//! Rate provider implementations.

mod cached;
mod coingecko;
mod fallback;
mod kraken;
mod noop;

pub use cached::CachedRateProvider;
pub use coingecko::CoinGeckoRateProvider;
pub use fallback::FallbackRateProvider;
pub use kraken::KrakenRateProvider;
pub use noop::NoOpRateProvider;
