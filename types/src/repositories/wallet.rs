//! Account wallet repository traits.
//!
//! Replaces the old per-store `StoreWalletReader`/`StoreWalletWriter`, which
//! kept an xpub and a counter on each store. Two stores handed the same xpub
//! each counted from zero and derived the same addresses; putting the counter
//! on the wallet makes that arithmetically impossible rather than merely
//! discouraged.
//!
//! Every lookup here is scoped by a CAIP-2 namespace, and that is a safety
//! property rather than a filter. A wallet holds an account-level xpub with
//! its BIP-44 coin type already baked in - 60 for Ethereum, 195 for Tron -
//! and the two are byte-indistinguishable, so nothing downstream can tell
//! which family a key came from. Resolving a Tron payment method onto an
//! Ethereum wallet produces a valid, checksum-correct `T...` address that the
//! merchant's own wallet does not watch, and the money that arrives there is
//! recoverable only by re-importing the seed at a non-standard path. So an
//! unanswerable namespace resolves to `None` - refused, loudly, at invoice
//! creation - rather than to the nearest wallet that happens to exist.

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

    /// The account's primary wallet for one chain family, if it has one.
    ///
    /// Scoped by namespace because a primary is a fallback, and an account
    /// collecting on two families needs one fallback in each. A single
    /// account-wide primary would hand every family the key of whichever one
    /// was registered first.
    async fn get_primary_wallet(
        &self,
        user_id: Uuid,
        namespace: &str,
    ) -> RepositoryResult<Option<Wallet>>;

    /// The wallet a store derives from for one chain family: its own override
    /// if that override is in this family, otherwise the owner's primary for
    /// this family.
    ///
    /// `None` means the account has no wallet for `namespace` - the store
    /// cannot be paid on that family until one exists. Refusing is the whole
    /// point: the alternative is deriving from a key exported under a
    /// different coin type, which yields an address the merchant's wallet will
    /// never show. The fallback walk is the whole store-resolution rule, and
    /// it lives here so no caller can spell it differently.
    async fn resolve_store_wallet(
        &self,
        store_id: Uuid,
        namespace: &str,
    ) -> RepositoryResult<Option<Wallet>>;

    /// The wallet id a store has been explicitly pinned to for one chain
    /// family, if any.
    ///
    /// Separate from [`Self::resolve_store_wallet`] because callers that
    /// render settings need to tell "pinned to the wallet that happens to be
    /// primary" from "following the primary".
    async fn get_store_wallet_override(
        &self,
        store_id: Uuid,
        namespace: &str,
    ) -> RepositoryResult<Option<Uuid>>;
}

/// Write operations for account wallets.
#[async_trait]
pub trait WalletWriter: Send + Sync {
    /// Add a wallet to an account, for one chain family.
    ///
    /// Re-registering an xpub the account already holds *in the same
    /// namespace* returns the existing row rather than making a second one: a
    /// second row would be a second counter on the same key, which is the
    /// collision this module exists to prevent.
    ///
    /// The same xpub in a *different* namespace is a different wallet, and
    /// deliberately so. The identity of a key here is (account, family, key):
    /// returning the Ethereum row to a merchant who asked to register the same
    /// bytes for Tron would silently answer a question they did not ask, and
    /// the answer would route their Tron income to an address only an
    /// Ethereum-path wallet can see. The two rows count independently, which
    /// is safe precisely because the addresses they derive live on different
    /// chains.
    ///
    /// The first wallet an account registers *in a namespace* becomes that
    /// namespace's primary, so a merchant with one wallet per family never has
    /// to meet the concept.
    async fn create_wallet(
        &self,
        user_id: Uuid,
        namespace: &str,
        xpub: &str,
        name: Option<&str>,
    ) -> RepositoryResult<Wallet>;

    /// Rename a wallet.
    async fn rename_wallet(&self, wallet_id: Uuid, name: Option<&str>) -> RepositoryResult<Wallet>;

    /// Make `wallet_id` the primary for its own chain family, demoting that
    /// family's previous primary.
    ///
    /// Scope is the namespace of the named wallet, taken from the row rather
    /// than from the caller: promoting a Tron wallet must not demote the
    /// account's Ethereum primary and leave every EVM store with nothing to
    /// derive from.
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

    /// Pin a store to a specific wallet, overriding the account primary for
    /// that wallet's chain family.
    ///
    /// A store holds one override per family, and this sets the one belonging
    /// to the named wallet's family - taken from its row, so the caller cannot
    /// file it under the wrong one. Pinning a Tron wallet says nothing about
    /// where the store's Ethereum payments go; a single override per store
    /// would have made it silently say something.
    async fn set_store_wallet(&self, store_id: Uuid, wallet_id: Uuid) -> RepositoryResult<()>;

    /// Drop a store's override for one chain family, so stores on it follow
    /// that family's account primary again.
    ///
    /// Scoped for the same reason [`Self::set_store_wallet`] is: "stop
    /// overriding for Tron" must not also stop overriding for Ethereum.
    async fn clear_store_wallet(&self, store_id: Uuid, namespace: &str) -> RepositoryResult<()>;

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
