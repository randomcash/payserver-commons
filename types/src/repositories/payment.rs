//! Payment repository traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::RepositoryResult;
use crate::store::StoreId;
use crate::traits::PaymentData;
use crate::types::InvoiceId;

/// Query parameters for listing payments.
#[derive(Debug, Clone, Default)]
pub struct PaymentQueryParams {
    pub store_id: Option<StoreId>,
    pub invoice_id: Option<InvoiceId>,
    pub confirmed: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

impl PaymentQueryParams {
    pub fn new() -> Self {
        Self {
            store_id: None,
            invoice_id: None,
            confirmed: None,
            limit: 50,
            offset: 0,
        }
    }

    pub fn with_store_id(mut self, store_id: StoreId) -> Self {
        self.store_id = Some(store_id);
        self
    }

    pub fn with_invoice_id(mut self, invoice_id: InvoiceId) -> Self {
        self.invoice_id = Some(invoice_id);
        self
    }

    pub fn with_confirmed(mut self, confirmed: bool) -> Self {
        self.confirmed = Some(confirmed);
        self
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
}

/// Read operations for payments.
#[async_trait]
pub trait PaymentReader: Send + Sync {
    /// Get a payment by ID.
    async fn get(&self, id: Uuid) -> RepositoryResult<Option<PaymentData>>;

    /// Get all payments for an invoice.
    async fn get_for_invoice(&self, invoice_id: &InvoiceId) -> RepositoryResult<Vec<PaymentData>>;

    /// Get payments awaiting confirmation (confirmed_at is NULL, not reorged).
    async fn get_awaiting_confirmation(&self) -> RepositoryResult<Vec<PaymentData>>;

    /// Get valid (non-reorged) payments for an invoice.
    async fn get_valid_for_invoice(&self, invoice_id: &InvoiceId) -> RepositoryResult<Vec<PaymentData>>;

    /// Check if an invoice has any valid (non-reorged) payments.
    async fn has_valid_payments(&self, invoice_id: &InvoiceId) -> RepositoryResult<bool>;

    /// Query payments with filters and pagination.
    async fn query(&self, params: &PaymentQueryParams) -> RepositoryResult<(i64, Vec<PaymentData>)>;
}

/// Write operations for payments.
#[async_trait]
pub trait PaymentWriter: Send + Sync {
    /// Insert or update a payment.
    async fn upsert(&self, payment: &PaymentData) -> RepositoryResult<()>;

    /// Mark a payment as confirmed by setting the confirmed_at timestamp.
    async fn mark_confirmed(&self, id: Uuid, confirmed_at: DateTime<Utc>) -> RepositoryResult<()>;

    /// Mark payments as reorged for a specific invoice, chain, and fork block.
    ///
    /// Only marks payments where:
    /// - invoice_id matches
    /// - chain_id matches
    /// - block_number >= fork_block (affected by the reorg)
    /// - not already reorged
    ///
    /// Returns the number of payments marked as reorged.
    async fn mark_reorged(
        &self,
        invoice_id: &InvoiceId,
        chain_id: u64,
        fork_block: u64,
    ) -> RepositoryResult<u64>;
}

/// Combined payment repository with full read/write access.
pub trait PaymentRepository: PaymentReader + PaymentWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: PaymentReader + PaymentWriter> PaymentRepository for T {}
