//! Account wallets, and the per-store override that points at one.

use serde::{Deserialize, Serialize};
use types::ChainId;
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Request to add a wallet to the account.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    /// Extended public key (xpub) for address derivation.
    pub xpub: String,
    /// Optional wallet name.
    pub name: Option<String>,
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
    /// Extended public key (masked for security).
    pub xpub_masked: String,
    /// Next derivation index this wallet will issue.
    pub derivation_index: i32,
    /// Wallet name.
    pub name: Option<String>,
    /// Whether stores fall back to this wallet.
    pub is_primary: bool,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
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
    /// Ethereum address (checksummed hex).
    pub address: String,
    /// BIP-44 derivation index.
    pub index: u32,
    /// Full BIP-44 derivation path.
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
    pub chain_id: ChainId,
    /// Asset symbol of the rotated payment method.
    pub asset_symbol: String,
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
