//! Store wallet repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::StoreWallet;

/// Read operations for store wallets.
#[async_trait]
pub trait StoreWalletReader: Send + Sync {
    /// Get wallet configuration for a store.
    async fn get_wallet(&self, store_id: Uuid) -> RepositoryResult<Option<StoreWallet>>;

    /// Get a wallet by its ID.
    async fn get_wallet_by_id(&self, wallet_id: Uuid) -> RepositoryResult<Option<StoreWallet>>;
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
