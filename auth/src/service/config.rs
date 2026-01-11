//! Authentication service configuration.

use chrono::Duration;

/// Configuration for the auth service.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Maximum failed login attempts before lockout.
    pub max_failed_attempts: u32,

    /// Lockout duration after max failed attempts.
    pub lockout_duration: Duration,

    /// Session expiration time (absolute timeout).
    pub session_duration: Duration,

    /// Session idle timeout. If None, idle timeout is disabled.
    /// Session expires if no activity for this duration.
    pub idle_timeout: Option<Duration>,

    /// Maximum devices per user.
    pub max_devices_per_user: u32,

    /// Maximum passkeys per user.
    pub max_passkeys_per_user: u32,

    /// Maximum wallets per user.
    pub max_wallets_per_user: u32,

    /// Wallet challenge expiration time.
    pub wallet_challenge_duration: Duration,

    /// WebAuthn Relying Party ID (typically the domain, e.g., "example.com").
    pub rp_id: String,

    /// WebAuthn Relying Party name (displayed to user, e.g., "Example App").
    pub rp_name: String,

    /// WebAuthn Relying Party origin (the full URL, e.g., "https://example.com").
    pub rp_origin: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration: Duration::minutes(15),
            session_duration: Duration::hours(24),
            idle_timeout: Some(Duration::hours(2)), // 2 hour idle timeout
            max_devices_per_user: 10,
            max_passkeys_per_user: 10,
            max_wallets_per_user: 10,
            wallet_challenge_duration: Duration::minutes(10),
            rp_id: "localhost".to_string(),
            rp_name: "PayServer".to_string(),
            rp_origin: "http://localhost:8080".to_string(),
        }
    }
}
