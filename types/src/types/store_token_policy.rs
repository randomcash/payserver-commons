//! Store token policy types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Policy mode controlling how entries are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenPolicyMode {
    /// Only listed entries are accepted.
    Allowlist,
    /// All enabled payment methods are accepted except listed entries.
    Blocklist,
}

impl TokenPolicyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allowlist => "allowlist",
            Self::Blocklist => "blocklist",
        }
    }
}

impl std::fmt::Display for TokenPolicyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TokenPolicyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allowlist" => Ok(Self::Allowlist),
            "blocklist" => Ok(Self::Blocklist),
            other => Err(format!("invalid token policy mode: {other}")),
        }
    }
}

/// Per-store token policy header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTokenPolicy {
    pub id: Uuid,
    pub store_id: Uuid,
    pub mode: TokenPolicyMode,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Single entry in a store token policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTokenPolicyEntry {
    pub id: Uuid,
    pub policy_id: Uuid,
    /// EIP-155 chain ID.
    pub chain_id: i64,
    /// ERC20 token contract address, None for native asset.
    pub token_address: Option<String>,
    /// Asset symbol for display (ETH, USDC, etc.)
    pub asset_symbol: String,
}

/// Full policy with entries, returned by reader methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTokenPolicyWithEntries {
    pub id: Uuid,
    pub store_id: Uuid,
    pub mode: TokenPolicyMode,
    pub entries: Vec<StoreTokenPolicyEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
