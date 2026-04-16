//! Refund-related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::store::StoreId;
use crate::types::InvoiceId;

/// Status of a refund.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundStatus {
    /// Refund created, transaction not yet broadcast.
    Pending,
    /// Refund transaction broadcast to the network.
    Broadcasting,
    /// Refund transaction confirmed on-chain.
    Confirmed,
    /// Refund failed (insufficient balance, gas issues, etc.).
    Failed,
}

impl RefundStatus {
    pub fn is_final(&self) -> bool {
        matches!(self, RefundStatus::Confirmed | RefundStatus::Failed)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RefundStatus::Pending => "pending",
            RefundStatus::Broadcasting => "broadcasting",
            RefundStatus::Confirmed => "confirmed",
            RefundStatus::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RefundStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for RefundStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(RefundStatus::Pending),
            "broadcasting" => Ok(RefundStatus::Broadcasting),
            "confirmed" => Ok(RefundStatus::Confirmed),
            "failed" => Ok(RefundStatus::Failed),
            _ => Err(format!("unknown refund status: {}", s)),
        }
    }
}

/// A refund record — funds sent back to the original payer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundData {
    pub id: Uuid,
    pub invoice_id: InvoiceId,
    /// The payment being refunded.
    pub payment_id: Uuid,
    /// Store that owns the invoice.
    pub store_id: StoreId,
    /// Destination address (original payer's from_address).
    pub to_address: String,
    /// EIP-155 chain ID.
    pub chain_id: u64,
    /// Asset type (native or erc20).
    pub asset_type: String,
    /// Asset symbol (e.g., "ETH", "USDC").
    pub asset_symbol: String,
    /// Token contract address (for ERC20 refunds).
    pub token_address: Option<String>,
    /// Refund amount in smallest unit.
    pub amount: String,
    /// Transaction hash of the refund (set after broadcast).
    pub tx_hash: Option<String>,
    pub status: RefundStatus,
    /// Gas fee paid for the refund transaction.
    pub fee_amount: Option<String>,
    /// Reason for the refund.
    pub reason: Option<String>,
    /// Error message if the refund failed.
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}
