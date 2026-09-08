//! Payments, refunds and payouts.

use serde::{Deserialize, Serialize};

use crate::invoice::InvoiceResponse;
use chrono::Utc;
use types::ChainId;
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Payment response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    /// Payment ID.
    pub id: String,
    /// Store the payment's invoice belongs to (list endpoints only).
    pub store_id: Option<String>,
    /// Store name (list endpoints only) — see [`InvoiceResponse::store_name`].
    pub store_name: Option<String>,
    /// Chain ID (EIP-155).
    pub chain_id: ChainId,
    /// Invoice ID this payment belongs to.
    pub invoice_id: String,
    /// Transaction hash.
    pub tx_hash: String,
    /// Amount received.
    pub amount: String,
    /// Asset symbol.
    pub asset_symbol: String,
    /// Token contract address (for ERC20).
    pub token_address: Option<String>,
    /// Block number.
    pub block_number: Option<u64>,
    /// Sender address.
    pub from_address: Option<String>,
    /// When the payment was detected.
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// When the payment was confirmed (None = awaiting confirmation).
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether this payment was invalidated by a chain reorg.
    pub reorged: bool,
    /// Token decimals (for display formatting).
    pub decimals: u8,
}

/// Paginated payment list response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentListResponse {
    /// Total number of matching payments.
    pub total: i64,
    /// Payments in this page.
    pub payments: Vec<PaymentResponse>,
}

/// Refund response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub id: Uuid,
    pub invoice_id: String,
    pub payment_id: Uuid,
    pub to_address: String,
    pub chain_id: ChainId,
    pub asset_type: String,
    pub asset_symbol: String,
    pub amount: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub fee_amount: Option<String>,
    pub reason: Option<String>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
}

/// Request body for creating a refund.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRefundRequest {
    /// Optional partial refund amount (in smallest unit).
    /// If omitted, refunds the full payment amount.
    pub amount: Option<String>,
    /// Reason for the refund.
    pub reason: Option<String>,
}
/// Payout response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub invoice_ids: Vec<String>,
    pub destination_address: String,
    pub chain_id: ChainId,
    pub asset_type: String,
    pub asset_symbol: String,
    pub amount: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub fee_amount: Option<String>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
}

/// Payout list response with pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutListResponse {
    pub total: i64,
    pub payouts: Vec<PayoutResponse>,
}

/// Request body for creating a payout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayoutRequest {
    /// Invoice IDs to include in this payout.
    /// If empty, sweeps all settled invoices.
    pub invoice_ids: Vec<String>,
    /// Destination wallet address.
    pub destination_address: String,
    /// EIP-155 chain ID to sweep from.
    pub chain_id: ChainId,
    /// Asset symbol to sweep (e.g., "ETH", "USDC").
    pub asset_symbol: String,
    /// Token contract address (required for ERC20 payouts).
    pub token_address: Option<String>,
}
/// Response for tx hash lookup — returns the matching invoice and payment.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxHashLookupResponse {
    /// The invoice linked to this transaction.
    pub invoice: InvoiceResponse,
    /// The payment matching the tx hash.
    pub payment: PaymentResponse,
}
