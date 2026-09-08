//! Store-related types.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// An account-level wallet: one extended public key, and the one derivation
/// counter that belongs to it (RCS-234).
///
/// Distinct from `auth::WalletCredential`, which is a wallet used to *log in*.
/// This is the one money arrives at.
///
/// The xpub and the counter live in the same row and nowhere else, and that
/// pairing is the point. An xpub reachable through two counters derives the
/// same address twice: `XpubDeriver::derive_address` is `m/44'/60'/0'/0/{i}`,
/// with no store, chain or asset in the path, so two counters that both reach
/// index 5 produce byte-identical addresses and two merchants' payments land
/// on one address. Everything that needs an address therefore asks a wallet
/// row for the next index rather than keeping a count beside it.
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: Uuid,
    /// Owning account. Wallets belong to a user, not to a store.
    pub user_id: Uuid,
    /// BIP-32 extended public key, at account level (m/44'/60'/0').
    pub xpub: String,
    /// Next derivation index to issue. Moved only by
    /// `WalletWriter::next_derivation_index`, which reads and advances it in
    /// one statement.
    pub derivation_index: i32,
    pub name: Option<String>,
    /// The wallet a store falls back to when it has no override of its own.
    /// At most one per user, enforced by a partial unique index in the schema
    /// rather than by application code.
    pub is_primary: bool,
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
    /// The account wallet this method derives from (RCS-234).
    pub wallet_id: Uuid,
    /// The wallet's xpub, joined in for convenience. Read-only here: the
    /// column lives on `wallets`, so several methods pointing at one wallet
    /// cannot drift apart (RCS-234).
    pub xpub: String,
    /// The wallet's next derivation index, joined in the same way. Shared with
    /// every other method on the same wallet, which is what stops two of them
    /// issuing the same address.
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

/// Store-level settings for defaults, branding, and notification preferences.
#[derive(Debug, Clone)]
pub struct StoreSettings {
    pub store_id: Uuid,
    /// Default chain ID for new invoices (null = no default).
    pub default_chain_id: Option<i64>,
    /// Default fiat display currency (e.g. "USD").
    pub default_display_currency: Option<String>,
    /// Logo URL for checkout branding.
    pub logo_url: Option<String>,
    /// Accent color hex (e.g. "#FF5500").
    pub accent_color: Option<String>,
    /// Per-event notification preferences: `{"event_name": {"webhook": bool}}`.
    pub notification_prefs: serde_json::Value,
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
