//! Core types for the PayServer ecosystem.
//!
//! This module contains network-agnostic types that are shared across all PayServers.
//! Network-specific types (like ERC20 tokens, Lightning invoices) are defined in
//! their respective PayServer crates.

mod events;
mod ids;
mod invoice;
mod network;
mod payment_option;
mod payout;
mod refund;
mod store;
mod token;
mod watched_address;

pub use events::{HealthStatus, PaymentEvent};
pub use ids::{InvoiceId, UserId};
pub use invoice::{AssetType, InvoiceStatus};
pub use network::Network;
pub use payment_option::{PaymentMethodId, PaymentOptionData, PaymentOptionId};
pub use payout::{PayoutData, PayoutStatus};
pub use refund::{RefundData, RefundStatus};
pub use store::{StorePaymentMethod, StoreWallet, StoreWebhook, WebhookDelivery};
pub use token::TokenData;
pub use watched_address::{CleanupAddressInfo, PendingWatchInfo};
