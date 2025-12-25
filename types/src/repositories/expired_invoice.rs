//! Expired invoice streaming repository traits.

use async_trait::async_trait;
use futures::stream::BoxStream;

use super::RepositoryResult;
use crate::{InvoiceId, Network};

/// Streaming operations for expired invoices.
///
/// This trait provides efficient streaming access to expired invoices
/// for batch processing without loading all invoices into memory.
#[async_trait]
pub trait ExpiredInvoiceStreamer: Send + Sync {
    /// Stream expired pending invoice IDs for a specific network.
    ///
    /// Only returns invoices where:
    /// - status = 'pending' (no payments detected)
    /// - network matches
    /// - expires_at < NOW()
    ///
    /// Returns a stream of invoice IDs for minimal memory usage.
    fn stream_expired_pending_for_network(
        &self,
        network: Network,
    ) -> BoxStream<'_, RepositoryResult<InvoiceId>>;

    /// Stream all expired pending invoice IDs across all networks.
    ///
    /// Only returns invoices where:
    /// - status = 'pending' (no payments detected)
    /// - expires_at < NOW()
    ///
    /// Returns a stream of (network, invoice_id) for minimal memory usage.
    fn stream_all_expired_pending(&self) -> BoxStream<'_, RepositoryResult<(Network, InvoiceId)>>;
}
