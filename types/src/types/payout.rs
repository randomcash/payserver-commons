//! Payout/settlement types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::store::StoreId;

/// Status of a payout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutStatus {
    /// Payout created, transactions not yet broadcast.
    Pending,
    /// Payout transactions being broadcast.
    Broadcasting,
    /// All payout transactions confirmed.
    Confirmed,
    /// Payout failed.
    Failed,
}

impl PayoutStatus {
    pub fn is_final(&self) -> bool {
        matches!(self, PayoutStatus::Confirmed | PayoutStatus::Failed)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PayoutStatus::Pending => "pending",
            PayoutStatus::Broadcasting => "broadcasting",
            PayoutStatus::Confirmed => "confirmed",
            PayoutStatus::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PayoutStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PayoutStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(PayoutStatus::Pending),
            "broadcasting" => Ok(PayoutStatus::Broadcasting),
            "confirmed" => Ok(PayoutStatus::Confirmed),
            "failed" => Ok(PayoutStatus::Failed),
            _ => Err(format!("unknown payout status: {}", s)),
        }
    }
}

/// A payout record — sweeping funds from derived addresses to the merchant's wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutData {
    pub id: Uuid,
    pub store_id: StoreId,
    /// Invoice IDs included in this payout.
    pub invoice_ids: Vec<String>,
    /// Destination address (merchant's configured wallet).
    pub destination_address: String,
    /// EIP-155 chain ID.
    pub chain_id: u64,
    /// Asset type (native or erc20).
    pub asset_type: String,
    /// Asset symbol.
    pub asset_symbol: String,
    /// Token contract address (for ERC20 payouts).
    pub token_address: Option<String>,
    /// Total amount swept in smallest unit.
    pub amount: String,
    /// Transaction hash (set after broadcast).
    pub tx_hash: Option<String>,
    pub status: PayoutStatus,
    /// Gas fee paid.
    pub fee_amount: Option<String>,
    /// Error message if failed.
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}
