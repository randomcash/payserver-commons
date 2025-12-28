//! Watched address types.

use super::PaymentOptionId;

/// Information about a watched address pending notification to the monitor.
#[derive(Debug, Clone)]
pub struct PendingWatchInfo {
    /// Payment address being watched.
    pub address: String,
    /// Payment option this watch is for.
    pub payment_option_id: PaymentOptionId,
    /// Invoice ID for logging/tracking.
    pub invoice_id: String,
    /// EIP-155 chain ID.
    pub chain_id: u64,
    /// Expected amount in the asset's smallest unit.
    pub expected_amount: Option<String>,
    /// Token contract address (for ERC20, None for native).
    pub token_address: Option<String>,
}

/// Information about a watched address that needs to be cleaned up (unwatched).
#[derive(Debug, Clone)]
pub struct CleanupAddressInfo {
    /// Payment address to stop watching.
    pub address: String,
    /// Payment option this watch was for.
    pub payment_option_id: PaymentOptionId,
    /// Invoice ID for logging/tracking.
    pub invoice_id: String,
    /// EIP-155 chain ID.
    pub chain_id: u64,
    /// Token contract address (for ERC20, None for native).
    pub token_address: Option<String>,
}
