//! The HTTP contract between a PayServer and its client.
//!
//! Every request and response shape the API speaks lives here, and both sides
//! depend on it. That is the whole point: before this crate the client kept its
//! own hand-written copy of each struct, the two were compiled separately, and
//! nothing checked that they agreed. When they drifted the workspace still
//! built, clippy still passed, every test still passed - and the page died on
//! contact with the API.
//!
//! It drifted three times in one day. `Wallet.store_id` against the server's
//! `user_id` (RCS-234). Eleven fields expecting a numeric `chain_id` after the
//! server moved to CAIP-2 strings (RCS-241). `ChainHealthInfo.chain_id` sending
//! `"1"` where `"eip155:1"` was expected. Two of the three were caught by a
//! human reading a diff, because CI structurally could not see them.
//!
//! # Why not in `types`
//!
//! `types` is the domain model - `PaymentData`, `InvoiceData`, the repository
//! traits. Those describe what the system *is*; these describe what it *says*.
//! They change for different reasons and have different audiences: a repository
//! trait is nobody's business outside a server, and a response shape is a
//! promise to every integrator. Keeping them apart is what stops "commons" from
//! meaning "everything".
//!
//! # What may be absent, and what may not
//!
//! Client and server deploy separately, so a response from an older server has
//! to keep rendering rather than failing the whole parse over one field. Where
//! a sensible default exists it is applied: `enabled` and `archived`,
//! `payment_options`, `decimals` (18), `reorged`, `amount_received` ("0"), and
//! the health fields.
//!
//! Three are deliberately **required**, because defaulting them would be worse
//! than failing:
//!
//! * `InvoiceResponse::status` - defaulting to `Pending` would tell a merchant
//!   an invoice is unpaid when the field simply did not arrive.
//! * `InvoiceResponse::store_id` and `StoreResponse::owner_id` - a nil UUID is
//!   a valid-looking value pointing at nothing, and both are used to scope
//!   what a caller is allowed to see.
//!
//! # Constraints
//!
//! Compiled into the browser bundle, so: no sqlx, no tokio, nothing that will
//! not build for `wasm32-unknown-unknown`. The `openapi` feature adds
//! `utoipa::ToSchema` for the server's generated spec and is off by default so
//! the client does not carry it.

pub mod admin;
pub mod api_key;
pub mod checkout;
pub mod common;
pub mod dashboard;
pub mod health;
pub mod invoice;
pub mod payment;
pub mod store;
pub mod wallet;

pub use admin::*;
pub use api_key::*;
pub use checkout::*;
pub use common::*;
pub use dashboard::*;
pub use health::*;
pub use invoice::*;
pub use payment::*;
pub use store::*;
pub use wallet::*;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use types::ChainId;

    /// The wire form of a chain is the CAIP-2 string, on every shape that
    /// carries one.
    ///
    /// This is the assertion that was missing when the client and the server
    /// each kept their own copy: the server changed `chain_id` from a number to
    /// an identifier and the client went on expecting a number, and nothing in
    /// either build noticed.
    #[test]
    fn chain_ids_travel_as_caip2_strings() {
        let payment = PaymentResponse {
            id: "p1".into(),
            invoice_id: "i1".into(),
            chain_id: ChainId::evm(11_155_111),
            asset_symbol: "ETH".into(),
            token_address: None,
            decimals: 18,
            amount: "1".into(),
            tx_hash: "0xabc".into(),
            from_address: None,
            block_number: None,
            detected_at: chrono::Utc::now(),
            confirmed_at: None,
            reorged: false,
            store_id: None,
            store_name: None,
        };

        let json = serde_json::to_value(&payment).unwrap();
        assert_eq!(json["chain_id"], "eip155:11155111");

        let back: PaymentResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back.chain_id, payment.chain_id);
    }

    /// A malformed identifier is refused at the boundary rather than reaching a
    /// page as a value nothing can resolve.
    #[test]
    fn a_bare_number_no_longer_deserialises_as_a_chain() {
        let json = serde_json::json!({
            "id": "p1", "invoice_id": "i1", "chain_id": 1, "asset_symbol": "ETH",
            "token_address": null, "decimals": 18, "amount": "1", "tx_hash": "0xabc",
            "from_address": null, "block_number": null,
            "detected_at": "2026-01-01T00:00:00Z", "confirmed_at": null,
            "reorged": false, "store_id": null, "store_name": null
        });
        assert!(serde_json::from_value::<PaymentResponse>(json).is_err());
    }

    /// A create-invoice request omits every absent optional field, not some of
    /// them.
    ///
    /// Three of the five carried `skip_serializing_if` and two did not, so the
    /// browser sent `"customer_email": null, "redirect_url": null` while
    /// omitting the rest. Harmless against this server, but a silent
    /// request-shape change - and the guard test only checked two of the five,
    /// which is why nobody noticed.
    #[test]
    fn a_create_invoice_request_omits_all_absent_optionals() {
        let req = CreateInvoiceRequest {
            store_id: uuid::Uuid::nil(),
            amount: "10".into(),
            currency: "USD".into(),
            expiration_seconds: None,
            metadata: None,
            customer_email: None,
            webhook_url: None,
            redirect_url: None,
        };

        let json = serde_json::to_value(&req).unwrap();
        for absent in [
            "expiration_seconds",
            "metadata",
            "customer_email",
            "webhook_url",
            "redirect_url",
        ] {
            assert!(
                json.get(absent).is_none(),
                "`{absent}` was serialised as null instead of being omitted"
            );
        }
    }

    /// A rotation entry represents both of its lookup-derived labels the same
    /// way.
    #[test]
    fn rotation_entry_labels_are_both_optional() {
        let entry = RotationEntry {
            id: uuid::Uuid::nil(),
            payment_method_id: uuid::Uuid::nil(),
            chain_id: None,
            asset_symbol: None,
            previous_xpub_masked: "xpub…".into(),
            previous_derivation_index: 0,
            rotated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert!(json["chain_id"].is_null());
        assert!(
            json["asset_symbol"].is_null(),
            "one label as null and the other as \"\" is two answers to one question"
        );
    }

    /// Requests deserialise as well as serialise, and responses both ways.
    ///
    /// The server only ever needed one direction per shape and the client the
    /// other, which is how two half-contracts came to exist. Both directions
    /// are derived here so neither side can quietly stop honouring one.
    #[test]
    fn every_shape_round_trips_in_both_directions() {
        let req = CreatePaymentMethodRequest {
            chain_id: ChainId::evm(1),
            token_address: None,
            asset_symbol: "ETH".into(),
            decimals: 18,
            xpub: "xpub123".into(),
        };
        let back: CreatePaymentMethodRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(back.chain_id, req.chain_id);

        let wallet = WalletResponse {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            xpub_masked: "xpub…".into(),
            derivation_index: 3,
            name: None,
            is_primary: true,
            created_at: chrono::Utc::now(),
        };
        let back: WalletResponse =
            serde_json::from_value(serde_json::to_value(&wallet).unwrap()).unwrap();
        assert_eq!(back.derivation_index, 3);
        assert!(back.is_primary);
    }
}
