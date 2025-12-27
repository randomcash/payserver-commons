//! Live watched address repository traits.
//!
//! These traits are for runtime address watching in evmmonitor.
//! They provide simple `(chain_id, address) -> invoice_id` mappings
//! for fast lookups during transaction monitoring.
//!
//! Note: This is separate from [`WatchedAddressReader`]/[`WatchedAddressWriter`]
//! which handle full persistence with invoice metadata in PostgreSQL.

use async_trait::async_trait;

use super::RepositoryResult;
use crate::types::InvoiceId;

/// Read operations for live watched addresses.
///
/// Used by evmmonitor to check if an address is being watched.
#[async_trait]
pub trait LiveWatchedAddressReader: Send + Sync {
    /// Get the invoice ID associated with an address on a specific chain.
    async fn get_watched_invoice(
        &self,
        address: &str,
        chain_id: u64,
    ) -> RepositoryResult<Option<InvoiceId>>;

    /// Get all currently watched addresses.
    ///
    /// Returns tuples of (address, invoice_id, chain_id).
    async fn get_all_watched(&self) -> RepositoryResult<Vec<(String, InvoiceId, u64)>>;
}

/// Write operations for live watched addresses.
///
/// Used by evmmonitor to add/remove addresses from the watch list.
#[async_trait]
pub trait LiveWatchedAddressWriter: Send + Sync {
    /// Start watching an address for payments.
    async fn watch_address(
        &self,
        address: &str,
        invoice_id: &InvoiceId,
        chain_id: u64,
    ) -> RepositoryResult<()>;

    /// Stop watching an address.
    ///
    /// Returns true if the address was being watched, false otherwise.
    async fn unwatch_address(&self, address: &str, chain_id: u64) -> RepositoryResult<bool>;
}

/// Combined live watched address repository with full read/write access.
pub trait LiveWatchedAddressRepository: LiveWatchedAddressReader + LiveWatchedAddressWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: LiveWatchedAddressReader + LiveWatchedAddressWriter> LiveWatchedAddressRepository for T {}
