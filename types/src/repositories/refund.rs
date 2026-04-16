//! Refund repository traits.

use async_trait::async_trait;

use super::RepositoryResult;
use crate::store::StoreId;
use crate::types::{InvoiceId, RefundData, RefundStatus};

/// Read operations for refunds.
#[async_trait]
pub trait RefundReader: Send + Sync {
    /// Get a refund by ID.
    async fn get_refund(&self, id: uuid::Uuid) -> RepositoryResult<Option<RefundData>>;

    /// Get all refunds for an invoice.
    async fn get_refunds_for_invoice(
        &self,
        invoice_id: &InvoiceId,
    ) -> RepositoryResult<Vec<RefundData>>;

    /// Get all refunds for a store.
    async fn get_refunds_for_store(
        &self,
        store_id: StoreId,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<(i64, Vec<RefundData>)>;

    /// Get all pending/broadcasting refunds that need monitoring.
    async fn get_active_refunds(&self) -> RepositoryResult<Vec<RefundData>>;
}

/// Write operations for refunds.
#[async_trait]
pub trait RefundWriter: Send + Sync {
    /// Create a new refund record.
    async fn create_refund(&self, refund: &RefundData) -> RepositoryResult<()>;

    /// Update refund status.
    async fn update_refund_status(
        &self,
        id: uuid::Uuid,
        status: RefundStatus,
        tx_hash: Option<&str>,
        fee_amount: Option<&str>,
        error_message: Option<&str>,
    ) -> RepositoryResult<()>;

    /// Mark a refund as confirmed.
    async fn confirm_refund(&self, id: uuid::Uuid) -> RepositoryResult<()>;
}

/// Combined refund repository.
pub trait RefundRepository: RefundReader + RefundWriter {}

impl<T: RefundReader + RefundWriter> RefundRepository for T {}
