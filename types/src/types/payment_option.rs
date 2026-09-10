//! Payment option types.
//!
//! A PaymentOption represents one way to pay an invoice.
//! For example, an invoice priced at $100 USD might have payment options:
//! - ETH on Ethereum mainnet
//! - USDC on Polygon
//! - ETH on Sepolia (testnet)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ChainId, InvoiceId};

/// Unique identifier for a payment option.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentOptionId(pub Uuid);

impl PaymentOptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for PaymentOptionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PaymentOptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PaymentOptionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Payment method identifier.
///
/// Format: `{ASSET}@{CHAIN}`, where the chain is a CAIP-2 identifier.
///
/// Examples:
/// - `ETH@eip155:1` (ETH on Ethereum mainnet)
/// - `ETH@eip155:11155111` (ETH on Sepolia)
/// - `USDC@eip155:137` (USDC on Polygon)
/// - `USDT@tron:728126428` (USDT on Tron)
///
/// The separator is `@`, not the `-` this used to use. A CAIP-2 reference may
/// contain hyphens (`[-_a-zA-Z0-9]`, e.g. `cosmos:cosmoshub-3`), so splitting
/// on the last `-` would have taken the chain apart in the middle. `@` is in
/// neither the CAIP-2 charset nor any asset symbol, so the split is
/// unambiguous in both directions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentMethodId(pub String);

/// Separates asset from chain. See the note above on why it is not `-`.
const METHOD_ID_SEPARATOR: char = '@';

impl PaymentMethodId {
    /// Create a new payment method ID.
    pub fn new(asset_symbol: &str, chain_id: &ChainId) -> Self {
        Self(format!(
            "{}{METHOD_ID_SEPARATOR}{chain_id}",
            asset_symbol.to_uppercase()
        ))
    }

    /// Parse a payment method ID string.
    ///
    /// Splits on the FIRST separator: an asset symbol cannot contain one, and
    /// the remainder is handed to `ChainId` to validate rather than assumed
    /// well-formed.
    pub fn parse(s: &str) -> Option<(String, ChainId)> {
        let (asset, chain) = s.split_once(METHOD_ID_SEPARATOR)?;
        if asset.is_empty() {
            return None;
        }
        Some((asset.to_string(), ChainId::parse(chain).ok()?))
    }

    /// Get the asset symbol.
    pub fn asset_symbol(&self) -> Option<String> {
        Self::parse(&self.0).map(|(asset, _)| asset)
    }

    /// Get the chain.
    pub fn chain_id(&self) -> Option<ChainId> {
        Self::parse(&self.0).map(|(_, chain_id)| chain_id)
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PaymentMethodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PaymentMethodId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PaymentMethodId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A payment option for an invoice.
///
/// Represents one way to pay an invoice. An invoice can have multiple
/// payment options (e.g., pay with ETH on Ethereum or USDC on Polygon).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOptionData {
    /// Unique identifier.
    pub id: PaymentOptionId,
    /// The invoice this option belongs to.
    pub invoice_id: InvoiceId,
    /// Payment method identifier (e.g., "ETH-1", "USDC-137").
    pub payment_method_id: PaymentMethodId,
    /// EIP-155 chain ID.
    pub chain_id: ChainId,
    /// Asset symbol (e.g., "ETH", "USDC").
    pub asset_symbol: String,
    /// Token contract address (None for native assets).
    pub token_address: Option<String>,
    /// Number of decimals for this asset.
    pub decimals: u8,
    /// Payment destination address.
    pub payment_address: String,
    /// The account wallet whose xpub produced `payment_address`.
    ///
    /// `None` on options created before wallets moved to the account, whose
    /// originating method could not be matched back. Provenance only -
    /// `payment_address` is authoritative and has always been recorded
    /// literally, so nothing here is needed to know where an old invoice was
    /// to be paid.
    pub wallet_id: Option<Uuid>,
    /// The index used within `wallet_id`.
    ///
    /// `None` means the option predates account wallets and is genuinely
    /// unknown. Do not read it as 0: index 0 is a real address, and conflating
    /// the two would report a customer's address as belonging to an invoice
    /// that never used it.
    pub derivation_index: Option<i32>,
    /// Amount to pay in this asset (smallest unit as string).
    /// This is the invoice amount converted to this asset.
    pub amount: String,
    /// Exchange rate used for conversion (if applicable).
    /// Represents: 1 invoice_currency = rate asset_units
    pub rate: Option<String>,
    /// When the exchange rate was fetched.
    pub rate_at: Option<DateTime<Utc>>,
    /// Whether this payment option is active.
    pub is_active: bool,
    /// When this option was created.
    pub created_at: DateTime<Utc>,
}

impl PaymentOptionData {
    /// Create a new payment option for a native asset (ETH, POL, etc.).
    pub fn native(
        invoice_id: InvoiceId,
        chain_id: ChainId,
        asset_symbol: &str,
        payment_address: &str,
        amount: &str,
    ) -> Self {
        Self {
            id: PaymentOptionId::new(),
            invoice_id,
            payment_method_id: PaymentMethodId::new(asset_symbol, &chain_id),
            chain_id,
            asset_symbol: asset_symbol.to_string(),
            token_address: None,
            decimals: 18,
            payment_address: payment_address.to_string(),
            wallet_id: None,
            derivation_index: None,
            amount: amount.to_string(),
            rate: None,
            rate_at: None,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    /// Create a new payment option for an ERC20 token.
    pub fn erc20(
        invoice_id: InvoiceId,
        chain_id: ChainId,
        asset_symbol: &str,
        token_address: &str,
        decimals: u8,
        payment_address: &str,
        amount: &str,
    ) -> Self {
        Self {
            id: PaymentOptionId::new(),
            invoice_id,
            payment_method_id: PaymentMethodId::new(asset_symbol, &chain_id),
            chain_id,
            asset_symbol: asset_symbol.to_string(),
            token_address: Some(token_address.to_string()),
            decimals,
            payment_address: payment_address.to_string(),
            wallet_id: None,
            derivation_index: None,
            amount: amount.to_string(),
            rate: None,
            rate_at: None,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    /// Check if this is a native asset (not a token).
    pub fn is_native(&self) -> bool {
        self.token_address.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_id() {
        let id = PaymentMethodId::new("ETH", &ChainId::evm(1));
        assert_eq!(id.as_str(), "ETH@eip155:1");
        assert_eq!(id.asset_symbol(), Some("ETH".to_string()));
        assert_eq!(id.chain_id(), Some(ChainId::evm(1)));

        let id = PaymentMethodId::new("USDC", &ChainId::evm(137));
        assert_eq!(id.as_str(), "USDC@eip155:137");

        let id = PaymentMethodId::new("ETH", &ChainId::evm(11_155_111));
        assert_eq!(id.as_str(), "ETH@eip155:11155111");
        assert_eq!(id.chain_id(), Some(ChainId::evm(11_155_111)));
    }

    #[test]
    fn method_id_carries_non_evm_chains() {
        let tron = ChainId::parse("tron:728126428").unwrap();
        let id = PaymentMethodId::new("USDT", &tron);
        assert_eq!(id.as_str(), "USDT@tron:728126428");
        assert_eq!(id.chain_id(), Some(tron));
    }

    #[test]
    fn test_payment_method_id_parse() {
        let (asset, chain_id) = PaymentMethodId::parse("ETH@eip155:1").unwrap();
        assert_eq!(asset, "ETH");
        assert_eq!(chain_id, ChainId::evm(1));

        let (asset, chain_id) = PaymentMethodId::parse("USDC@eip155:137").unwrap();
        assert_eq!(asset, "USDC");
        assert_eq!(chain_id, ChainId::evm(137));

        // An asset symbol may contain a hyphen; it no longer collides with the
        // separator, which is why the separator changed.
        let (asset, chain_id) = PaymentMethodId::parse("WETH-USDC@eip155:1").unwrap();
        assert_eq!(asset, "WETH-USDC");
        assert_eq!(chain_id, ChainId::evm(1));
    }

    /// A CAIP-2 reference may contain hyphens, which is precisely what the old
    /// `{ASSET}-{CHAIN}` format could not survive: splitting `ATOM-cosmos:cosmoshub-3`
    /// on the last hyphen yields the chain `3`.
    #[test]
    fn a_hyphenated_chain_reference_survives() {
        let cosmos = ChainId::parse("cosmos:cosmoshub-3").unwrap();
        let id = PaymentMethodId::new("ATOM", &cosmos);
        assert_eq!(id.chain_id(), Some(cosmos));
    }

    #[test]
    fn malformed_method_ids_are_rejected() {
        assert!(PaymentMethodId::parse("invalid").is_none());
        // The old format must not parse: it would yield a chain nothing can
        // resolve, and the migration converts these explicitly.
        assert!(PaymentMethodId::parse("ETH-1").is_none());
        assert!(PaymentMethodId::parse("ETH@abc").is_none());
        assert!(PaymentMethodId::parse("@eip155:1").is_none());
    }
}
