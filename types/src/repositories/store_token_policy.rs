//! Store token policy repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::{StoreTokenPolicyWithEntries, TokenPolicyMode};

/// Read operations for store token policies.
#[async_trait]
pub trait StoreTokenPolicyReader: Send + Sync {
    /// Get the token policy for a store, including all entries.
    /// Returns None if no policy is configured (accept-all default).
    async fn get_token_policy(
        &self,
        store_id: Uuid,
    ) -> RepositoryResult<Option<StoreTokenPolicyWithEntries>>;
}

/// Input entry for creating/updating a token policy.
pub struct TokenPolicyEntryInput {
    pub chain_id: i64,
    pub token_address: Option<String>,
    pub asset_symbol: String,
}

/// Write operations for store token policies.
#[async_trait]
pub trait StoreTokenPolicyWriter: Send + Sync {
    /// Create or replace the token policy for a store.
    ///
    /// Atomically replaces any existing policy and all its entries.
    async fn upsert_token_policy(
        &self,
        store_id: Uuid,
        mode: TokenPolicyMode,
        entries: &[TokenPolicyEntryInput],
    ) -> RepositoryResult<StoreTokenPolicyWithEntries>;

    /// Delete the token policy for a store, reverting to accept-all.
    async fn delete_token_policy(&self, store_id: Uuid) -> RepositoryResult<()>;
}

/// Combined store token policy repository.
pub trait StoreTokenPolicyRepository: StoreTokenPolicyReader + StoreTokenPolicyWriter {}

impl<T: StoreTokenPolicyReader + StoreTokenPolicyWriter> StoreTokenPolicyRepository for T {}
