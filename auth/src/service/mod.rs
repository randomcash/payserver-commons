//! Authentication service with business logic.
//!
//! This service provides passkey-only authentication with BIP39 mnemonic recovery.
//! Password-based authentication has been removed for better security.

use std::sync::Arc;

use url::Url;
use webauthn_rs::Webauthn;
use webauthn_rs::prelude::*;

use crate::error::Result;
use crate::models::{
    CompleteNewUserPasskeyRegistrationRequest, CompleteNewUserWalletRegistrationRequest,
    CompletePasskeyLoginRequest, CompletePasskeyRegistrationRequest, CompleteRecoveryRequest,
    CompleteWalletLoginRequest, CompleteWalletRegistrationRequest, DeviceId, DeviceInfo,
    LoginResponse, PasskeyId, PasskeyInfo, Session, SessionId,
    StartNewUserPasskeyRegistrationResponse, StartNewUserWalletRegistrationRequest,
    StartNewUserWalletRegistrationResponse, StartPasskeyLoginResponse,
    StartPasskeyRegistrationRequest, StartPasskeyRegistrationResponse, StartRecoveryRequest,
    StartWalletLoginRequest, StartWalletLoginResponse, StartWalletRegistrationRequest,
    StartWalletRegistrationResponse, UserInfo, WalletCredentialId, WalletInfo,
};
use crate::repository::{
    ChallengeRepository, DeviceRepository, PasskeyRepository, SessionRepository, UserRepository,
    WalletRepository,
};
use crate::traits::{
    DeviceService, PasskeyAuthService, RecoveryService, SessionService, WalletAuthService,
};

// Sub-modules
mod config;
mod device;
mod passkey;
mod recovery;
mod session;
mod validation;
mod wallet;

// Re-exports
pub use config::AuthConfig;

/// Authentication service handling registration, login, and session management.
///
/// This is the concrete WebAuthn-based implementation of the authentication traits.
pub struct WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository,
{
    repo: Arc<R>,
    config: AuthConfig,
    webauthn: Arc<Webauthn>,
}

/// Type alias for backward compatibility.
pub type AuthService<R> = WebAuthnAuthService<R>;

/// Build a Webauthn instance from config.
fn build_webauthn(config: &AuthConfig) -> std::result::Result<Webauthn, WebauthnError> {
    let rp_origin = Url::parse(&config.rp_origin).map_err(|_| WebauthnError::Configuration)?;
    let builder = WebauthnBuilder::new(&config.rp_id, &rp_origin)?.rp_name(&config.rp_name);
    builder.build()
}

impl<R> WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository,
{
    /// Create a new auth service with the given repository and default config.
    ///
    /// # Panics
    /// Panics if the WebAuthn configuration is invalid.
    pub fn new(repo: Arc<R>) -> Self {
        let config = AuthConfig::default();
        let webauthn = build_webauthn(&config).expect("Invalid WebAuthn config");
        Self {
            repo,
            config,
            webauthn: Arc::new(webauthn),
        }
    }

    /// Create a new auth service with custom config.
    ///
    /// # Panics
    /// Panics if the WebAuthn configuration is invalid.
    pub fn with_config(repo: Arc<R>, config: AuthConfig) -> Self {
        let webauthn = build_webauthn(&config).expect("Invalid WebAuthn config");
        Self {
            repo,
            config,
            webauthn: Arc::new(webauthn),
        }
    }
}

// =============================================================================
// Trait Implementations
// =============================================================================

#[async_trait::async_trait]
impl<R> SessionService for WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository
        + Send
        + Sync
        + 'static,
{
    async fn validate_session(&self, session_id: SessionId) -> Result<(UserInfo, Session)> {
        WebAuthnAuthService::validate_session(self, session_id).await
    }

    async fn logout(&self, session_id: SessionId) -> Result<()> {
        WebAuthnAuthService::logout(self, session_id).await
    }

    async fn logout_all(&self, session_id: SessionId) -> Result<()> {
        WebAuthnAuthService::logout_all(self, session_id).await
    }

    async fn cleanup_stale_sessions(&self) -> Result<u64> {
        WebAuthnAuthService::cleanup_stale_sessions(self).await
    }
}

#[async_trait::async_trait]
impl<R> DeviceService for WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository
        + Send
        + Sync
        + 'static,
{
    async fn get_devices(&self, session_id: SessionId) -> Result<Vec<DeviceInfo>> {
        WebAuthnAuthService::get_devices(self, session_id).await
    }

    async fn revoke_device(&self, session_id: SessionId, device_id: DeviceId) -> Result<()> {
        WebAuthnAuthService::revoke_device(self, session_id, device_id).await
    }
}

#[async_trait::async_trait]
impl<R> RecoveryService for WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository
        + Send
        + Sync
        + 'static,
{
    async fn start_account_recovery(
        &self,
        request: StartRecoveryRequest,
    ) -> Result<StartPasskeyRegistrationResponse> {
        WebAuthnAuthService::start_account_recovery(self, request).await
    }

    async fn complete_account_recovery(
        &self,
        identifier: &str,
        request: CompleteRecoveryRequest,
    ) -> Result<LoginResponse> {
        WebAuthnAuthService::complete_account_recovery(self, identifier, request).await
    }
}

#[async_trait::async_trait]
impl<R> PasskeyAuthService for WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository
        + Send
        + Sync
        + 'static,
{
    async fn start_new_user_passkey_registration(
        &self,
    ) -> Result<StartNewUserPasskeyRegistrationResponse> {
        WebAuthnAuthService::start_new_user_passkey_registration(self).await
    }

    async fn complete_new_user_passkey_registration(
        &self,
        request: CompleteNewUserPasskeyRegistrationRequest,
    ) -> Result<LoginResponse> {
        WebAuthnAuthService::complete_new_user_passkey_registration(self, request).await
    }

    async fn start_passkey_login(&self) -> Result<StartPasskeyLoginResponse> {
        WebAuthnAuthService::start_passkey_login(self).await
    }

    async fn complete_passkey_login(
        &self,
        request: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse> {
        WebAuthnAuthService::complete_passkey_login(self, request).await
    }

    async fn start_passkey_registration(
        &self,
        session_id: SessionId,
        request: StartPasskeyRegistrationRequest,
    ) -> Result<StartPasskeyRegistrationResponse> {
        WebAuthnAuthService::start_passkey_registration(self, session_id, request).await
    }

    async fn complete_passkey_registration(
        &self,
        session_id: SessionId,
        request: CompletePasskeyRegistrationRequest,
    ) -> Result<PasskeyInfo> {
        WebAuthnAuthService::complete_passkey_registration(self, session_id, request).await
    }

    async fn get_passkeys(&self, session_id: SessionId) -> Result<Vec<PasskeyInfo>> {
        WebAuthnAuthService::get_passkeys(self, session_id).await
    }

    async fn revoke_passkey(&self, session_id: SessionId, passkey_id: PasskeyId) -> Result<()> {
        WebAuthnAuthService::revoke_passkey(self, session_id, passkey_id).await
    }
}

#[async_trait::async_trait]
impl<R> WalletAuthService for WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository
        + Send
        + Sync
        + 'static,
{
    async fn start_new_user_wallet_registration(
        &self,
        request: StartNewUserWalletRegistrationRequest,
    ) -> Result<StartNewUserWalletRegistrationResponse> {
        WebAuthnAuthService::start_new_user_wallet_registration(self, request).await
    }

    async fn complete_new_user_wallet_registration(
        &self,
        request: CompleteNewUserWalletRegistrationRequest,
    ) -> Result<LoginResponse> {
        WebAuthnAuthService::complete_new_user_wallet_registration(self, request).await
    }

    async fn start_wallet_login(
        &self,
        request: StartWalletLoginRequest,
    ) -> Result<StartWalletLoginResponse> {
        WebAuthnAuthService::start_wallet_login(self, request).await
    }

    async fn complete_wallet_login(
        &self,
        request: CompleteWalletLoginRequest,
    ) -> Result<LoginResponse> {
        WebAuthnAuthService::complete_wallet_login(self, request).await
    }

    async fn start_wallet_registration(
        &self,
        session_id: SessionId,
        request: StartWalletRegistrationRequest,
    ) -> Result<StartWalletRegistrationResponse> {
        WebAuthnAuthService::start_wallet_registration(self, session_id, request).await
    }

    async fn complete_wallet_registration(
        &self,
        session_id: SessionId,
        request: CompleteWalletRegistrationRequest,
    ) -> Result<WalletInfo> {
        WebAuthnAuthService::complete_wallet_registration(self, session_id, request).await
    }

    async fn get_wallets(&self, session_id: SessionId) -> Result<Vec<WalletInfo>> {
        WebAuthnAuthService::get_wallets(self, session_id).await
    }

    async fn revoke_wallet(
        &self,
        session_id: SessionId,
        wallet_id: WalletCredentialId,
    ) -> Result<()> {
        WebAuthnAuthService::revoke_wallet(self, session_id, wallet_id).await
    }
}

#[cfg(test)]
mod tests;
