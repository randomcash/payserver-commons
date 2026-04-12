//! Network badge components.

use leptos::prelude::*;
use types::Network;

/// Network badge colors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NetworkColor {
    #[default]
    Gray,
    Blue,
    Purple,
    Orange,
    Green,
    Red,
    Yellow,
}

impl NetworkColor {
    fn class(&self) -> &'static str {
        match self {
            Self::Gray => "ps-badge-gray",
            Self::Blue => "ps-badge-blue",
            Self::Purple => "ps-badge-purple",
            Self::Orange => "ps-badge-orange",
            Self::Green => "ps-badge-green",
            Self::Red => "ps-badge-red",
            Self::Yellow => "ps-badge-yellow",
        }
    }
}

/// Get network color for a Network type.
///
/// Returns the brand color for mainnet networks.
/// Use `testnet=true` on `NetworkBadge` for testnets (they use a default gray color).
pub fn color_for_network(network: &Network) -> NetworkColor {
    match network {
        // Bitcoin family - orange (Bitcoin brand color)
        Network::BitcoinMainnet | Network::BitcoinLightning => NetworkColor::Orange,

        // Ethereum and L2s that use ETH - blue
        Network::Ethereum => NetworkColor::Blue,
        Network::Arbitrum => NetworkColor::Blue,
        Network::Base => NetworkColor::Blue,
        Network::Linea => NetworkColor::Blue,

        // Optimism - red (brand color)
        Network::Optimism => NetworkColor::Red,

        // Polygon - purple (brand color)
        Network::Polygon => NetworkColor::Purple,

        // zkSync - purple
        Network::ZkSync => NetworkColor::Purple,

        // Avalanche - red (brand color)
        Network::Avalanche => NetworkColor::Red,

        // BSC/BNB - yellow (brand color)
        Network::BinanceSmartChain => NetworkColor::Yellow,

        // Scroll - orange
        Network::Scroll => NetworkColor::Orange,
    }
}

/// Network badge component.
///
/// Displays a colored badge for a blockchain network.
/// Testnets always use a gray color regardless of the network.
#[component]
pub fn NetworkBadge(
    /// The network type.
    network: Network,
    /// Custom name override. If not provided, uses `network.display_name()`.
    #[prop(optional)]
    name: Option<String>,
    /// Explicit color override (ignored for testnets).
    #[prop(optional)]
    color: Option<NetworkColor>,
    /// Whether this is a testnet. Testnets use gray color.
    #[prop(default = false)]
    testnet: bool,
) -> impl IntoView {
    // Testnets always use gray, mainnets use network color or explicit override
    let badge_color = if testnet {
        NetworkColor::Gray
    } else {
        color.unwrap_or_else(|| color_for_network(&network))
    };

    let display_name = name.unwrap_or_else(|| network.display_name().to_string());

    view! {
        <span class=format!("ps-network-badge {}", badge_color.class())>
            <span class="ps-network-name">{display_name}</span>
            {testnet.then(|| view! {
                <span class="ps-network-testnet">"Testnet"</span>
            })}
        </span>
    }
}

/// Status badge for payment/invoice states.
#[component]
pub fn StatusBadge(status: String) -> impl IntoView {
    let status_class = match status.to_lowercase().as_str() {
        "pending" => "ps-status-pending",
        "processing" => "ps-status-processing",
        "paid" => "ps-status-paid",
        "expired" => "ps-status-expired",
        "cancelled" => "ps-status-cancelled",
        "late_paid" => "ps-status-late",
        _ => "ps-status-unknown",
    };

    view! {
        <span class=format!("ps-status-badge {}", status_class)>
            {status}
        </span>
    }
}

/// Badge styles CSS.
pub const BADGE_STYLES: &str = r#"
.ps-network-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--ps-spacing-xs);
    padding: var(--ps-spacing-xs) var(--ps-spacing-sm);
    font-size: var(--ps-font-sm);
    font-weight: 500;
    border-radius: var(--ps-radius-full);
}

.ps-network-name { color: inherit; }
.ps-network-testnet { font-size: 0.75em; opacity: 0.8; }

.ps-badge-gray { background-color: #e5e7eb; color: #374151; }
.ps-badge-blue { background-color: #dbeafe; color: #1e40af; }
.ps-badge-purple { background-color: #e9d5ff; color: #6b21a8; }
.ps-badge-orange { background-color: #fed7aa; color: #c2410c; }
.ps-badge-green { background-color: #bbf7d0; color: #166534; }
.ps-badge-red { background-color: #fecaca; color: #b91c1c; }
.ps-badge-yellow { background-color: #fef08a; color: #a16207; }

.ps-status-badge {
    display: inline-block;
    padding: var(--ps-spacing-xs) var(--ps-spacing-sm);
    font-size: var(--ps-font-sm);
    font-weight: 500;
    border-radius: var(--ps-radius-full);
    text-transform: capitalize;
}

.ps-status-pending { background-color: #fef3c7; color: #92400e; }
.ps-status-processing { background-color: #dbeafe; color: #1e40af; }
.ps-status-paid { background-color: #bbf7d0; color: #166534; }
.ps-status-expired { background-color: #e5e7eb; color: #374151; }
.ps-status-cancelled { background-color: #fecaca; color: #b91c1c; }
.ps-status-late { background-color: #fed7aa; color: #c2410c; }
.ps-status-unknown { background-color: #e5e7eb; color: #374151; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_color_class() {
        assert_eq!(NetworkColor::Gray.class(), "ps-badge-gray");
        assert_eq!(NetworkColor::Blue.class(), "ps-badge-blue");
        assert_eq!(NetworkColor::Purple.class(), "ps-badge-purple");
        assert_eq!(NetworkColor::Orange.class(), "ps-badge-orange");
        assert_eq!(NetworkColor::Green.class(), "ps-badge-green");
        assert_eq!(NetworkColor::Red.class(), "ps-badge-red");
        assert_eq!(NetworkColor::Yellow.class(), "ps-badge-yellow");
    }

    #[test]
    fn test_network_color_default() {
        assert_eq!(NetworkColor::default(), NetworkColor::Gray);
    }

    #[test]
    fn test_color_for_network_bitcoin() {
        assert_eq!(
            color_for_network(&Network::BitcoinMainnet),
            NetworkColor::Orange
        );
        assert_eq!(
            color_for_network(&Network::BitcoinLightning),
            NetworkColor::Orange
        );
    }

    #[test]
    fn test_color_for_network_ethereum_family() {
        // Ethereum and L2s using ETH should be blue
        assert_eq!(color_for_network(&Network::Ethereum), NetworkColor::Blue);
        assert_eq!(color_for_network(&Network::Arbitrum), NetworkColor::Blue);
        assert_eq!(color_for_network(&Network::Base), NetworkColor::Blue);
        assert_eq!(color_for_network(&Network::Linea), NetworkColor::Blue);
    }

    #[test]
    fn test_color_for_network_brand_colors() {
        // Networks with distinct brand colors
        assert_eq!(color_for_network(&Network::Optimism), NetworkColor::Red);
        assert_eq!(color_for_network(&Network::Polygon), NetworkColor::Purple);
        assert_eq!(color_for_network(&Network::ZkSync), NetworkColor::Purple);
        assert_eq!(color_for_network(&Network::Avalanche), NetworkColor::Red);
        assert_eq!(
            color_for_network(&Network::BinanceSmartChain),
            NetworkColor::Yellow
        );
        assert_eq!(color_for_network(&Network::Scroll), NetworkColor::Orange);
    }

    #[test]
    fn test_all_networks_have_colors() {
        // Ensure every Network variant returns a color (not panicking)
        let networks = [
            Network::BitcoinMainnet,
            Network::BitcoinLightning,
            Network::Ethereum,
            Network::Polygon,
            Network::Arbitrum,
            Network::Optimism,
            Network::Base,
            Network::Avalanche,
            Network::BinanceSmartChain,
            Network::ZkSync,
            Network::Linea,
            Network::Scroll,
        ];

        for network in networks {
            let color = color_for_network(&network);
            // Should not be default gray for any mainnet
            assert_ne!(
                color,
                NetworkColor::Gray,
                "Network {:?} should have a brand color",
                network
            );
        }
    }
}
