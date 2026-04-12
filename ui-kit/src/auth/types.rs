//! Client-side auth types.
//!
//! These mirror the types from payserver-commons/auth for client-side use.
//! The auth crate has server-side dependencies that aren't WASM-compatible,
//! so we define client-side versions here.

use serde::{Deserialize, Serialize};

/// Unique identifier for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub uuid::Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub uuid::Uuid);

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub uuid::Uuid);

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of device for UI categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Browser,
    Desktop,
    Mobile,
    ApiClient,
    #[default]
    Unknown,
}

/// User role for permission checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    #[default]
    User,
}

/// KDF parameters for key derivation.
/// Mirrors crypto::KdfParams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Algorithm identifier (always "argon2id").
    pub algorithm: String,
    /// Memory cost in KB.
    pub memory_kb: u32,
    /// Number of iterations.
    pub iterations: u32,
    /// Degree of parallelism.
    pub parallelism: u32,
    /// Random salt (base64 encoded).
    pub salt: String,
}

/// Encrypted data blob.
/// Mirrors crypto::EncryptedBlob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub ciphertext: String, // Base64 encoded
    pub iv: String,         // Base64 encoded
    pub mac: String,        // Base64 encoded
}

// ============================================================================
// Wallet Authentication Types
// ============================================================================

/// Request to start wallet login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWalletLoginRequest {
    pub address: String,
}

/// Response for starting wallet login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWalletLoginResponse {
    pub challenge_message: String,
    pub user_id: UserId,
}

/// Request to complete wallet login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteWalletLoginRequest {
    pub user_id: UserId,
    pub address: String,
    pub signature: String,
    pub device_id: Option<DeviceId>,
    pub device_name: String,
    pub device_type: DeviceType,
}

/// Request to start new user registration with wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartNewUserWalletRegistrationRequest {
    pub address: String,
    pub wallet_name: String,
}

/// Response for starting new user wallet registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartNewUserWalletRegistrationResponse {
    pub challenge_message: String,
    pub user_id: UserId,
    pub address: String,
}

/// Request to complete new user registration with wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteNewUserWalletRegistrationRequest {
    pub user_id: UserId,
    pub address: String,
    pub signature: String,
    pub wallet_name: String,
    pub kdf_params: KdfParams,
    pub encrypted_symmetric_key: EncryptedBlob,
    pub recovery_verification_hash: String,
    pub device_name: String,
    pub device_type: DeviceType,
}

// ============================================================================
// Passkey Authentication Types
// ============================================================================

/// Response for starting passkey login (discoverable credentials).
/// Contains WebAuthn request options and challenge_id as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartPasskeyLoginResponse {
    /// WebAuthn request options (JSON-serialized RequestChallengeResponse).
    pub options: serde_json::Value,
    /// Challenge ID to send back when completing authentication.
    pub challenge_id: uuid::Uuid,
}

/// Request to complete passkey login (discoverable credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletePasskeyLoginRequest {
    /// Challenge ID from StartPasskeyLoginResponse.
    pub challenge_id: uuid::Uuid,
    /// The credential response from the authenticator (JSON-serialized PublicKeyCredential).
    pub credential: serde_json::Value,
    pub device_id: Option<DeviceId>,
    pub device_name: String,
    pub device_type: DeviceType,
}

/// Response for starting new user passkey registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartNewUserPasskeyRegistrationResponse {
    /// WebAuthn creation options (JSON-serialized CreationChallengeResponse).
    pub options: serde_json::Value,
    pub user_id: UserId,
}

/// Request to complete new user passkey registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteNewUserPasskeyRegistrationRequest {
    pub user_id: UserId,
    /// The credential response from the authenticator (JSON-serialized RegisterPublicKeyCredential).
    pub credential: serde_json::Value,
    pub kdf_params: KdfParams,
    pub encrypted_symmetric_key: EncryptedBlob,
    pub recovery_verification_hash: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub passkey_name: String,
}

// ============================================================================
// Common Response Types
// ============================================================================

/// Data returned to client after successful login or registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_id: SessionId,
    pub device_id: DeviceId,
    pub encrypted_symmetric_key: EncryptedBlob,
    pub kdf_params: KdfParams,
    pub email: Option<String>,
    pub primary_wallet_address: Option<String>,
    pub expires_at: String, // ISO 8601 datetime
}

/// Sanitized user information for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub email: Option<String>,
    pub primary_wallet_address: Option<String>,
    pub created_at: String, // ISO 8601 datetime
    pub last_login_at: Option<String>,
    pub role: Role,
}

// ============================================================================
// CAPTCHA Types
// ============================================================================

/// Server CAPTCHA configuration returned by `GET /auth/captcha/config`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaConfigResponse {
    /// Whether CAPTCHA is enabled on this server.
    pub enabled: bool,
    /// Provider name (e.g. `"turnstile"`).
    pub provider: Option<String>,
    /// Public site key for the widget.
    pub site_key: Option<String>,
}

// ============================================================================
// Client-side Session Types
// ============================================================================

/// Session data stored in localStorage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub session_id: SessionId,
    pub device_id: DeviceId,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub expires_at: String, // ISO 8601 datetime
}

impl StoredSession {
    /// Create a stored session from a login response.
    pub fn from_login_response(response: &LoginResponse) -> Self {
        Self {
            session_id: response.session_id,
            device_id: response.device_id,
            email: response.email.clone(),
            wallet_address: response.primary_wallet_address.clone(),
            expires_at: response.expires_at.clone(),
        }
    }

    /// Check if the session is expired.
    pub fn is_expired(&self) -> bool {
        // Parse ISO 8601 datetime and compare with current time
        // js_sys::Date::now() returns current time in milliseconds
        // js_sys::Date::parse() returns parsed time in milliseconds
        let now = js_sys::Date::now();
        let expires = js_sys::Date::parse(&self.expires_at);
        now > expires
    }
}
