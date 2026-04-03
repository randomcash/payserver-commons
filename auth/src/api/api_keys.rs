//! API key management handlers.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    ApiKey, ApiKeyId, ApiKeyInfo, AuthenticationService, SessionId,
};

use super::AuthState;

/// Extract session ID from Authorization header (Bearer token).
fn extract_session_from_header(headers: &HeaderMap) -> Result<SessionId, (StatusCode, String)> {
    use axum::http::header;

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string()))?;

    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Bearer token format".to_string()))?;

    let session_uuid = Uuid::parse_str(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session ID format".to_string()))?;

    Ok(SessionId(session_uuid))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    /// Human-readable name for the API key (e.g., "Production Key", "Testing")
    pub name: String,

    /// Optional expiration date for the key
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResponse {
    /// The key ID
    pub id: ApiKeyId,

    /// Human-readable name
    pub name: String,

    /// Display prefix (e.g., "ak_live_****1234")
    pub key_prefix: String,

    /// The plaintext API key (shown only once at creation)
    /// Format: "ak_live_" or "ak_test_" followed by 32 random characters
    pub plaintext_key: String,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyListResponse {
    pub keys: Vec<ApiKeyInfo>,
}

/// Generate a new API key with prefix.
///
/// Format: "ak_live_" (or "ak_test_") + 32 random alphanumeric characters
/// Returns: (plaintext_key, key_prefix, key_hash)
fn generate_api_key() -> (String, String, String) {
    use rand::Rng;

    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    // Generate 32 random characters
    let random_part: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let plaintext_key = format!("ak_live_{}", random_part);

    // Generate display prefix (first 4 + last 4 chars)
    let display_suffix = random_part.chars().rev().take(4).collect::<String>();
    let prefix_start = random_part.chars().take(4).collect::<String>();
    let key_prefix = format!("ak_live_{}****{}", prefix_start, display_suffix);

    // Hash the plaintext key for storage
    let mut hasher = Sha256::new();
    hasher.update(plaintext_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    (plaintext_key, key_prefix, key_hash)
}

#[utoipa::path(
    get,
    path = "/auth/api-keys",
    tag = "api_keys",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of API keys", body = ApiKeyListResponse),
        (status = 401, description = "Invalid session"),
    )
)]
pub async fn list_api_keys<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    headers: HeaderMap,
) -> Result<Json<ApiKeyListResponse>, (StatusCode, String)> {
    let session_id = extract_session_from_header(&headers)?;

    let (user_info, _session) = state
        .service
        .validate_session(session_id)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let keys = state
        .service
        .list_api_keys(user_info.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiKeyListResponse { keys }))
}

#[utoipa::path(
    post,
    path = "/auth/api-keys",
    tag = "api_keys",
    security(("bearer_auth" = [])),
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = CreateApiKeyResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Invalid session"),
    )
)]
pub async fn create_api_key<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), (StatusCode, String)> {
    let session_id = extract_session_from_header(&headers)?;

    let (user_info, _session) = state
        .service
        .validate_session(session_id)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let (plaintext_key, key_prefix, key_hash) = generate_api_key();

    let api_key = ApiKey {
        id: ApiKeyId::new(),
        user_id: user_info.id,
        name: req.name.clone(),
        key_hash,
        key_prefix: key_prefix.clone(),
        is_active: true,
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: req.expires_at,
    };

    state
        .service
        .create_api_key(&api_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = CreateApiKeyResponse {
        id: api_key.id,
        name: req.name,
        key_prefix,
        plaintext_key,
        created_at: api_key.created_at,
        expires_at: req.expires_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/auth/api-keys/{id}",
    tag = "api_keys",
    security(("bearer_auth" = [])),
    params(("id" = ApiKeyId, Path)),
    responses(
        (status = 204, description = "API key deleted"),
        (status = 401, description = "Invalid session"),
        (status = 404, description = "API key not found"),
    )
)]
pub async fn delete_api_key<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    headers: HeaderMap,
    Path(key_id): Path<ApiKeyId>,
) -> Result<StatusCode, (StatusCode, String)> {
    let session_id = extract_session_from_header(&headers)?;

    let (user_info, _session) = state
        .service
        .validate_session(session_id)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    // Verify ownership before deleting
    if let Some(key) = state
        .service
        .get_api_key(key_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        if key.user_id != user_info.id {
            return Err((StatusCode::FORBIDDEN, "Cannot delete another user's API key".to_string()));
        }
    } else {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    state
        .service
        .delete_api_key(key_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
