//! Auth API client methods.

use crate::hooks::use_api::{ApiClient, ApiError};

use super::types::{
    CompleteNewUserPasskeyRegistrationRequest, CompleteNewUserWalletRegistrationRequest,
    CompletePasskeyLoginRequest, CompleteWalletLoginRequest, LoginResponse,
    StartNewUserPasskeyRegistrationResponse, StartNewUserPasskeyRequest,
    StartNewUserWalletRegistrationRequest, StartNewUserWalletRegistrationResponse,
    StartPasskeyLoginRequest, StartPasskeyLoginResponse, StartWalletLoginRequest,
    StartWalletLoginResponse, UserInfo,
};

impl ApiClient {
    // ========================================================================
    // Wallet Authentication
    // ========================================================================

    /// Start wallet login flow.
    /// Returns a challenge message that must be signed by the wallet.
    pub async fn start_wallet_login(
        &self,
        address: &str,
    ) -> Result<StartWalletLoginResponse, ApiError> {
        let req = StartWalletLoginRequest {
            address: address.to_string(),
        };
        self.post("/auth/wallet/login/start", &req).await
    }

    /// Complete wallet login with signed challenge.
    pub async fn complete_wallet_login(
        &self,
        req: CompleteWalletLoginRequest,
    ) -> Result<LoginResponse, ApiError> {
        self.post("/auth/wallet/login/complete", &req).await
    }

    /// Start new user wallet registration.
    /// Returns a challenge message that must be signed by the wallet.
    pub async fn start_wallet_register(
        &self,
        address: &str,
        wallet_name: &str,
    ) -> Result<StartNewUserWalletRegistrationResponse, ApiError> {
        let req = StartNewUserWalletRegistrationRequest {
            address: address.to_string(),
            wallet_name: wallet_name.to_string(),
        };
        self.post("/auth/wallet/new-user/start", &req).await
    }

    /// Complete new user wallet registration with signed challenge.
    pub async fn complete_wallet_register(
        &self,
        req: CompleteNewUserWalletRegistrationRequest,
    ) -> Result<LoginResponse, ApiError> {
        self.post("/auth/wallet/new-user/complete", &req).await
    }

    // ========================================================================
    // Passkey Authentication
    // ========================================================================

    /// Start passkey login flow.
    /// Returns WebAuthn request options for the authenticator.
    pub async fn start_passkey_login(
        &self,
        email: &str,
    ) -> Result<StartPasskeyLoginResponse, ApiError> {
        let req = StartPasskeyLoginRequest {
            email: email.to_string(),
        };
        self.post("/auth/passkey/login/start", &req).await
    }

    /// Complete passkey login with authenticator response.
    pub async fn complete_passkey_login(
        &self,
        req: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse, ApiError> {
        self.post("/auth/passkey/login/complete", &req).await
    }

    /// Start new user passkey registration.
    /// Returns WebAuthn creation options for the authenticator.
    pub async fn start_passkey_register(
        &self,
        email: &str,
    ) -> Result<StartNewUserPasskeyRegistrationResponse, ApiError> {
        let req = StartNewUserPasskeyRequest {
            email: email.to_string(),
        };
        self.post("/auth/passkey/new-user/start", &req).await
    }

    /// Complete new user passkey registration with authenticator response.
    pub async fn complete_passkey_register(
        &self,
        req: CompleteNewUserPasskeyRegistrationRequest,
    ) -> Result<LoginResponse, ApiError> {
        self.post("/auth/passkey/new-user/complete", &req).await
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    /// Get current user information.
    /// Requires authentication (session token).
    pub async fn get_current_user(&self) -> Result<UserInfo, ApiError> {
        self.get("/auth/me").await
    }

    /// Logout and invalidate current session.
    pub async fn logout(&self) -> Result<(), ApiError> {
        self.post::<(), _>("/auth/logout", &()).await
    }
}
