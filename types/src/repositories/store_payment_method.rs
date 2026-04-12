//! Store payment method repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::StorePaymentMethod;

/// Read operations for store payment methods.
#[async_trait]
pub trait StorePaymentMethodReader: Send + Sync {
    /// Get all payment methods for a store.
    async fn get_payment_methods(
        &self,
        store_id: Uuid,
    ) -> RepositoryResult<Vec<StorePaymentMethod>>;

    /// Get enabled payment methods for a store.
    async fn get_enabled_payment_methods(
        &self,
        store_id: Uuid,
    ) -> RepositoryResult<Vec<StorePaymentMethod>>;

    /// Get a specific payment method by ID.
    async fn get_payment_method(&self, id: Uuid) -> RepositoryResult<Option<StorePaymentMethod>>;

    /// Get payment method by store, chain, and token address.
    async fn get_payment_method_by_chain(
        &self,
        store_id: Uuid,
        chain_id: u64,
        token_address: Option<&str>,
    ) -> RepositoryResult<Option<StorePaymentMethod>>;

    /// Find payment methods matching a currency/asset symbol.
    async fn find_by_asset_symbol(
        &self,
        store_id: Uuid,
        asset_symbol: &str,
    ) -> RepositoryResult<Vec<StorePaymentMethod>>;
}

/// Write operations for store payment methods.
#[async_trait]
pub trait StorePaymentMethodWriter: Send + Sync {
    /// Create a new payment method for a store.
    async fn create_payment_method(
        &self,
        store_id: Uuid,
        chain_id: u64,
        token_address: Option<&str>,
        asset_symbol: &str,
        decimals: u8,
        xpub: &str,
    ) -> RepositoryResult<StorePaymentMethod>;

    /// Update a payment method (enable/disable, change xpub).
    async fn update_payment_method(
        &self,
        id: Uuid,
        enabled: Option<bool>,
        xpub: Option<&str>,
    ) -> RepositoryResult<StorePaymentMethod>;

    /// Delete a payment method.
    async fn delete_payment_method(&self, id: Uuid) -> RepositoryResult<()>;

    /// Get and increment the derivation index for a payment method.
    ///
    /// Returns the current index before incrementing.
    async fn next_derivation_index(&self, id: Uuid) -> RepositoryResult<i32>;
}

/// Combined store payment method repository.
pub trait StorePaymentMethodRepository:
    StorePaymentMethodReader + StorePaymentMethodWriter
{
}

impl<T: StorePaymentMethodReader + StorePaymentMethodWriter> StorePaymentMethodRepository for T {}
