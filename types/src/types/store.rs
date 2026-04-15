//! Store-related types.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Store wallet configuration for payment address derivation.
///
/// DEPRECATED: Use `StorePaymentMethod` instead for multi-chain support.
#[derive(Debug, Clone)]
pub struct StoreWallet {
    pub id: Uuid,
    pub store_id: Uuid,
    pub xpub: String,
    pub derivation_index: i32,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Store payment method configuration.
///
/// Represents a (chain, asset, wallet) combination that a store accepts for payment.
/// Each store can have multiple payment methods across different chains and assets.
#[derive(Debug, Clone)]
pub struct StorePaymentMethod {
    pub id: Uuid,
    pub store_id: Uuid,
    /// EIP-155 chain ID (1=Ethereum, 137=Polygon, 11155111=Sepolia, etc.)
    pub chain_id: u64,
    /// ERC20 token contract address, None for native asset.
    pub token_address: Option<String>,
    /// Asset symbol for display (ETH, USDC, etc.)
    pub asset_symbol: String,
    /// Number of decimals for this asset (18 for ETH, 6 for USDC/USDT).
    pub decimals: u8,
    /// BIP-32 extended public key for deriving payment addresses.
    pub xpub: String,
    /// Next derivation index to use.
    pub derivation_index: i32,
    /// Whether this payment method is enabled.
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// Store webhook configuration for invoice notifications.
#[derive(Debug, Clone)]
pub struct StoreWebhook {
    pub id: Uuid,
    pub store_id: Uuid,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Record of a single webhook delivery attempt.
#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub store_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub http_status: Option<i16>,
    pub response_body: Option<String>,
    pub latency_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub attempt_number: i32,
    pub created_at: DateTime<Utc>,
}
