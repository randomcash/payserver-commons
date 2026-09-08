//! Chain identity, as CAIP-2.
//!
//! A payment server has to name the chain a payment arrived on, and that name
//! ends up in the database, in the API, and in front of a wallet. This is that
//! name.
//!
//! It used to be `chain_id: u64` — an EIP-155 number. That works for exactly one
//! family of chains. Tron, Solana, Monero and Bitcoin have no EIP-155 id, so
//! supporting them meant either inventing numbers for them or adding a second
//! identifier alongside the first. There was also a closed `Network` enum, which
//! had the worse problem: a new chain required editing commons and cutting a
//! release, so no plugin could ever add one.
//!
//! [CAIP-2] is the identifier the rest of the ecosystem already uses for this.
//! CASA — MetaMask, WalletConnect, Ledger among them — publishes it, and
//! WalletConnect negotiates sessions in terms of it. So the id stored here is
//! the id a wallet SDK expects, with nothing in between. It is also open by
//! construction: a namespace is a string, not a variant, so a chain can be added
//! without touching this crate.
//!
//! ```text
//! chain_id:  namespace + ":" + reference
//! namespace: [-a-z0-9]{3,8}
//! reference: [-_a-zA-Z0-9]{1,32}
//! ```
//!
//! | chain           | CAIP-2                                     |
//! |-----------------|--------------------------------------------|
//! | Ethereum        | `eip155:1`                                 |
//! | Sepolia         | `eip155:11155111`                          |
//! | Tron mainnet    | `tron:728126428`                           |
//! | Solana mainnet  | `solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`  |
//! | Monero mainnet  | `monero:418015bb9ae982a1975da7d79277c270`  |
//! | Bitcoin mainnet | `bip122:000000000019d6689c085ae165831e93`  |
//!
//! Note that most references are **opaque**. Only `eip155` and `tron` are
//! numbers; the rest are genesis hashes truncated to the 32 characters CAIP-2
//! allows. Nothing may infer a display name from an identifier — that mapping is
//! data (`chain_configs`), which is also what lets a plugin describe its own
//! chain.
//!
//! [CAIP-2]: https://standards.chainagnostic.org/CAIPs/caip-2

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// CAIP-2 namespace for EVM chains, keyed by EIP-155 chain id.
pub const NAMESPACE_EIP155: &str = "eip155";
/// CAIP-2 namespace for Tron, keyed by its decimal chain id.
pub const NAMESPACE_TRON: &str = "tron";
/// CAIP-2 namespace for Solana, keyed by a truncated genesis hash.
pub const NAMESPACE_SOLANA: &str = "solana";
/// CAIP-2 namespace for Monero, keyed by a truncated genesis hash.
pub const NAMESPACE_MONERO: &str = "monero";
/// CAIP-2 namespace for Bitcoin-family chains, keyed by a truncated genesis hash.
pub const NAMESPACE_BIP122: &str = "bip122";

/// Why a string is not a chain identifier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChainIdError {
    #[error("chain id is empty")]
    Empty,
    #[error("chain id `{0}` has no `:` separating namespace from reference")]
    MissingSeparator(String),
    #[error(
        "namespace `{0}` is not 3-8 characters of [-a-z0-9] (CAIP-2); \
         note it is lowercase only"
    )]
    InvalidNamespace(String),
    #[error("reference `{0}` is not 1-32 characters of [-_a-zA-Z0-9] (CAIP-2)")]
    InvalidReference(String),
}

/// A chain, identified the way the rest of the ecosystem identifies chains.
///
/// Holds the whole identifier as one validated string so that [`Self::as_str`]
/// is free — this is bound into SQL and rendered into JSON far more often than
/// it is taken apart.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(value_type = String, example = "eip155:1"))]
pub struct ChainId(String);

impl ChainId {
    /// Parse and validate a CAIP-2 identifier.
    pub fn parse(s: impl Into<String>) -> Result<Self, ChainIdError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ChainIdError::Empty);
        }

        // Split on the FIRST colon only. A second one lands in the reference and
        // fails its charset check there, which reports the actual offending text
        // rather than a generic "too many separators".
        let (namespace, reference) = s
            .split_once(':')
            .ok_or_else(|| ChainIdError::MissingSeparator(s.clone()))?;

        validate_namespace(namespace)?;
        validate_reference(reference)?;

        Ok(Self(s))
    }

    /// Build from parts, validating each.
    pub fn new(namespace: &str, reference: &str) -> Result<Self, ChainIdError> {
        validate_namespace(namespace)?;
        validate_reference(reference)?;
        Ok(Self(format!("{namespace}:{reference}")))
    }

    /// An EVM chain, from its EIP-155 id.
    ///
    /// Infallible: a `u64` in decimal is at most 20 digits, well inside the
    /// 32-character reference limit, and digits are always in the charset.
    pub fn evm(eip155_chain_id: u64) -> Self {
        Self(format!("{NAMESPACE_EIP155}:{eip155_chain_id}"))
    }

    /// The whole identifier. Free — this is the stored form.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The part before the colon: which family of chain this is.
    pub fn namespace(&self) -> &str {
        // Safe to unwrap: no `ChainId` exists without a validated separator.
        self.0.split_once(':').map(|(ns, _)| ns).unwrap_or(&self.0)
    }

    /// The part after the colon. Opaque for every namespace except `eip155`
    /// and `tron`.
    pub fn reference(&self) -> &str {
        self.0.split_once(':').map(|(_, r)| r).unwrap_or("")
    }

    /// Whether this is an EVM chain.
    pub fn is_evm(&self) -> bool {
        self.namespace() == NAMESPACE_EIP155
    }

    /// The EIP-155 chain id, for the EVM tooling that still needs a number.
    ///
    /// `None` for any other namespace — including `tron`, whose reference is
    /// also numeric but is emphatically not an EIP-155 id and must not be
    /// handed to an EVM RPC.
    pub fn evm_chain_id(&self) -> Option<u64> {
        if !self.is_evm() {
            return None;
        }
        self.reference().parse().ok()
    }
}

fn validate_namespace(namespace: &str) -> Result<(), ChainIdError> {
    let valid = (3..=8).contains(&namespace.len())
        && namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    valid
        .then_some(())
        .ok_or_else(|| ChainIdError::InvalidNamespace(namespace.to_string()))
}

fn validate_reference(reference: &str) -> Result<(), ChainIdError> {
    let valid = (1..=32).contains(&reference.len())
        && reference
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    valid
        .then_some(())
        .ok_or_else(|| ChainIdError::InvalidReference(reference.to_string()))
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ChainId {
    type Err = ChainIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl AsRef<str> for ChainId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ChainId> for String {
    fn from(chain: ChainId) -> Self {
        chain.0
    }
}

impl TryFrom<String> for ChainId {
    type Error = ChainIdError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Serialised as the plain identifier, never as a struct.
///
/// It travels through JSON APIs, a `TEXT` column and wallet SDKs, and all three
/// want `"eip155:1"`. A `{"namespace": ..., "reference": ...}` shape would have
/// to be unpicked at every one of those boundaries.
impl Serialize for ChainId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

/// Validating on the way in, so an invalid identifier cannot exist.
impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identifiers this system is actually expected to carry, taken from
    /// the CASA namespaces registry rather than invented. Every one of these
    /// must survive a round trip; a change that breaks one is a change that
    /// silently stops identifying a chain we support.
    const REAL_CHAINS: &[(&str, &str)] = &[
        ("eip155:1", "Ethereum mainnet"),
        ("eip155:137", "Polygon"),
        ("eip155:11155111", "Sepolia"),
        ("tron:728126428", "Tron mainnet"),
        ("tron:3448148188", "Tron Nile"),
        ("tron:2494104990", "Tron Shasta"),
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", "Solana mainnet"),
        ("solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1", "Solana devnet"),
        ("monero:418015bb9ae982a1975da7d79277c270", "Monero mainnet"),
        ("monero:76ee3cc98646292206cd3e86f74d88b4", "Monero stagenet"),
        ("bip122:000000000019d6689c085ae165831e93", "Bitcoin mainnet"),
        ("bip122:12a765e31ffd4059bada1e25190f6e98", "Litecoin"),
    ];

    #[test]
    fn every_chain_we_intend_to_support_round_trips() {
        for (id, label) in REAL_CHAINS {
            let chain = ChainId::parse(*id).unwrap_or_else(|e| panic!("{label} ({id}): {e}"));
            assert_eq!(chain.as_str(), *id, "{label} did not round trip");
            assert_eq!(chain.to_string(), *id);
            assert_eq!(id.parse::<ChainId>().unwrap(), chain);
        }
    }

    #[test]
    fn parts_are_recoverable() {
        let sepolia = ChainId::parse("eip155:11155111").unwrap();
        assert_eq!(sepolia.namespace(), "eip155");
        assert_eq!(sepolia.reference(), "11155111");

        let monero = ChainId::parse("monero:418015bb9ae982a1975da7d79277c270").unwrap();
        assert_eq!(monero.namespace(), "monero");
        assert_eq!(monero.reference(), "418015bb9ae982a1975da7d79277c270");
    }

    #[test]
    fn evm_constructor_matches_the_parsed_form() {
        assert_eq!(ChainId::evm(1), ChainId::parse("eip155:1").unwrap());
        assert_eq!(
            ChainId::evm(11_155_111),
            ChainId::parse("eip155:11155111").unwrap()
        );
        // The largest u64 is 20 digits — still inside the 32-char reference.
        assert!(ChainId::parse(ChainId::evm(u64::MAX).as_str()).is_ok());
    }

    /// Tron's reference is numeric too, and it is NOT an EIP-155 id. Handing
    /// `728126428` to an EVM RPC because it happened to parse as a number is
    /// the mistake this guards.
    #[test]
    fn only_evm_chains_yield_an_eip155_id() {
        assert_eq!(ChainId::evm(137).evm_chain_id(), Some(137));
        assert!(ChainId::evm(137).is_evm());

        let tron = ChainId::parse("tron:728126428").unwrap();
        assert!(!tron.is_evm());
        assert_eq!(
            tron.evm_chain_id(),
            None,
            "a numeric Tron reference must not be mistaken for an EIP-155 id"
        );

        let solana = ChainId::parse("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp").unwrap();
        assert_eq!(solana.evm_chain_id(), None);
    }

    #[test]
    fn namespace_bounds_are_enforced() {
        assert!(ChainId::parse("ab:1").is_err(), "2 chars is below minimum");
        assert!(ChainId::parse("abc:1").is_ok(), "3 chars is the minimum");
        assert!(
            ChainId::parse("abcdefgh:1").is_ok(),
            "8 chars is the maximum"
        );
        assert!(ChainId::parse("abcdefghi:1").is_err(), "9 chars is over");
    }

    /// CAIP-2 namespaces are lowercase. `EIP155:1` is not a valid identifier,
    /// and accepting it would mean two spellings of one chain in the database.
    #[test]
    fn namespaces_are_lowercase_only() {
        assert!(matches!(
            ChainId::parse("EIP155:1"),
            Err(ChainIdError::InvalidNamespace(_))
        ));
        assert!(matches!(
            ChainId::parse("Eip155:1"),
            Err(ChainIdError::InvalidNamespace(_))
        ));
    }

    /// References ARE case-sensitive — Solana's base58 genesis hash depends on
    /// it, so lowercasing a chain id would corrupt it.
    #[test]
    fn references_keep_their_case() {
        let solana = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";
        assert_eq!(ChainId::parse(solana).unwrap().as_str(), solana);
    }

    #[test]
    fn reference_bounds_are_enforced() {
        assert!(matches!(
            ChainId::parse("eip155:"),
            Err(ChainIdError::InvalidReference(_))
        ));
        let max = "a".repeat(32);
        assert!(ChainId::parse(format!("eip155:{max}")).is_ok());
        let over = "a".repeat(33);
        assert!(ChainId::parse(format!("eip155:{over}")).is_err());
    }

    #[test]
    fn malformed_input_is_rejected_with_a_reason() {
        assert!(matches!(ChainId::parse(""), Err(ChainIdError::Empty)));
        assert!(matches!(
            ChainId::parse("eip155"),
            Err(ChainIdError::MissingSeparator(_))
        ));
        // A second colon lands in the reference and fails its charset check.
        assert!(matches!(
            ChainId::parse("eip155:1:2"),
            Err(ChainIdError::InvalidReference(_))
        ));
        assert!(matches!(
            ChainId::parse("eip 155:1"),
            Err(ChainIdError::InvalidNamespace(_))
        ));
    }

    /// A bare EIP-155 number is what the old column held. It must NOT parse:
    /// the migration converts those explicitly, and silently accepting one here
    /// would let un-migrated data through as an unidentifiable chain.
    #[test]
    fn a_bare_eip155_number_is_not_a_chain_id() {
        assert!(ChainId::parse("1").is_err());
        assert!(ChainId::parse("11155111").is_err());
    }

    #[test]
    fn serialises_as_a_plain_string() {
        let chain = ChainId::evm(1);
        assert_eq!(serde_json::to_string(&chain).unwrap(), "\"eip155:1\"");
        assert_eq!(
            serde_json::from_str::<ChainId>("\"eip155:1\"").unwrap(),
            chain
        );
    }

    /// Deserialisation validates, so an invalid identifier cannot enter through
    /// an API payload or a stored blob.
    #[test]
    fn deserialising_rejects_an_invalid_identifier() {
        assert!(serde_json::from_str::<ChainId>("\"nope\"").is_err());
        assert!(serde_json::from_str::<ChainId>("\"EIP155:1\"").is_err());
        assert!(serde_json::from_str::<ChainId>("1").is_err());
    }

    /// Used as a map key and sorted in listings, so both must be stable.
    #[test]
    fn is_usable_as_a_key() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ChainId::evm(1), "mainnet");
        assert_eq!(
            map.get(&ChainId::parse("eip155:1").unwrap()),
            Some(&"mainnet")
        );

        let mut chains = [ChainId::evm(137), ChainId::evm(1)];
        chains.sort();
        assert_eq!(chains[0], ChainId::evm(1));
    }
}
