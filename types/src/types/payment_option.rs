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

use super::InvoiceId;

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
/// Format: `{ASSET}-{CHAIN_ID}`
/// Examples:
/// - `ETH-1` (ETH on Ethereum mainnet)
/// - `ETH-11155111` (ETH on Sepolia testnet)
/// - `USDC-137` (USDC on Polygon)
/// - `USDT-42161` (USDT on Arbitrum)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentMethodId(pub String);

impl PaymentMethodId {
    /// Create a new payment method ID.
    pub fn new(asset_symbol: &str, chain_id: u64) -> Self {
        Self(format!("{}-{}", asset_symbol.to_uppercase(), chain_id))
    }

    /// Parse a payment method ID string.
    pub fn parse(s: &str) -> Option<(String, u64)> {
        let parts: Vec<&str> = s.rsplitn(2, '-').collect();
        if parts.len() != 2 {
            return None;
        }
        let chain_id: u64 = parts[0].parse().ok()?;
        let asset = parts[1].to_string();
        Some((asset, chain_id))
    }

    /// Get the asset symbol.
    pub fn asset_symbol(&self) -> Option<String> {
        Self::parse(&self.0).map(|(asset, _)| asset)
    }

    /// Get the chain ID.
    pub fn chain_id(&self) -> Option<u64> {
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
    pub chain_id: u64,
    /// Asset symbol (e.g., "ETH", "USDC").
    pub asset_symbol: String,
    /// Token contract address (None for native assets).
    pub token_address: Option<String>,
    /// Number of decimals for this asset.
    pub decimals: u8,
    /// Payment destination address.
    pub payment_address: String,
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
        chain_id: u64,
        asset_symbol: &str,
        payment_address: &str,
        amount: &str,
    ) -> Self {
        Self {
            id: PaymentOptionId::new(),
            invoice_id,
            payment_method_id: PaymentMethodId::new(asset_symbol, chain_id),
            chain_id,
            asset_symbol: asset_symbol.to_string(),
            token_address: None,
            decimals: 18,
            payment_address: payment_address.to_string(),
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
        chain_id: u64,
        asset_symbol: &str,
        token_address: &str,
        decimals: u8,
        payment_address: &str,
        amount: &str,
    ) -> Self {
        Self {
            id: PaymentOptionId::new(),
            invoice_id,
            payment_method_id: PaymentMethodId::new(asset_symbol, chain_id),
            chain_id,
            asset_symbol: asset_symbol.to_string(),
            token_address: Some(token_address.to_string()),
            decimals,
            payment_address: payment_address.to_string(),
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
        let id = PaymentMethodId::new("ETH", 1);
        assert_eq!(id.as_str(), "ETH-1");
        assert_eq!(id.asset_symbol(), Some("ETH".to_string()));
        assert_eq!(id.chain_id(), Some(1));

        let id = PaymentMethodId::new("USDC", 137);
        assert_eq!(id.as_str(), "USDC-137");

        let id = PaymentMethodId::new("ETH", 11155111);
        assert_eq!(id.as_str(), "ETH-11155111");
        assert_eq!(id.chain_id(), Some(11155111));
    }

    #[test]
    fn test_payment_method_id_parse() {
        let (asset, chain_id) = PaymentMethodId::parse("ETH-1").unwrap();
        assert_eq!(asset, "ETH");
        assert_eq!(chain_id, 1);

        let (asset, chain_id) = PaymentMethodId::parse("USDC-137").unwrap();
        assert_eq!(asset, "USDC");
        assert_eq!(chain_id, 137);

        // Edge case: asset with hyphen
        let (asset, chain_id) = PaymentMethodId::parse("WETH-USDC-1").unwrap();
        assert_eq!(asset, "WETH-USDC");
        assert_eq!(chain_id, 1);

        assert!(PaymentMethodId::parse("invalid").is_none());
        assert!(PaymentMethodId::parse("ETH-abc").is_none());
    }
}
