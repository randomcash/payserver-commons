//! Live watched address repository traits.
//!
//! These traits are for runtime address watching in evmmonitor.
//! They provide `(chain_id, address, token_address) -> invoice_id` mappings
//! for fast lookups during transaction monitoring.
//!
//! The token_address distinguishes between native assets (None) and ERC20 tokens (Some).
//! This allows the same receiving address to be watched for different tokens.
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
    ///
    /// `token_address` should be None for native assets, Some for ERC20 tokens.
    async fn get_watched_invoice(
        &self,
        address: &str,
        chain_id: u64,
        token_address: Option<&str>,
    ) -> RepositoryResult<Option<InvoiceId>>;

    /// Get all currently watched addresses.
    ///
    /// Returns tuples of (address, invoice_id, chain_id, token_address).
    async fn get_all_watched(&self) -> RepositoryResult<Vec<(String, InvoiceId, u64, Option<String>)>>;
}

/// Write operations for live watched addresses.
///
/// Used by evmmonitor to add/remove addresses from the watch list.
#[async_trait]
pub trait LiveWatchedAddressWriter: Send + Sync {
    /// Start watching an address for payments.
    ///
    /// `token_address` should be None for native assets, Some for ERC20 tokens.
    async fn watch_address(
        &self,
        address: &str,
        invoice_id: &InvoiceId,
        chain_id: u64,
        token_address: Option<&str>,
    ) -> RepositoryResult<()>;

    /// Stop watching an address.
    ///
    /// `token_address` should be None for native assets, Some for ERC20 tokens.
    /// Returns true if the address was being watched, false otherwise.
    async fn unwatch_address(
        &self,
        address: &str,
        chain_id: u64,
        token_address: Option<&str>,
    ) -> RepositoryResult<bool>;
}

/// Combined live watched address repository with full read/write access.
pub trait LiveWatchedAddressRepository: LiveWatchedAddressReader + LiveWatchedAddressWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: LiveWatchedAddressReader + LiveWatchedAddressWriter> LiveWatchedAddressRepository for T {}
