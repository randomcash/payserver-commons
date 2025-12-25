//! Payment repository traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::RepositoryResult;
use crate::traits::PaymentData;
use crate::types::InvoiceId;

/// Query parameters for listing payments.
#[derive(Debug, Clone, Default)]
pub struct PaymentQueryParams {
    pub invoice_id: Option<InvoiceId>,
    pub min_confirmations: Option<u32>,
    pub limit: i64,
    pub offset: i64,
}

/// Read operations for payments.
#[async_trait]
pub trait PaymentReader: Send + Sync {
    /// Get a payment by ID.
    async fn get(&self, id: Uuid) -> RepositoryResult<Option<PaymentData>>;

    /// Get all payments for an invoice.
    async fn get_for_invoice(&self, invoice_id: &InvoiceId) -> RepositoryResult<Vec<PaymentData>>;

    /// Get payments with fewer than N confirmations (for monitoring).
    async fn get_unconfirmed(&self, min_confirmations: u32) -> RepositoryResult<Vec<PaymentData>>;

    /// Get valid (non-reorged) payments for an invoice.
    async fn get_valid_for_invoice(&self, invoice_id: &InvoiceId) -> RepositoryResult<Vec<PaymentData>>;

    /// Check if an invoice has any valid (non-reorged) payments.
    async fn has_valid_payments(&self, invoice_id: &InvoiceId) -> RepositoryResult<bool>;
}

/// Write operations for payments.
#[async_trait]
pub trait PaymentWriter: Send + Sync {
    /// Insert or update a payment.
    async fn upsert(&self, payment: &PaymentData) -> RepositoryResult<()>;

    /// Update payment confirmations.
    async fn update_confirmations(
        &self,
        id: Uuid,
        confirmations: u32,
        confirmed_at: Option<DateTime<Utc>>,
    ) -> RepositoryResult<()>;

    /// Mark payments as reorged for a specific invoice, network, and fork block.
    ///
    /// Only marks payments where:
    /// - invoice_id matches
    /// - network matches
    /// - block_number >= fork_block (affected by the reorg)
    /// - not already reorged
    ///
    /// Returns the number of payments marked as reorged.
    async fn mark_reorged(
        &self,
        invoice_id: &InvoiceId,
        network: crate::Network,
        fork_block: u64,
    ) -> RepositoryResult<u64>;
}

/// Combined payment repository with full read/write access.
pub trait PaymentRepository: PaymentReader + PaymentWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: PaymentReader + PaymentWriter> PaymentRepository for T {}
