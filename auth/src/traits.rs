//! Authentication service traits.
//!
//! These traits define the interface for authentication services, enabling
//! dependency inversion and testability. The concrete implementation is
//! `WebAuthnAuthService`.

use async_trait::async_trait;

use crate::error::Result;
use crate::models::{
    CompleteNewUserPasskeyRegistrationRequest, CompleteNewUserWalletRegistrationRequest,
    CompletePasskeyLoginRequest, CompletePasskeyRegistrationRequest, CompleteRecoveryRequest,
    CompleteWalletLoginRequest, CompleteWalletRegistrationRequest, DeviceId, DeviceInfo,
    LoginResponse, PasskeyId, PasskeyInfo, Session, SessionId, StartNewUserPasskeyRegistrationResponse,
    StartNewUserWalletRegistrationRequest, StartNewUserWalletRegistrationResponse,
    StartPasskeyLoginResponse, StartPasskeyRegistrationRequest, StartPasskeyRegistrationResponse,
    StartRecoveryRequest, StartWalletLoginRequest, StartWalletLoginResponse,
    StartWalletRegistrationRequest, StartWalletRegistrationResponse, UserInfo, WalletCredentialId,
    WalletInfo,
};

/// Session management service.
///
/// Handles session validation, logout, and cleanup.
#[async_trait]
pub trait SessionService: Send + Sync {
    /// Validate a session and return user info.
    ///
    /// Checks both absolute expiration and idle timeout.
    async fn validate_session(&self, session_id: SessionId) -> Result<(UserInfo, Session)>;

    /// Logout - invalidate a session.
    async fn logout(&self, session_id: SessionId) -> Result<()>;

    /// Logout from all devices.
    ///
    /// Requires a valid session to prove ownership.
    async fn logout_all(&self, session_id: SessionId) -> Result<()>;

    /// Clean up stale sessions.
    ///
    /// Removes expired and idle-timed-out sessions.
    async fn cleanup_stale_sessions(&self) -> Result<u64>;
}

/// Passkey authentication service.
///
/// Handles passkey-based authentication flows including registration, login,
/// and credential management.
#[async_trait]
pub trait PasskeyAuthService: Send + Sync {
    // =========================================================================
    // New User Registration
    // =========================================================================

    /// Start new user registration with passkey.
    ///
    /// Returns a WebAuthn challenge for the client to pass to the authenticator.
    /// No email required - passkey-only registration.
    async fn start_new_user_passkey_registration(
        &self,
    ) -> Result<StartNewUserPasskeyRegistrationResponse>;

    /// Complete new user registration with passkey.
    ///
    /// Creates the user account and passkey credential.
    async fn complete_new_user_passkey_registration(
        &self,
        request: CompleteNewUserPasskeyRegistrationRequest,
    ) -> Result<LoginResponse>;

    // =========================================================================
    // Login
    // =========================================================================

    /// Start passkey login using discoverable credentials.
    ///
    /// Returns a WebAuthn challenge for authentication.
    /// No email required - the authenticator returns the user identity.
    async fn start_passkey_login(&self) -> Result<StartPasskeyLoginResponse>;

    /// Complete passkey login.
    ///
    /// Validates the credential and creates a session.
    async fn complete_passkey_login(
        &self,
        request: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse>;

    // =========================================================================
    // Passkey Management (requires session)
    // =========================================================================

    /// Start adding a passkey to an existing account.
    async fn start_passkey_registration(
        &self,
        session_id: SessionId,
        request: StartPasskeyRegistrationRequest,
    ) -> Result<StartPasskeyRegistrationResponse>;

    /// Complete adding a passkey to an existing account.
    async fn complete_passkey_registration(
        &self,
        session_id: SessionId,
        request: CompletePasskeyRegistrationRequest,
    ) -> Result<PasskeyInfo>;

    /// Get all passkeys for the current user.
    async fn get_passkeys(&self, session_id: SessionId) -> Result<Vec<PasskeyInfo>>;

    /// Revoke a passkey.
    async fn revoke_passkey(&self, session_id: SessionId, passkey_id: PasskeyId) -> Result<()>;
}

/// Wallet authentication service.
///
/// Handles Ethereum wallet-based authentication flows including registration,
/// login, and wallet management.
#[async_trait]
pub trait WalletAuthService: Send + Sync {
    // =========================================================================
    // New User Registration
    // =========================================================================

    /// Start new user registration with wallet.
    async fn start_new_user_wallet_registration(
        &self,
        request: StartNewUserWalletRegistrationRequest,
    ) -> Result<StartNewUserWalletRegistrationResponse>;

    /// Complete new user registration with wallet.
    async fn complete_new_user_wallet_registration(
        &self,
        request: CompleteNewUserWalletRegistrationRequest,
    ) -> Result<LoginResponse>;

    // =========================================================================
    // Login
    // =========================================================================

    /// Start wallet login.
    async fn start_wallet_login(
        &self,
        request: StartWalletLoginRequest,
    ) -> Result<StartWalletLoginResponse>;

    /// Complete wallet login.
    async fn complete_wallet_login(
        &self,
        request: CompleteWalletLoginRequest,
    ) -> Result<LoginResponse>;

    // =========================================================================
    // Wallet Management (requires session)
    // =========================================================================

    /// Start adding a wallet to an existing account.
    async fn start_wallet_registration(
        &self,
        session_id: SessionId,
        request: StartWalletRegistrationRequest,
    ) -> Result<StartWalletRegistrationResponse>;

    /// Complete adding a wallet to an existing account.
    async fn complete_wallet_registration(
        &self,
        session_id: SessionId,
        request: CompleteWalletRegistrationRequest,
    ) -> Result<WalletInfo>;

    /// Get all wallets for the current user.
    async fn get_wallets(&self, session_id: SessionId) -> Result<Vec<WalletInfo>>;

    /// Revoke a wallet.
    async fn revoke_wallet(
        &self,
        session_id: SessionId,
        wallet_id: WalletCredentialId,
    ) -> Result<()>;
}

/// Device management service.
///
/// Handles device listing and revocation.
#[async_trait]
pub trait DeviceService: Send + Sync {
    /// Get all active devices for the current user.
    async fn get_devices(&self, session_id: SessionId) -> Result<Vec<DeviceInfo>>;

    /// Revoke a device.
    ///
    /// Cannot revoke the device associated with the current session.
    async fn revoke_device(&self, session_id: SessionId, device_id: DeviceId) -> Result<()>;
}

/// Account recovery service.
///
/// Handles BIP39 mnemonic-based account recovery.
#[async_trait]
pub trait RecoveryService: Send + Sync {
    /// Start account recovery.
    ///
    /// Verifies the mnemonic and returns a passkey registration challenge.
    async fn start_account_recovery(
        &self,
        request: StartRecoveryRequest,
    ) -> Result<StartPasskeyRegistrationResponse>;

    /// Complete account recovery.
    ///
    /// Registers new passkey, revokes old credentials, and creates session.
    async fn complete_account_recovery(
        &self,
        identifier: &str,
        request: CompleteRecoveryRequest,
    ) -> Result<LoginResponse>;
}

/// Combined authentication service trait.
///
/// Convenience trait that combines all authentication capabilities.
/// Use this when you need full authentication functionality.
pub trait AuthenticationService:
    SessionService + PasskeyAuthService + WalletAuthService + DeviceService + RecoveryService
{
}

// Blanket implementation for any type implementing all traits
impl<T> AuthenticationService for T where
    T: SessionService + PasskeyAuthService + WalletAuthService + DeviceService + RecoveryService
{
}
