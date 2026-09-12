//! Payment request URIs, so a scanned QR fills the wallet in.
//!
//! A QR holding a bare address makes the customer type the amount and pick the
//! network themselves, at the one moment in the flow where a mistake costs
//! money. Both values are already known exactly - encoding them removes the
//! typing.
//!
//! EVM chains use **EIP-681** (*URL Format for Transaction Requests*), the
//! payment specialisation of EIP-831's `ethereum:` scheme. It is what MetaMask,
//! Rainbow, Trust and Coinbase Wallet parse from a QR.
//!
//! ```text
//! native    ethereum:<recipient>@<chain>?value=<wei>
//! ERC-20    ethereum:<token>@<chain>/transfer?address=<recipient>&uint256=<units>
//! ```
//!
//! The ERC-20 shape is not a variation on the native one: the URI targets the
//! **token contract**, and the recipient becomes a parameter, because the wallet
//! has to build a `transfer()` call rather than send value. Emitting the native
//! form for a token would point a wallet at sending ETH to the recipient, and
//! the token would never move.
//!
//! Amounts are integers in base units - wei, or the token's own decimals. No
//! float ever touches this: `0.1 + 0.2` is not `0.3`, and the number here is
//! what a customer pays.

use crate::ChainId;

/// A payment request a wallet can act on, or nothing if the chain has no
/// standard we can express.
///
/// Returns `None` rather than guessing. A malformed URI is worse than a bare
/// address: the address at least lets a customer proceed by hand, where a URI a
/// wallet misparses can send the wrong amount to the wrong place.
#[must_use]
pub fn payment_request_uri(
    chain_id: &ChainId,
    recipient: &str,
    amount_base_units: &str,
    token_address: Option<&str>,
) -> Option<String> {
    // Only EIP-155 chains have an EIP-681 meaning. A `tron:` or `solana:`
    // reference needs its own scheme, and inventing one here would produce a
    // string no wallet understands.
    let eip155 = chain_id.evm_chain_id()?;

    if amount_base_units.is_empty() || !amount_base_units.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if !is_hex_address(recipient) {
        return None;
    }

    Some(match token_address {
        Some(token) if is_hex_address(token) => {
            format!(
                "ethereum:{token}@{eip155}/transfer?address={recipient}&uint256={amount_base_units}"
            )
        }
        // A token address we cannot use is not a native payment - saying so
        // would tell the customer to send ETH where a token was owed.
        Some(_) => return None,
        None => format!("ethereum:{recipient}@{eip155}?value={amount_base_units}"),
    })
}

/// A 0x-prefixed 20-byte hex address.
///
/// Checked because the value goes into a URI a wallet will act on, and because
/// a caller passing a masked or truncated address would otherwise produce a
/// plausible-looking string pointing somewhere else.
fn is_hex_address(s: &str) -> bool {
    s.len() == 42 && s.starts_with("0x") && s[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECIPIENT: &str = "0x66da354a361225a8C4FF5232f413A80af943C14c";
    const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    #[test]
    fn a_native_payment_carries_the_chain_and_the_amount_in_wei() {
        let uri = payment_request_uri(
            &ChainId::evm(11155111),
            RECIPIENT,
            "47351100518494550",
            None,
        )
        .expect("native uri");
        assert_eq!(
            uri,
            "ethereum:0x66da354a361225a8C4FF5232f413A80af943C14c@11155111?value=47351100518494550"
        );
    }

    #[test]
    fn an_erc20_payment_targets_the_token_and_moves_the_recipient_into_a_parameter() {
        // The shape that is easy to get wrong. Emitting the native form here
        // would have a wallet send ETH to the recipient and never move the
        // token.
        let uri = payment_request_uri(&ChainId::evm(1), RECIPIENT, "1000000", Some(USDC))
            .expect("erc20 uri");
        assert_eq!(
            uri,
            "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48@1/transfer\
             ?address=0x66da354a361225a8C4FF5232f413A80af943C14c&uint256=1000000"
        );
        assert!(
            !uri.contains("value="),
            "a token transfer must not carry a native value: {uri}"
        );
    }

    #[test]
    fn a_non_evm_chain_gets_no_uri_rather_than_a_guess() {
        let tron = ChainId::parse("tron:728126428").expect("caip2");
        assert_eq!(payment_request_uri(&tron, RECIPIENT, "1000000", None), None);
    }

    #[test]
    fn a_decimal_amount_is_refused() {
        // Base units only. A wallet reading "0.047" as wei would send 0.047 wei
        // - or reject it - and either way the customer does not pay.
        assert_eq!(
            payment_request_uri(&ChainId::evm(1), RECIPIENT, "0.047", None),
            None
        );
        assert_eq!(
            payment_request_uri(&ChainId::evm(1), RECIPIENT, "", None),
            None
        );
        assert_eq!(
            payment_request_uri(&ChainId::evm(1), RECIPIENT, "4.7e16", None),
            None
        );
    }

    #[test]
    fn an_address_that_is_not_an_address_is_refused() {
        // A masked address is the realistic mistake: it looks right and points
        // nowhere.
        for bad in [
            "0x66da354a...943C14c",
            "66da354a361225a8C4FF5232f413A80af943C14c",
            "0x",
            "",
        ] {
            assert_eq!(
                payment_request_uri(&ChainId::evm(1), bad, "1", None),
                None,
                "should refuse {bad:?}"
            );
        }
    }

    #[test]
    fn an_unusable_token_address_is_not_downgraded_to_a_native_payment() {
        // The dangerous fallback. Owed USDC, told to send ETH.
        assert_eq!(
            payment_request_uri(
                &ChainId::evm(1),
                RECIPIENT,
                "1000000",
                Some("not-an-address")
            ),
            None
        );
    }
}

#[cfg(test)]
mod reachable {
    //! The function is only useful to a consumer, so pin that a consumer can
    //! reach it.
    //!
    //! It was added to `types::types`'s re-export list but not the crate root's,
    //! so `types::payment_request_uri` did not resolve outside this crate. Every
    //! test above passed, because they call it by its in-crate path - a `pub`
    //! item can be untouchable from outside and the tests will not say so.

    #[test]
    fn the_crate_root_exports_it() {
        // Names it through the crate root exactly as a consumer would.
        let f: fn(&crate::ChainId, &str, &str, Option<&str>) -> Option<String> =
            crate::payment_request_uri;
        assert!(
            f(
                &crate::ChainId::evm(1),
                "0x66da354a361225a8C4FF5232f413A80af943C14c",
                "1",
                None
            )
            .is_some()
        );
    }
}
