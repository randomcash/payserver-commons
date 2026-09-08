//! Server administration: users, roles and server-wide settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use types::ChainId;

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

/// Server settings response (returns defaults if no row).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettingsResponse {
    pub default_confirmations: i32,
    pub invoice_expiry_minutes: i32,
    pub rate_limit_rpm: i32,
    pub enabled_chain_ids: Vec<ChainId>,
}

/// Request body for updating server settings.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerSettingsRequest {
    pub default_confirmations: i32,
    pub invoice_expiry_minutes: i32,
    pub rate_limit_rpm: i32,
    pub enabled_chain_ids: Vec<ChainId>,
}

/// Request body for role update.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}
