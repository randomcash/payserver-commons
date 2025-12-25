//! Store wallet repository traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::RepositoryResult;

/// Store wallet configuration for payment address derivation.
#[derive(Debug, Clone)]
pub struct StoreWallet {
    pub id: Uuid,
    pub store_id: Uuid,
    pub xpub: String,
    pub derivation_index: i32,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Read operations for store wallets.
#[async_trait]
pub trait StoreWalletReader: Send + Sync {
    /// Get wallet configuration for a store.
    async fn get_wallet(&self, store_id: Uuid) -> RepositoryResult<Option<StoreWallet>>;
}

/// Write operations for store wallets.
#[async_trait]
pub trait StoreWalletWriter: Send + Sync {
    /// Create or update wallet configuration for a store.
    async fn upsert_wallet(
        &self,
        store_id: Uuid,
        xpub: &str,
        name: Option<&str>,
    ) -> RepositoryResult<StoreWallet>;

    /// Delete wallet configuration for a store.
    async fn delete_wallet(&self, store_id: Uuid) -> RepositoryResult<()>;

    /// Get and increment the derivation index for a store.
    ///
    /// Returns the current index before incrementing.
    async fn next_derivation_index(&self, store_id: Uuid) -> RepositoryResult<i32>;
}

/// Combined store wallet repository.
pub trait StoreWalletRepository: StoreWalletReader + StoreWalletWriter {}

impl<T: StoreWalletReader + StoreWalletWriter> StoreWalletRepository for T {}
