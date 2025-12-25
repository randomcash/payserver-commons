//! Watched address types.

use super::Network;

/// Information about a watched address pending notification to the monitor.
#[derive(Debug, Clone)]
pub struct PendingWatchInfo {
    pub address: String,
    pub invoice_id: String,
    pub network: Network,
    pub expected_amount: Option<String>,
    /// Asset-specific identifier (e.g., token contract address for ERC20).
    pub asset_id: Option<String>,
}

/// Information about a watched address that needs to be cleaned up (unwatched).
#[derive(Debug, Clone)]
pub struct CleanupAddressInfo {
    pub address: String,
    pub network: Network,
    pub invoice_id: String,
}
