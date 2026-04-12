//! Auth API client methods.

use crate::hooks::use_api::{ApiClient, ApiError};

use super::types::{
    CaptchaConfigResponse, CompleteNewUserPasskeyRegistrationRequest,
    CompleteNewUserWalletRegistrationRequest, CompletePasskeyLoginRequest,
    CompleteWalletLoginRequest, LoginResponse, StartNewUserPasskeyRegistrationResponse,
    StartNewUserWalletRegistrationResponse, StartPasskeyLoginResponse, StartWalletLoginRequest,
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
        captcha_token: Option<&str>,
    ) -> Result<StartNewUserWalletRegistrationResponse, ApiError> {
        #[derive(serde::Serialize)]
        struct Req<'a> {
            address: &'a str,
            wallet_name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha_token: Option<&'a str>,
        }
        let req = Req {
            address,
            wallet_name,
            captcha_token,
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
    // Passkey Authentication (Discoverable Credentials)
    // ========================================================================

    /// Start passkey login flow using discoverable credentials.
    /// No email required - the authenticator will present available passkeys.
    /// Returns WebAuthn request options and a challenge_id for the authenticator.
    pub async fn start_passkey_login(&self) -> Result<StartPasskeyLoginResponse, ApiError> {
        self.post("/auth/passkey/login/start", &()).await
    }

    /// Complete passkey login with authenticator response.
    pub async fn complete_passkey_login(
        &self,
        req: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse, ApiError> {
        self.post("/auth/passkey/login/complete", &req).await
    }

    /// Start new user passkey registration.
    /// No email required - user_id is generated server-side.
    /// Returns WebAuthn creation options for the authenticator.
    pub async fn start_passkey_register(
        &self,
        captcha_token: Option<&str>,
    ) -> Result<StartNewUserPasskeyRegistrationResponse, ApiError> {
        #[derive(serde::Serialize)]
        struct Req<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha_token: Option<&'a str>,
        }
        let req = Req { captcha_token };
        self.post("/auth/passkey/new-user/start", &req).await
    }

    /// Get the server's CAPTCHA configuration.
    pub async fn get_captcha_config(&self) -> Result<CaptchaConfigResponse, ApiError> {
        self.get("/auth/captcha/config").await
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
