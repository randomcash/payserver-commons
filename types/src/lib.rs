//! Common types and traits for the PayServer ecosystem.
//!
//! This crate provides the foundation for building payment servers that support
//! various blockchain networks. Each PayServer implementation (ethpayserver,
//! bitcoinpayserver, etc.) uses these common types and implements the core traits.
//!
//! # Architecture
//!
//! - `Network`: Enum of all supported blockchain networks
//! - `PayServer` trait: Core interface all payment servers implement
//! - `InvoiceData`, `PaymentData`: Generic data structures for invoices and payments
//! - Repository traits: `InvoiceRepository`, `PaymentRepository`, `WatchedAddressRepository`
//! - `Store`, `StoreRole`, `UserStore`: Multi-tenant store management
//! - Network-specific types (like ERC20 tokens) are defined in their respective PayServer crates

pub mod error;
pub mod repositories;
pub mod store;
pub mod traits;
pub mod types;

// Re-export commonly used types at the crate root for convenience.
pub use error::{PayServerError, PayServerResult};
pub use repositories::{
    // Combined traits
    DataService,
    DataServiceReader,
    DataServiceWriter,
    // Invoice
    InvoiceQueryParams,
    InvoiceReader,
    InvoiceRepository,
    InvoiceWriter,
    // Live Watched Address (for evmmonitor/Redis)
    LiveWatchedAddressReader,
    LiveWatchedAddressRepository,
    LiveWatchedAddressWriter,
    // Payment Event
    PaymentEventWriter,
    // Payment Option
    PaymentOptionReader,
    PaymentOptionRepository,
    PaymentOptionWriter,
    // Payment
    PaymentQueryParams,
    PaymentReader,
    PaymentRepository,
    PaymentWriter,
    // Errors
    RepositoryError,
    RepositoryResult,
    // Store Payment Method
    StorePaymentMethodReader,
    StorePaymentMethodRepository,
    StorePaymentMethodWriter,
    // Store Wallet (deprecated)
    StoreWalletReader,
    StoreWalletRepository,
    StoreWalletWriter,
    // Store Webhook
    StoreWebhookReader,
    StoreWebhookRepository,
    StoreWebhookWriter,
    // Token
    TokenQueryParams,
    TokenReader,
    TokenRepository,
    TokenWriter,
    // Watched Address (for PostgreSQL persistence)
    WatchedAddressReader,
    WatchedAddressRepository,
    WatchedAddressWriter,
};
pub use store::{Store, StoreId, StoreInfo};
pub use traits::{
    CreateInvoiceRequest, InvoiceData, InvoiceQuery, PayServer, PaymentData, PaymentEventPublisher,
    PaymentEventSubscriber, PaymentMonitor,
};
pub use types::{
    AssetType, CleanupAddressInfo, HealthStatus, InvoiceId, InvoiceStatus, Network, PaymentEvent,
    PaymentMethodId, PaymentOptionData, PaymentOptionId, PendingWatchInfo, StorePaymentMethod,
    StoreWallet, StoreWebhook, TokenData, UserId,
};
