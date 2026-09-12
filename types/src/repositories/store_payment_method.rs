//! Store payment method repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::ChainId;
use crate::types::{DerivationAllocation, StorePaymentMethod};

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
        chain_id: &ChainId,
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
    /// Add a payment method to a store.
    ///
    /// `xpub` is the extended **public** key to derive receive addresses from.
    /// `None` means "use the key this store already resolves to" - the store's
    /// own wallet, else the account primary - and the method is left unpinned
    /// so it follows that resolution afterwards. `Some` pins the method to that
    /// key, creating the account wallet if it is new.
    ///
    /// With `None` and nothing to resolve to, this is a [`RepositoryError::Conflict`]:
    /// the request is well-formed and the account state refuses it, which is
    /// the merchant's to fix by adding a key. Returning a database error there
    /// would tell them to retry something that can never succeed.
    async fn create_payment_method(
        &self,
        store_id: Uuid,
        chain_id: &ChainId,
        token_address: Option<&str>,
        asset_symbol: &str,
        decimals: u8,
        xpub: Option<&str>,
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

    /// Take the next derivation slot for a payment method.
    ///
    /// Resolves the wallet the method actually derives from - its own pin, its
    /// store's override, or the account primary - advances that wallet's
    /// counter and returns the key and index together.
    ///
    /// Replaces the old `next_derivation_index`, which returned an index alone
    /// and left the caller to pair it with an xpub it had read earlier. That
    /// pairing was only safe while the counter lived on the same row as the
    /// key; once it moved to the wallet, a rotation between the read and the
    /// allocation could combine two different wallets and re-issue an address.
    ///
    /// Must be atomic: two invoices allocating at once on one wallet have to
    /// receive different indices.
    async fn allocate_derivation(&self, id: Uuid) -> RepositoryResult<DerivationAllocation>;
}

/// Combined store payment method repository.
pub trait StorePaymentMethodRepository:
    StorePaymentMethodReader + StorePaymentMethodWriter
{
}

impl<T: StorePaymentMethodReader + StorePaymentMethodWriter> StorePaymentMethodRepository for T {}
