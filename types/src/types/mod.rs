//! Core types for the PayServer ecosystem.
//!
//! This module contains network-agnostic types that are shared across all PayServers.
//! Network-specific types (like ERC20 tokens, Lightning invoices) are defined in
//! their respective PayServer crates.

mod events;
mod ids;
mod invoice;
mod network;
mod store;
mod token;
mod watched_address;

pub use events::{HealthStatus, PaymentEvent};
pub use ids::{InvoiceId, UserId};
pub use invoice::{AssetType, InvoiceStatus};
pub use network::Network;
pub use store::{StoreWallet, StoreWebhook};
pub use token::TokenData;
pub use watched_address::{CleanupAddressInfo, PendingWatchInfo};
