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
    /// The store this instance bills its own subscriptions through, if it
    /// sells anything to itself.
    ///
    /// Read once at boot, so a change here does not take effect until the
    /// server restarts. That is the behaviour and not a limitation waiting
    /// to be fixed: this id decides both where subscription invoices are
    /// issued and which store's settled payments a billing plugin is told
    /// about, and moving it while invoices are outstanding would leave those
    /// invoices settling on a store nothing is watching - the merchant pays
    /// and is never credited. A client showing this must say so.
    #[serde(default)]
    pub billing_store_id: Option<StoreId>,
    /// Whether the value above is the one this process is actually running
    /// with.
    ///
    /// False after a change that has not been restarted into. Without this a
    /// client cannot tell "set and live" from "set and pending", and would
    /// show an operator a configuration the server is not using.
    #[serde(default)]
    pub billing_store_id_active: bool,
}

/// Request body for updating server settings.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerSettingsRequest {
    pub default_confirmations: i32,
    pub invoice_expiry_minutes: i32,
    pub rate_limit_rpm: i32,
    pub enabled_chain_ids: Vec<ChainId>,
    /// Absent leaves it alone; `null` clears it; a value sets it.
    ///
    /// A double option, and it earns its awkwardness. This endpoint replaces
    /// the whole settings object, so a plain `Option` would make an omitted
    /// field indistinguishable from an explicit null - and every older client
    /// that PUTs the other four fields would silently switch billing off on
    /// the next settings save. Absent has to mean "I am not talking about
    /// this", which only a nested option can express.
    #[serde(default, deserialize_with = "present_option")]
    pub billing_store_id: Option<Option<StoreId>>,
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
    /// field switches billing off.
    #[test]
    fn an_absent_billing_store_is_not_an_explicit_null() {
        let absent: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{}}}", base())).unwrap();
        assert_eq!(
            absent.billing_store_id, None,
            "absent must mean 'not talking about this field'"
        );

        let cleared: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{},\"billing_store_id\":null}}", base())).unwrap();
        assert_eq!(
            cleared.billing_store_id,
            Some(None),
            "an explicit null must mean 'clear it', which absent does not"
        );

        let id = uuid::Uuid::from_u128(7);
        let set: UpdateServerSettingsRequest =
            serde_json::from_str(&format!("{{{},\"billing_store_id\":\"{id}\"}}", base())).unwrap();
        assert_eq!(set.billing_store_id, Some(Some(StoreId(id))));

        assert_ne!(
            absent.billing_store_id, cleared.billing_store_id,
            "if these two ever compare equal, an older client wipes the billing store on save"
        );
    }
}
