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
