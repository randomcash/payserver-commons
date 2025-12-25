//! Token types.

use serde::{Deserialize, Serialize};

use super::Network;

/// Token data for persistence.
///
/// This is a generic token structure that can represent tokens from any blockchain.
/// Network-specific validation and token types (like ERC20, ERC721, etc.) should
/// be handled in the respective payserver crates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenData {
    /// Database ID (None if not persisted yet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// Token type/standard as a string (e.g., "erc20", "erc721", "brc20", "runes").
    /// Each payserver defines its own valid token types.
    pub token_type: String,

    /// Contract/token address or identifier.
    /// Format depends on the network (e.g., "0x..." for EVM, inscription ID for ordinals).
    pub address: String,

    /// Network this token is on.
    pub network: Network,

    /// Whether this token is enabled for payments.
    #[serde(default = "default_token_enabled")]
    pub enabled: bool,

    /// Token name (e.g., "Tether USD", "Bored Ape Yacht Club").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Token symbol (e.g., "USDT", "BAYC").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Number of decimals for fungible tokens.
    /// None for non-fungible tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,

    /// Specific token ID within a collection/contract.
    /// Used for NFTs or multi-token standards.
    /// Note: Token IDs can be very large, stored as string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
}

fn default_token_enabled() -> bool {
    true
}

impl TokenData {
    /// Create a new token with the basic required fields.
    pub fn new(
        token_type: impl Into<String>,
        address: impl Into<String>,
        network: Network,
    ) -> Self {
        Self {
            id: None,
            token_type: token_type.into(),
            address: address.into(),
            network,
            enabled: true,
            name: None,
            symbol: None,
            decimals: None,
            token_id: None,
        }
    }

    /// Set the token name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the token symbol.
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    /// Set the number of decimals.
    pub fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = Some(decimals);
        self
    }

    /// Set the token ID (for NFTs or multi-token standards).
    pub fn with_token_id(mut self, token_id: impl Into<String>) -> Self {
        self.token_id = Some(token_id.into());
        self
    }

    /// Set whether this token is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
