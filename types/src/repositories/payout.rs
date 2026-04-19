//! Payout repository traits.

use async_trait::async_trait;

use super::RepositoryResult;
use crate::store::StoreId;
use crate::types::{PayoutData, PayoutStatus};

/// Read operations for payouts.
#[async_trait]
pub trait PayoutReader: Send + Sync {
    /// Get a payout by ID.
    async fn get_payout(&self, id: uuid::Uuid) -> RepositoryResult<Option<PayoutData>>;

    /// Get all payouts for a store.
    async fn get_payouts_for_store(
        &self,
        store_id: StoreId,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<(i64, Vec<PayoutData>)>;

    /// Get all pending/broadcasting payouts that need monitoring.
    async fn get_active_payouts(&self) -> RepositoryResult<Vec<PayoutData>>;
}

/// Write operations for payouts.
#[async_trait]
pub trait PayoutWriter: Send + Sync {
    /// Create a new payout record.
    async fn create_payout(&self, payout: &PayoutData) -> RepositoryResult<()>;

    /// Update payout status.
    async fn update_payout_status(
        &self,
        id: uuid::Uuid,
        status: PayoutStatus,
        tx_hash: Option<&str>,
        fee_amount: Option<&str>,
        error_message: Option<&str>,
    ) -> RepositoryResult<()>;

    /// Mark a payout as confirmed.
    async fn confirm_payout(&self, id: uuid::Uuid) -> RepositoryResult<()>;
}

/// Combined payout repository.
pub trait PayoutRepository: PayoutReader + PayoutWriter {}

impl<T: PayoutReader + PayoutWriter> PayoutRepository for T {}
