//! Network types.

use serde::{Deserialize, Serialize};

/// Supported blockchain networks across the PayServer ecosystem.
///
/// Each PayServer implementation supports a subset of these networks.
/// For example, `ethpayserver` handles all EVM networks, while `bitcoinpayserver`
/// handles Bitcoin mainnet and Lightning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    // =========================================================================
    // Bitcoin family
    // =========================================================================
    /// Bitcoin mainnet (on-chain)
    BitcoinMainnet,
    /// Bitcoin Lightning Network
    BitcoinLightning,

    // =========================================================================
    // EVM-compatible networks
    // =========================================================================
    /// Ethereum mainnet
    Ethereum,
    /// Polygon (formerly Matic)
    Polygon,
    /// Arbitrum One
    Arbitrum,
    /// Optimism
    Optimism,
    /// Base (Coinbase L2)
    Base,
    /// Avalanche C-Chain
    Avalanche,
    /// BNB Smart Chain (formerly BSC)
    BinanceSmartChain,
    /// zkSync Era
    ZkSync,
    /// Linea
    Linea,
    /// Scroll
    Scroll,
    /// Fantom Opera
    Fantom,
    /// Gnosis (formerly xDai)
    Gnosis,
}

impl Network {
    /// Returns true if this is an EVM-compatible network.
    pub fn is_evm(&self) -> bool {
        matches!(
            self,
            Network::Ethereum
                | Network::Polygon
                | Network::Arbitrum
                | Network::Optimism
                | Network::Base
                | Network::Avalanche
                | Network::BinanceSmartChain
                | Network::ZkSync
                | Network::Linea
                | Network::Scroll
                | Network::Fantom
                | Network::Gnosis
        )
    }

    /// Returns true if this is a Bitcoin-family network.
    pub fn is_bitcoin(&self) -> bool {
        matches!(self, Network::BitcoinMainnet | Network::BitcoinLightning)
    }

    /// Returns the display name for this network.
    pub fn display_name(&self) -> &'static str {
        match self {
            Network::BitcoinMainnet => "Bitcoin",
            Network::BitcoinLightning => "Lightning Network",
            Network::Ethereum => "Ethereum",
            Network::Polygon => "Polygon",
            Network::Arbitrum => "Arbitrum",
            Network::Optimism => "Optimism",
            Network::Base => "Base",
            Network::Avalanche => "Avalanche",
            Network::BinanceSmartChain => "BNB Chain",
            Network::ZkSync => "zkSync",
            Network::Linea => "Linea",
            Network::Scroll => "Scroll",
            Network::Fantom => "Fantom",
            Network::Gnosis => "Gnosis",
        }
    }

    /// Returns the native currency symbol for this network.
    pub fn native_symbol(&self) -> &'static str {
        match self {
            Network::BitcoinMainnet | Network::BitcoinLightning => "BTC",
            Network::Ethereum => "ETH",
            Network::Polygon => "POL",
            Network::Arbitrum => "ETH",
            Network::Optimism => "ETH",
            Network::Base => "ETH",
            Network::Avalanche => "AVAX",
            Network::BinanceSmartChain => "BNB",
            Network::ZkSync => "ETH",
            Network::Linea => "ETH",
            Network::Scroll => "ETH",
            Network::Fantom => "FTM",
            Network::Gnosis => "xDAI",
        }
    }

    /// Returns the network identifier as a string (for database storage).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BitcoinMainnet => "bitcoin_mainnet",
            Self::BitcoinLightning => "bitcoin_lightning",
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::Avalanche => "avalanche",
            Self::BinanceSmartChain => "binance_smart_chain",
            Self::ZkSync => "zksync",
            Self::Linea => "linea",
            Self::Scroll => "scroll",
            Self::Fantom => "fantom",
            Self::Gnosis => "gnosis",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bitcoin_mainnet" | "bitcoin" | "btc" => Ok(Self::BitcoinMainnet),
            "bitcoin_lightning" | "lightning" | "ln" => Ok(Self::BitcoinLightning),
            "ethereum" | "eth" => Ok(Self::Ethereum),
            "polygon" | "matic" => Ok(Self::Polygon),
            "arbitrum" | "arb" => Ok(Self::Arbitrum),
            "optimism" | "op" => Ok(Self::Optimism),
            "base" => Ok(Self::Base),
            "avalanche" | "avax" => Ok(Self::Avalanche),
            "binance_smart_chain" | "bsc" | "bnb" => Ok(Self::BinanceSmartChain),
            "zksync" | "zk_sync" => Ok(Self::ZkSync),
            "linea" => Ok(Self::Linea),
            "scroll" => Ok(Self::Scroll),
            "fantom" | "ftm" => Ok(Self::Fantom),
            "gnosis" | "xdai" => Ok(Self::Gnosis),
            _ => Err(format!("unknown network: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_is_evm() {
        assert!(Network::Ethereum.is_evm());
        assert!(Network::Polygon.is_evm());
        assert!(Network::Arbitrum.is_evm());
        assert!(!Network::BitcoinMainnet.is_evm());
        assert!(!Network::BitcoinLightning.is_evm());
    }

    #[test]
    fn test_network_is_bitcoin() {
        assert!(Network::BitcoinMainnet.is_bitcoin());
        assert!(Network::BitcoinLightning.is_bitcoin());
        assert!(!Network::Ethereum.is_bitcoin());
    }

    #[test]
    fn test_network_native_symbol() {
        assert_eq!(Network::BitcoinMainnet.native_symbol(), "BTC");
        assert_eq!(Network::Ethereum.native_symbol(), "ETH");
        assert_eq!(Network::Polygon.native_symbol(), "POL");
        assert_eq!(Network::Avalanche.native_symbol(), "AVAX");
        assert_eq!(Network::BinanceSmartChain.native_symbol(), "BNB");
        assert_eq!(Network::Fantom.native_symbol(), "FTM");
        assert_eq!(Network::Gnosis.native_symbol(), "xDAI");
    }

    #[test]
    fn test_network_display() {
        assert_eq!(Network::Ethereum.to_string(), "ethereum");
        assert_eq!(Network::BitcoinLightning.to_string(), "bitcoin_lightning");
        assert_eq!(
            Network::BinanceSmartChain.to_string(),
            "binance_smart_chain"
        );
    }

    #[test]
    fn test_network_from_str() {
        assert_eq!("ethereum".parse::<Network>().unwrap(), Network::Ethereum);
        assert_eq!("eth".parse::<Network>().unwrap(), Network::Ethereum);
        assert_eq!("polygon".parse::<Network>().unwrap(), Network::Polygon);
        assert_eq!(
            "bsc".parse::<Network>().unwrap(),
            Network::BinanceSmartChain
        );
        assert_eq!("fantom".parse::<Network>().unwrap(), Network::Fantom);
        assert_eq!("ftm".parse::<Network>().unwrap(), Network::Fantom);
        assert_eq!("gnosis".parse::<Network>().unwrap(), Network::Gnosis);
        assert_eq!("xdai".parse::<Network>().unwrap(), Network::Gnosis);
        assert!("invalid".parse::<Network>().is_err());
    }
}
