//! Rate provider implementations.

mod kraken;
mod noop;

pub use kraken::KrakenRateProvider;
pub use noop::NoOpRateProvider;
