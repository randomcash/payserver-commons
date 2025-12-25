//! Watched address repository traits.

use async_trait::async_trait;

use super::RepositoryResult;
use crate::types::{InvoiceId, Network};

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

/// Read operations for watched addresses.
#[async_trait]
pub trait WatchedAddressReader: Send + Sync {
    /// Get the invoice ID associated with an address.
    async fn get_invoice_id(
        &self,
        address: &str,
        network: Network,
    ) -> RepositoryResult<Option<InvoiceId>>;

    /// Get all active watched addresses.
    /// Returns tuples of (address, invoice_id, network).
    async fn get_active(&self) -> RepositoryResult<Vec<(String, InvoiceId, Network)>>;

    /// Get watched addresses pending notification to the monitor.
    async fn get_pending(&self) -> RepositoryResult<Vec<PendingWatchInfo>>;
}

/// Write operations for watched addresses.
#[async_trait]
pub trait WatchedAddressWriter: Send + Sync {
    /// Insert or update a watched address.
    async fn upsert(
        &self,
        address: &str,
        invoice_id: &InvoiceId,
        network: Network,
    ) -> RepositoryResult<()>;

    /// Insert or update a watched address with optional asset identifier.
    ///
    /// The `asset_id` is network-specific (e.g., token contract address for ERC20).
    async fn upsert_with_asset(
        &self,
        address: &str,
        invoice_id: &InvoiceId,
        network: Network,
        asset_id: Option<&str>,
    ) -> RepositoryResult<()>;

    /// Remove a watched address.
    async fn remove(&self, address: &str, network: Network) -> RepositoryResult<()>;

    /// Mark a watched address as notified to the monitor.
    async fn mark_notified(&self, address: &str, network: Network) -> RepositoryResult<()>;
}

/// Combined watched address repository with full read/write access.
pub trait WatchedAddressRepository: WatchedAddressReader + WatchedAddressWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: WatchedAddressReader + WatchedAddressWriter> WatchedAddressRepository for T {}

/// Cleanup operations for watched addresses.
///
/// These methods find addresses that need to be unwatched based on invoice status.
#[async_trait]
pub trait WatchedAddressCleanup: Send + Sync {
    /// Get addresses for expired invoices past the grace period.
    ///
    /// Returns addresses where:
    /// - The watched address is still active
    /// - The associated invoice has status 'expired'
    /// - The invoice's expires_at + grace_period has passed
    ///
    /// These addresses should be unwatched from the monitor.
    async fn get_expired_addresses_for_cleanup(
        &self,
        grace_period_secs: i64,
    ) -> RepositoryResult<Vec<CleanupAddressInfo>>;

    /// Get addresses for paid invoices that are still being watched.
    ///
    /// These addresses should be unwatched immediately since the invoice is complete.
    async fn get_paid_addresses_for_cleanup(&self) -> RepositoryResult<Vec<CleanupAddressInfo>>;

    /// Get addresses for cancelled invoices that are still being watched.
    ///
    /// These addresses should be unwatched immediately since the invoice is cancelled.
    async fn get_cancelled_addresses_for_cleanup(&self) -> RepositoryResult<Vec<CleanupAddressInfo>>;

    /// Deactivate a watched address (set is_active = false).
    ///
    /// Returns true if an address was deactivated, false if not found or already inactive.
    async fn deactivate_watched_address(
        &self,
        address: &str,
        network: Network,
    ) -> RepositoryResult<bool>;
}
