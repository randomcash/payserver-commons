//! The public checkout page.

use serde::{Deserialize, Serialize};

use crate::invoice::PaymentOptionResponse;
use types::ChainId;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Public checkout response — only payment-relevant fields.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    /// Invoice ID.
    pub id: String,
    /// Invoice currency (e.g., "USD").
    pub currency: String,
    /// Current status.
    pub status: String,
    /// Requested amount.
    pub amount: String,
    /// Amount received so far.
    pub amount_received: String,
    /// Expiration timestamp.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Whether the invoice is expired.
    pub is_expired: bool,
    /// Whether the invoice is fully paid.
    pub is_paid: bool,
    /// Payment options (addresses, chains, amounts).
    pub payment_options: Vec<PaymentOptionResponse>,
    /// Payments received (privacy-filtered — no sender addresses).
    pub payments: Vec<CheckoutPaymentInfo>,
}

/// Public view of a payment on the checkout page.
///
/// Deliberately omits fields present on the authenticated `PaymentResponse`:
/// - `from_address` — sender wallet address. Leaking it here would let anyone
///   with the invoice link correlate an invoice to the customer's wallet.
/// - `reorged` — internal state that confuses customers with transient
///   "your payment was invalidated" UX when a transient reorg happens.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutPaymentInfo {
    /// Payment ID.
    pub id: String,
    /// Chain ID (EIP-155).
    pub chain_id: ChainId,
    /// Transaction hash — already public on-chain.
    pub tx_hash: String,
    /// Amount received (smallest unit as string).
    pub amount: String,
    /// Asset symbol.
    pub asset_symbol: String,
    /// Token contract address (ERC20 only, None for native).
    pub token_address: Option<String>,
    /// Block number (for confirmation counting).
    pub block_number: Option<u64>,
    /// When the payment was detected.
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// When the payment reached required confirmations (None = pending).
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
}
