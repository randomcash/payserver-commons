//! Account wallets, and the per-store override that points at one.

use serde::{Deserialize, Serialize};

use crate::common::mask_xpub;
use types::{ChainId, NAMESPACE_EIP155, Wallet};
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The chain family a wallet is assumed to be for when a request does not say.
///
/// Every wallet registered before namespaces existed is an Ethereum one, and
/// every client written before them sends no namespace, so this is what those
/// requests mean. It is a compatibility default and nothing more: a merchant
/// registering a Tron key has to say `tron`, because the key itself cannot be
/// asked.
fn default_namespace() -> String {
    NAMESPACE_EIP155.to_string()
}

/// Request to add a wallet to the account.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    /// Extended public key (xpub) for address derivation.
    pub xpub: String,
    /// Optional wallet name.
    pub name: Option<String>,
    /// CAIP-2 namespace this key was exported for: `eip155`, `tron`, ... .
    /// Defaults to `eip155`.
    ///
    /// This has to be asked because it cannot be inferred. An account-level
    /// xpub has its BIP-44 coin type baked in and its parent unreachable, and
    /// an `m/44'/60'/0'` key is byte-identical in form to an `m/44'/195'/0'`
    /// one - same alphabet, same version bytes. Getting it wrong produces
    /// valid-looking addresses on the wrong family that the merchant's wallet
    /// never watches, which is why the response carries addresses to check.
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

/// Request to update a wallet.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWalletRequest {
    /// New name. Absent leaves the name alone.
    pub name: Option<String>,
    /// Set to `true` to make this the account primary. `false` is ignored:
    /// an account either has a primary or is choosing a different one, and
    /// "no primary" is not a state a merchant can usefully ask for.
    pub is_primary: Option<bool>,
}

/// Request to point a store at a wallet.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStoreWalletRequest {
    /// Wallet to use for this store. Must belong to the same account.
    pub wallet_id: Uuid,
}

/// Account wallet response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletResponse {
    /// Wallet ID.
    pub id: Uuid,
    /// Owning account.
    pub user_id: Uuid,
    /// CAIP-2 namespace this key derives for (`eip155`, `tron`, ...). Also
    /// decides the BIP-44 coin type in `derivation_path` and the encoding of
    /// every address shown for this wallet.
    pub namespace: String,
    /// Extended public key (masked for security).
    pub xpub_masked: String,
    /// Next derivation index this wallet will issue.
    pub derivation_index: i32,
    /// Wallet name.
    pub name: Option<String>,
    /// Whether stores on this wallet's chain family fall back to it. Scoped
    /// per family: an account can have one primary per namespace.
    pub is_primary: bool,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response to adding a wallet: the wallet, and addresses to check it with.
///
/// The addresses are the product requirement, not a convenience. Nothing about
/// an xpub says which BIP-44 coin type it was exported under, so a key pasted
/// into the wrong family is accepted, derives valid addresses, and fails
/// silently - the merchant's own wallet simply never shows the money. The only
/// defence available is for the merchant to compare the first few addresses
/// against their wallet before an invoice is ever quoted against this key, so
/// creating a wallet hands them straight back.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletResponse {
    /// The wallet, as `GET /wallets` would report it.
    #[serde(flatten)]
    pub wallet: WalletResponse,
    /// The first addresses this key derives, for the merchant to check against
    /// their own wallet before it is used. Returned on every create, including
    /// one that matched an existing wallet, because "did I already have this
    /// namespace?" is not a question the caller should have to answer to know
    /// whether the check is available.
    pub verification_addresses: Vec<DerivedAddressEntry>,
}

/// The wallet a store derives from, and how it got there.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreWalletResponse {
    /// Store ID.
    pub store_id: Uuid,
    /// The wallet this store's addresses come from.
    #[serde(flatten)]
    pub wallet: WalletResponse,
    /// True when the store is pinned to this wallet, false when it is simply
    /// following the account primary. The distinction is what tells a merchant
    /// whether changing their primary will move this store's payouts.
    pub is_override: bool,
}

/// Wallet xpub export response (full, unmasked).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletXpubResponse {
    /// Wallet ID.
    pub id: Uuid,
    /// Owning account.
    pub user_id: Uuid,
    /// CAIP-2 namespace this key derives for. Part of the export: the coin
    /// type is baked into the key but not readable from it, so this is what
    /// says which wallet application the xpub belongs in.
    pub namespace: String,
    /// Full extended public key (unmasked).
    pub xpub: String,
    /// Next derivation index this wallet will issue.
    pub derivation_index: i32,
    /// Wallet name.
    pub name: Option<String>,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A derived wallet address with its index and derivation path.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedAddressEntry {
    /// The derived address, in its family's own encoding: EIP-55 checksummed
    /// hex for `eip155`, base58check `T...` for `tron`.
    pub address: String,
    /// BIP-44 derivation index.
    pub index: u32,
    /// Full BIP-44 derivation path, coin type included - `m/44'/60'/0'/0/3`
    /// for Ethereum, `m/44'/195'/0'/0/3` for Tron. The coin type is the part
    /// worth reading: it is what the merchant's wallet has to agree with.
    pub derivation_path: String,
    /// Whether this index has been assigned to a payment option.
    pub used: bool,
}

/// Response for listing derived wallet addresses.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddressesResponse {
    /// Wallet ID.
    pub wallet_id: Uuid,
    /// Next derivation index (number of addresses assigned so far).
    pub derivation_index: i32,
    /// Derived addresses.
    pub addresses: Vec<DerivedAddressEntry>,
}

/// Request to rotate wallet xpub.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateWalletRequest {
    /// New extended public key to rotate to.
    pub xpub: String,
    /// Optional reason for rotation (e.g., "key compromise", "scheduled rotation").
    pub reason: Option<String>,
    /// CAIP-2 namespace of the key, and so of the payment methods that move.
    /// Defaults to `eip155`.
    ///
    /// Rotation is a response to a compromised key, and a key belongs to one
    /// chain family. Without this, rotating a store onto a new Ethereum xpub
    /// would repoint its Tron methods at that key too - deriving their
    /// addresses at coin type 60, on a chain whose wallets look under 195.
    /// Methods on other families are left exactly where they were.
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

/// A single rotation event in the response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationEntry {
    /// Rotation ID.
    pub id: Uuid,
    /// Payment method that was rotated.
    pub payment_method_id: Uuid,
    /// Chain ID of the rotated payment method.
    /// Chain the rotated method was on.
    ///
    /// Optional because the rotation record does not carry it - it is looked up
    /// from a snapshot of the store's methods, and a method created between
    /// that snapshot and the rotation has no entry. The rotation still happened
    /// and is still identified by `payment_method_id`; only the label is
    /// missing.
    pub chain_id: Option<ChainId>,
    /// Asset symbol of the rotated payment method.
    /// Asset the rotated method was for.
    ///
    /// Optional for the same reason as `chain_id` above: both are labels looked
    /// up from a snapshot of the store's methods, and both are absent for the
    /// same method. Representing one as `null` and the other as `""` would be
    /// two answers to one question in a single payload.
    pub asset_symbol: Option<String>,
    /// Previous xpub (masked).
    pub previous_xpub_masked: String,
    /// Derivation index at time of rotation.
    pub previous_derivation_index: i32,
    /// When the rotation occurred.
    pub rotated_at: chrono::DateTime<chrono::Utc>,
}

/// Response from wallet rotation.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateWalletResponse {
    /// Store ID.
    pub store_id: Uuid,
    /// New xpub (masked).
    pub new_xpub_masked: String,
    /// Number of payment methods rotated.
    pub methods_rotated: usize,
    /// Individual rotation entries.
    pub rotations: Vec<RotationEntry>,
}

impl From<Wallet> for WalletResponse {
    fn from(w: Wallet) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            namespace: w.namespace,
            xpub_masked: mask_xpub(&w.xpub),
            derivation_index: w.derivation_index,
            name: w.name,
            is_primary: w.is_primary,
            created_at: w.created_at,
        }
    }
}
