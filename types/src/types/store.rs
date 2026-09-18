//! Store-related types.

use super::ChainId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// An account-level wallet: one extended public key, the chain family that key
/// belongs to, and the one derivation counter that belongs to both.
///
/// Distinct from `auth::WalletCredential`, which is a wallet used to *log in*.
/// This is the one money arrives at.
///
/// The xpub and the counter live in the same row and nowhere else, and that
/// pairing is the point. An xpub reachable through two counters derives the
/// same address twice: derivation is `m/44'/<coin>'/0'/0/{i}`, with no store,
/// chain or asset in the path, so two counters that both reach index 5 produce
/// byte-identical addresses and two merchants' payments land on one address.
/// Everything that needs an address therefore asks a wallet row for the next
/// index rather than keeping a count beside it.
///
/// `namespace` is the other half of that identity, and it is not cosmetic. An
/// account-level xpub has its BIP-44 coin type already baked into it and its
/// parent is unreachable, so the family a key was exported for cannot be
/// recovered from the key: `m/44'/60'/0'` (Ethereum) and `m/44'/195'/0'`
/// (Tron) are byte-indistinguishable - same base58 alphabet, same version
/// bytes. Running an Ethereum account xpub through Tron's address encoding
/// yields a valid, checksum-correct `T...` address that the merchant's Tron
/// wallet will never show, because that wallet looks under coin type 195. The
/// funds are then reachable only by re-importing the seed at a non-standard
/// path. Nothing can detect that after the fact, so the family is recorded
/// when the key is registered and every resolution is scoped by it.
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: Uuid,
    /// Owning account. Wallets belong to a user, not to a store.
    pub user_id: Uuid,
    /// The CAIP-2 namespace this key derives for - `eip155`, `tron`, ... .
    ///
    /// Decides both the BIP-44 coin type the merchant's wallet used to export
    /// the key and the encoding the derived address is rendered in. A wallet
    /// only ever serves payment methods whose chain is in this namespace.
    pub namespace: String,
    /// BIP-32 extended public key, at account level (`m/44'/<coin>'/0'`, with
    /// `<coin>` fixed by [`Self::namespace`]).
    pub xpub: String,
    /// Next derivation index to issue. Moved only by
    /// `WalletWriter::next_derivation_index`, which reads and advances it in
    /// one statement.
    pub derivation_index: i32,
    pub name: Option<String>,
    /// The wallet a store falls back to when it has no override of its own.
    /// At most one per (user, namespace), enforced by a partial unique index
    /// in the schema rather than by application code. Per namespace, because
    /// an account paid on two families needs one fallback in each and a single
    /// primary would force one of them onto the wrong key.
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

/// Store payment method configuration.
///
/// Represents a (chain, asset, wallet) combination that a store accepts for payment.
/// Each store can have multiple payment methods across different chains and assets.
#[derive(Debug, Clone)]
pub struct StorePaymentMethod {
    pub id: Uuid,
    pub store_id: Uuid,
    /// EIP-155 chain ID (1=Ethereum, 137=Polygon, 11155111=Sepolia, etc.)
    pub chain_id: ChainId,
    /// ERC20 token contract address, None for native asset.
    pub token_address: Option<String>,
    /// Asset symbol for display (ETH, USDC, etc.)
    pub asset_symbol: String,
    /// Number of decimals for this asset (18 for ETH, 6 for USDC/USDT).
    pub decimals: u8,
    /// The wallet this method actually derives from, after resolution.
    ///
    /// Resolved, not stored: a method may be pinned to a wallet, and otherwise
    /// follows its store's override, and otherwise the account primary. What
    /// is reported here is the answer that derivation will reach, so a caller
    /// cannot render one key while payments are collected on another.
    ///
    /// Every step of that walk is filtered to `chain_id`'s CAIP-2 namespace. A
    /// wallet in another family is not a worse answer than the right one, it
    /// is a wrong one: its key was exported under a different BIP-44 coin type
    /// and the address derived from it is unreachable from the merchant's
    /// wallet.
    ///
    /// `None` means the chain runs out - no pin, no store override, no account
    /// primary *in this namespace*. The method exists but cannot be paid until
    /// a wallet for its family does.
    pub wallet_id: Option<Uuid>,
    /// The resolved wallet's xpub. `None` for the same reason as `wallet_id`.
    ///
    /// Read-only: the column lives on `wallets`, so methods sharing a wallet
    /// cannot drift apart. Do not snapshot this and then allocate an index
    /// separately - a rotation landing in between pairs one wallet's key with
    /// another's index. Use `allocate_derivation`, which returns both from one
    /// statement.
    pub xpub: Option<String>,
    /// The resolved wallet's next derivation index, shared with every other
    /// method resolving to the same wallet.
    pub derivation_index: Option<i32>,
    /// Whether this payment method is enabled.
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// One derivation slot, taken atomically.
///
/// The key and the index come out of the same statement against the same
/// wallet row. Fetching them separately is a duplicate-address bug: a rotation
/// committing between the two pairs the old wallet's xpub with an index
/// consumed from the new one, and the old wallet never advances past it, so it
/// hands that address out again later.
#[derive(Debug, Clone)]
pub struct DerivationAllocation {
    /// The wallet the index was taken from.
    pub wallet_id: Uuid,
    /// That wallet's chain family, from the same row as the key.
    ///
    /// Carried rather than re-derived at the call site for the same reason the
    /// xpub is: it fixes the coin type the key was exported under, and so the
    /// encoding the address must be rendered in. A caller that paired this
    /// xpub with a namespace read from somewhere else could render an
    /// Ethereum key as a Tron address, which is valid, checksum-correct and
    /// unspendable from the merchant's wallet.
    pub namespace: String,
    /// That wallet's xpub - the one to derive with, not a snapshot.
    pub xpub: String,
    /// The index to derive at. Already consumed; nobody else will get it.
    pub index: i32,
}

/// Store webhook configuration for invoice notifications.
#[derive(Debug, Clone)]
pub struct StoreWebhook {
    pub id: Uuid,
    pub store_id: Uuid,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Store-level settings for defaults, branding, and notification preferences.
#[derive(Debug, Clone)]
pub struct StoreSettings {
    pub store_id: Uuid,
    /// Default chain ID for new invoices (null = no default).
    /// Chain this store quotes by default, as a CAIP-2 identifier.
    ///
    /// Was `Option<i64>` of an EIP-155 number. The column became the `caip2`
    /// domain but this did not follow, so the repository was reading
    /// and binding a TEXT column as `i64` - which compiles, because sqlx is
    /// checked at runtime, and fails the first time a store actually sets one.
    pub default_chain_id: Option<ChainId>,
    /// Default fiat display currency (e.g. "USD").
    pub default_display_currency: Option<String>,
    /// Logo URL for checkout branding.
    pub logo_url: Option<String>,
    /// Accent color hex (e.g. "#FF5500").
    pub accent_color: Option<String>,
    /// Per-event notification preferences: `{"event_name": {"webhook": bool}}`.
    pub notification_prefs: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Record of a single webhook delivery attempt.
#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub store_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub http_status: Option<i16>,
    pub response_body: Option<String>,
    pub latency_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub attempt_number: i32,
    pub created_at: DateTime<Utc>,
}
