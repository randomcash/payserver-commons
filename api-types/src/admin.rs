//! Server administration: users, roles and server-wide settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use types::{ChainId, StoreId};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// User info for admin views (excludes sensitive key material).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub primary_wallet_address: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub locked_until: Option<DateTime<Utc>>,
}

/// Paginated user list response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<AdminUserInfo>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Distinguishes a field that was absent from one explicitly set to `null`.
///
/// `#[serde(default)]` alone cannot: both arrive as `None`. This runs only
/// when the key is present, so absent stays `None` and an explicit null
/// becomes `Some(None)`.
fn present_option<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// Server settings response (returns defaults if no row).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettingsResponse {
    pub default_confirmations: i32,
    pub invoice_expiry_minutes: i32,
    pub rate_limit_rpm: i32,
    pub enabled_chain_ids: Vec<ChainId>,
    /// The operator's own store: where this instance issues and settles its
    /// own invoices, if it issues any to itself at all.
    ///
    /// Read once at boot, so a change here does not take effect until the
    /// server restarts. That is the behaviour and not a limitation waiting
    /// to be fixed: this id decides both where those invoices are issued and
    /// which store's settled payments a plugin watching it is told about,
    /// and moving it while invoices are outstanding would leave those
    /// invoices settling on a store nothing is watching - the merchant pays
    /// and is never credited. A client showing this must say so.
    #[serde(default)]
    pub operator_store_id: Option<StoreId>,
    /// Whether the value above is the one this process is actually running
    /// with.
    ///
    /// False after a change that has not been restarted into. Without this a
    /// client cannot tell "set and live" from "set and pending", and would
    /// show an operator a configuration the server is not using.
    #[serde(default)]
    pub operator_store_id_active: bool,
}

/// Request body for updating server settings.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerSettingsRequest {
    pub default_confirmations: i32,
    pub invoice_expiry_minutes: i32,
    pub rate_limit_rpm: i32,
    /// Absent leaves the stored list alone; a value replaces it.
    ///
    /// Optional for the same reason `operator_store_id` is, and with a
    /// sharper consequence. A stored `enabled_chain_ids` is authoritative:
    /// once one exists, every chain not in it is refused. And `GET` answers
    /// with compiled-in *mainnet* defaults when no row exists, so a client
    /// that round-trips whatever it was given writes a list nobody chose -
    /// on a testnet deployment that list has no Sepolia in it, and the
    /// instance stops accepting the only chain it watches.
    ///
    /// So a client that is not changing chains must not mention them. One
    /// that is sends the whole list, which is still a replace.
    #[serde(default, deserialize_with = "present_option")]
    pub enabled_chain_ids: Option<Vec<ChainId>>,
    /// Absent leaves it alone; `null` clears it; a value sets it.
    ///
    /// A double option, and it earns its awkwardness. This endpoint replaces
    /// the whole settings object, so a plain `Option` would make an omitted
    /// field indistinguishable from an explicit null - and every older client
    /// that PUTs the other four fields would silently clear the operator's
    /// own store on the next settings save. Absent has to mean "I am not
    /// talking about this", which only a nested option can express.
    #[serde(default, deserialize_with = "present_option")]
    pub operator_store_id: Option<Option<StoreId>>,
}

/// Request body for role update.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn base() -> String {
        r#""default_confirmations":3,"invoice_expiry_minutes":60,"rate_limit_rpm":100,"enabled_chain_ids":[]"#
            .to_string()
    }

    /// The three states have to stay distinguishable. Collapse absent into
    /// null and every client that saves settings without knowing about this
    /// field clears the operator's own store.
    #[test]
    fn an_absent_operator_store_is_not_an_explicit_null() {
        let absent: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{}}}", base())).unwrap();
        assert_eq!(
            absent.operator_store_id, None,
            "absent must mean 'not talking about this field'"
        );

        let cleared: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{},\"operator_store_id\":null}}", base())).unwrap();
        assert_eq!(
            cleared.operator_store_id,
            Some(None),
            "an explicit null must mean 'clear it', which absent does not"
        );

        let id = uuid::Uuid::from_u128(7);
        let set: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{},\"operator_store_id\":\"{id}\"}}", base()))
                .unwrap();
        assert_eq!(set.operator_store_id, Some(Some(StoreId(id))));

        assert_ne!(
            absent.operator_store_id, cleared.operator_store_id,
            "if these two ever compare equal, an older client clears the operator's own store"
        );
    }

    /// The case this exists to prevent: a client saving some *other* setting
    /// must not silently rewrite the chain list it was handed by `GET`.
    ///
    /// On a testnet deployment that list is the compiled-in mainnet defaults,
    /// which contain no Sepolia - so the round trip would leave the instance
    /// refusing the only chain it watches, from a save nobody thought was
    /// about chains.
    #[test]
    fn a_save_that_does_not_mention_chains_leaves_them_alone() {
        let without: UpdateServerSettingsRequest = serde_json::from_str(
            r#"{"default_confirmations":3,"invoice_expiry_minutes":60,"rate_limit_rpm":100}"#,
        )
        .unwrap();
        assert_eq!(
            without.enabled_chain_ids, None,
            "absent must mean 'not changing this', not 'set it to nothing'"
        );

        let cleared: UpdateServerSettingsRequest = serde_json::from_str(
            r#"{"default_confirmations":3,"invoice_expiry_minutes":60,"rate_limit_rpm":100,"enabled_chain_ids":[]}"#,
        )
        .unwrap();
        assert_eq!(
            cleared.enabled_chain_ids,
            Some(Vec::new()),
            "an explicit empty list is a deliberate choice and must survive as one"
        );
        assert_ne!(
            without.enabled_chain_ids, cleared.enabled_chain_ids,
            "if these compare equal, saving any setting rewrites the chain list"
        );

        let set: UpdateServerSettingsRequest = serde_json::from_str(
            r#"{"default_confirmations":3,"invoice_expiry_minutes":60,"rate_limit_rpm":100,"enabled_chain_ids":["eip155:11155111"]}"#,
        )
        .unwrap();
        assert_eq!(set.enabled_chain_ids, Some(vec![ChainId::evm(11155111)]));
    }
}
