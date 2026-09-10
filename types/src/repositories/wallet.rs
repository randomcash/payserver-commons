//! Account wallet repository traits.
//!
//! Replaces the old per-store `StoreWalletReader`/`StoreWalletWriter`, which
//! kept an xpub and a counter on each store. Two stores handed the same xpub
//! each counted from zero and derived the same addresses; putting the counter
//! on the wallet makes that arithmetically impossible rather than merely
//! discouraged.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::Wallet;

/// Read operations for account wallets.
#[async_trait]
pub trait WalletReader: Send + Sync {
    /// Get a wallet by its id.
    async fn get_wallet(&self, wallet_id: Uuid) -> RepositoryResult<Option<Wallet>>;

    /// Every wallet on an account, primary first, then oldest first.
    async fn list_wallets(&self, user_id: Uuid) -> RepositoryResult<Vec<Wallet>>;

    /// The account's primary wallet, if it has one.
    async fn get_primary_wallet(&self, user_id: Uuid) -> RepositoryResult<Option<Wallet>>;

    /// The wallet a store derives from: its own override if it has one,
    /// otherwise the owner's primary.
    ///
    /// `None` means the account has no wallet at all - a store cannot be paid
    /// until one exists. The two-step fallback is the whole store-resolution
    /// rule, and it lives here so no caller can spell it differently.
    async fn resolve_store_wallet(&self, store_id: Uuid) -> RepositoryResult<Option<Wallet>>;

    /// The wallet id a store has been explicitly pinned to, if any.
    ///
    /// Separate from `resolve_store_wallet` because callers that render
    /// settings need to tell "pinned to the wallet that happens to be primary"
    /// from "following the primary".
    async fn get_store_wallet_override(&self, store_id: Uuid) -> RepositoryResult<Option<Uuid>>;
}

/// Write operations for account wallets.
#[async_trait]
pub trait WalletWriter: Send + Sync {
    /// Add a wallet to an account.
    ///
    /// Re-registering an xpub the account already holds returns the existing
    /// row rather than making a second one: a second row would be a second
    /// counter on the same key, which is the collision this module exists to
    /// prevent. The first wallet on an account becomes its primary.
    async fn create_wallet(
        &self,
        user_id: Uuid,
        xpub: &str,
        name: Option<&str>,
    ) -> RepositoryResult<Wallet>;

    /// Rename a wallet.
    async fn rename_wallet(&self, wallet_id: Uuid, name: Option<&str>) -> RepositoryResult<Wallet>;

    /// Make `wallet_id` the account's primary, demoting the previous one.
    ///
    /// Both halves happen in one transaction, demotion first, because the
    /// partial unique index refuses to hold two primaries even momentarily.
    async fn set_primary_wallet(&self, user_id: Uuid, wallet_id: Uuid) -> RepositoryResult<Wallet>;

    /// Delete a wallet.
    ///
    /// Refused while a payment method or a store override still points at it:
    /// the addresses it derived are still being watched, and losing the xpub
    /// loses the ability to say which key they came from.
    async fn delete_wallet(&self, wallet_id: Uuid) -> RepositoryResult<()>;

    /// Pin a store to a specific wallet, overriding the account primary.
    async fn set_store_wallet(&self, store_id: Uuid, wallet_id: Uuid) -> RepositoryResult<()>;

    /// Drop a store's override so it follows the account primary again.
    async fn clear_store_wallet(&self, store_id: Uuid) -> RepositoryResult<()>;

    /// Take the next derivation index for a wallet, advancing the counter.
    ///
    /// Returns the index to derive with; the stored counter is left pointing
    /// one past it. Must be atomic: two invoices allocating concurrently on
    /// the same wallet have to receive different indices, or they are handed
    /// the same address.
    async fn next_derivation_index(&self, wallet_id: Uuid) -> RepositoryResult<i32>;
}

/// Combined account wallet repository.
pub trait WalletRepository: WalletReader + WalletWriter {}

impl<T: WalletReader + WalletWriter> WalletRepository for T {}
