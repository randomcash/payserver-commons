//! Network badge components.

use leptos::prelude::*;
use types::ChainId;

/// Network badge colors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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

/// Pick a badge colour for a chain.
///
/// Brand colours for the chains we know, and a deterministic fallback for the
/// ones we do not. The fallback is the point: this used to `match` on a closed
/// enum, so a chain that was not a variant could not be rendered at all — and
/// adding a variant meant editing this crate and cutting a release. A chain
/// added by a plugin gets a stable colour here with no configuration and no
/// change to this file.
///
/// Keyed on the CAIP-2 identifier, so `eip155:10` is Optimism red on any server
/// that speaks to Optimism, without anyone agreeing on a display name first.
pub fn color_for_chain(chain_id: &ChainId) -> NetworkColor {
    match chain_id.as_str() {
        // Ethereum and the L2s that settle in ETH.
        "eip155:1" | "eip155:42161" | "eip155:8453" | "eip155:59144" => NetworkColor::Blue,
        // Optimism, Avalanche.
        "eip155:10" | "eip155:43114" => NetworkColor::Red,
        // Polygon, zkSync.
        "eip155:137" | "eip155:324" => NetworkColor::Purple,
        // BNB Chain.
        "eip155:56" => NetworkColor::Yellow,
        // Scroll, and Bitcoin's orange.
        "eip155:534352" => NetworkColor::Orange,
        "eip155:250" => NetworkColor::Blue,
        "eip155:100" => NetworkColor::Green,
        _ if chain_id.namespace() == "bip122" => NetworkColor::Orange,
        _ if chain_id.namespace() == "monero" => NetworkColor::Orange,
        _ if chain_id.namespace() == "solana" => NetworkColor::Purple,
        _ if chain_id.namespace() == "tron" => NetworkColor::Red,
        other => fallback_color(other),
    }
}

/// A stable colour for a chain nobody has named.
///
/// Deliberately not `Gray`: gray means "testnet" in this component, and an
/// unknown mainnet is not a testnet. Any colour is better than the wrong
/// meaning, and the same chain must get the same colour on every render, so
/// this hashes rather than counts.
fn fallback_color(identifier: &str) -> NetworkColor {
    const PALETTE: [NetworkColor; 6] = [
        NetworkColor::Blue,
        NetworkColor::Purple,
        NetworkColor::Orange,
        NetworkColor::Green,
        NetworkColor::Red,
        NetworkColor::Yellow,
    ];
    // FNV-1a. Not for security - just a stable spread that does not pull in a
    // dependency and does not vary between processes the way `DefaultHasher`
    // is permitted to.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in identifier.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    PALETTE[(hash % PALETTE.len() as u64) as usize]
}

/// Network badge component.
///
/// Displays a coloured badge for a chain. Testnets always use gray.
///
/// The display name is a parameter rather than something derived from the
/// chain: CAIP-2 references are mostly opaque genesis hashes, so no function
/// can turn `monero:418015bb9ae982a1975da7d79277c270` into "Monero". That
/// mapping is data (`chain_configs`), which is also what lets a plugin name its
/// own chain.
#[component]
pub fn NetworkBadge(
    /// The chain, as a CAIP-2 identifier.
    chain_id: ChainId,
    /// Human-readable name. Falls back to the raw identifier, which is ugly but
    /// honest — better than inventing a name for a chain we do not know.
    #[prop(optional)]
    name: Option<String>,
    /// Explicit colour override (ignored for testnets).
    #[prop(optional)]
    color: Option<NetworkColor>,
    /// Whether this is a testnet. Testnets use gray.
    #[prop(default = false)]
    testnet: bool,
) -> impl IntoView {
    let badge_color = if testnet {
        NetworkColor::Gray
    } else {
        color.unwrap_or_else(|| color_for_chain(&chain_id))
    };

    let display_name = name.unwrap_or_else(|| chain_id.to_string());

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
    fn brand_colors_are_keyed_on_the_caip2_identifier() {
        assert_eq!(color_for_chain(&ChainId::evm(1)), NetworkColor::Blue);
        assert_eq!(color_for_chain(&ChainId::evm(42161)), NetworkColor::Blue);
        assert_eq!(color_for_chain(&ChainId::evm(10)), NetworkColor::Red);
        assert_eq!(color_for_chain(&ChainId::evm(137)), NetworkColor::Purple);
        assert_eq!(color_for_chain(&ChainId::evm(56)), NetworkColor::Yellow);
        assert_eq!(color_for_chain(&ChainId::evm(100)), NetworkColor::Green);
    }

    #[test]
    fn non_evm_families_get_their_own_colors() {
        let btc = ChainId::parse("bip122:000000000019d6689c085ae165831e93").unwrap();
        assert_eq!(color_for_chain(&btc), NetworkColor::Orange);

        let xmr = ChainId::parse("monero:418015bb9ae982a1975da7d79277c270").unwrap();
        assert_eq!(color_for_chain(&xmr), NetworkColor::Orange);

        let sol = ChainId::parse("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp").unwrap();
        assert_eq!(color_for_chain(&sol), NetworkColor::Purple);

        let trx = ChainId::parse("tron:728126428").unwrap();
        assert_eq!(color_for_chain(&trx), NetworkColor::Red);
    }

    /// The property the closed enum could not have: a chain nobody hardcoded
    /// still renders, with the same colour every time.
    #[test]
    fn an_unknown_chain_still_gets_a_stable_color() {
        let unknown = ChainId::parse("cosmos:cosmoshub-3").unwrap();
        let first = color_for_chain(&unknown);
        assert_eq!(first, color_for_chain(&unknown), "colour must be stable");

        // And it must not be gray, which this component uses to mean "testnet".
        assert_ne!(
            first,
            NetworkColor::Gray,
            "an unknown mainnet must not be indistinguishable from a testnet"
        );
    }

    #[test]
    fn unknown_chains_do_not_all_collapse_to_one_color() {
        let ids = [
            "cosmos:cosmoshub-3",
            "starknet:SN_GOERLI",
            "lip9:9ee11e9df416b18b",
            "eip155:999999",
            "eip155:888888",
            "near:mainnet",
        ];
        let colors: std::collections::HashSet<_> = ids
            .iter()
            .map(|id| color_for_chain(&ChainId::parse(*id).unwrap()))
            .collect();
        assert!(
            colors.len() > 1,
            "the fallback should spread across the palette, not pick one colour"
        );
    }

    #[test]
    fn every_chain_we_support_gets_a_non_gray_color() {
        let chains = [
            "eip155:1",
            "eip155:10",
            "eip155:56",
            "eip155:100",
            "eip155:137",
            "eip155:250",
            "eip155:324",
            "eip155:8453",
            "eip155:42161",
            "eip155:43114",
            "eip155:59144",
            "eip155:534352",
            "tron:728126428",
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            "monero:418015bb9ae982a1975da7d79277c270",
            "bip122:000000000019d6689c085ae165831e93",
        ];

        for id in chains {
            let chain_id = ChainId::parse(id).unwrap();
            assert_ne!(
                color_for_chain(&chain_id),
                NetworkColor::Gray,
                "{id} rendered as gray, which this component uses to mean testnet"
            );
        }
    }
}
