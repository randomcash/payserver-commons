//! API keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// API key info for list/get responses.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfoResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    /// Per-key rate limit in requests per minute. Null = server default.
    pub rate_limit_rpm: Option<i32>,
    /// Set when the key is deprecated via rotation. Key remains valid during
    /// the grace window; null means not deprecated.
    pub deprecated_at: Option<DateTime<Utc>>,
    /// When the grace window ends for a deprecated key (computed server-side
    /// from `deprecated_at` + grace seconds). Null for non-deprecated keys.
    /// Surfacing this lets the client render the exact expiry without
    /// hardcoding the grace duration.
    pub deprecation_expires_at: Option<DateTime<Utc>>,
}

/// Response for listing API keys.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListResponse {
    pub keys: Vec<ApiKeyInfoResponse>,
}

/// Request to create a new API key.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyPayload {
    /// Human-readable name for the key.
    pub name: String,
    /// Optional expiration time.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response after creating an API key (includes plaintext key).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponsePayload {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    /// The plaintext API key. Store this securely — it cannot be retrieved again.
    pub key: String,
}

/// Response after rotating an API key.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateApiKeyResponsePayload {
    /// The new API key's ID.
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    /// The new plaintext API key. Store this securely.
    pub key: String,
    /// When the old key was deprecated (grace window starts here).
    pub old_key_deprecated_at: DateTime<Utc>,
    /// When the old key's grace window ends and it stops authenticating.
    /// Clients should show this directly instead of hardcoding "48 hours".
    pub old_key_grace_expires_at: DateTime<Utc>,
}

/// Request to update an API key's rate limit.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyPayload {
    /// Per-key rate limit in requests per minute. Null = use server default.
    pub rate_limit_rpm: Option<i32>,
}
