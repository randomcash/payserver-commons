//! Ethereum wallet (EIP-191) authentication handlers.

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    AuthenticationService, CompleteNewUserWalletRegistrationRequest, CompleteWalletLoginRequest,
    CompleteWalletRegistrationRequest, LoginResponse, SessionId,
    StartNewUserWalletRegistrationRequest, StartNewUserWalletRegistrationResponse,
    StartWalletLoginRequest, StartWalletLoginResponse, StartWalletRegistrationRequest,
    StartWalletRegistrationResponse, WalletInfo,
};

use super::AuthState;

/// Request body for starting new-user wallet registration.
/// Wraps the standard wallet request with an optional CAPTCHA token.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartNewUserRequest {
    /// CAPTCHA response token from the client widget.
    /// Required when the server has CAPTCHA enabled.
    pub captcha_token: Option<String>,
    /// Wallet address for the new account (will be checksummed by server).
    pub address: String,
    /// Human-readable name for this wallet.
    pub wallet_name: String,
}

#[utoipa::path(
    post,
    path = "/auth/wallet/new-user/start",
    tag = "wallet",
    request_body = StartNewUserRequest,
    responses(
        (status = 200, description = "Challenge created", body = StartNewUserWalletRegistrationResponse),
        (status = 400, description = "Invalid address, user exists, or CAPTCHA failed"),
    )
)]
pub async fn start_new_user_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<StartNewUserRequest>,
) -> Result<Json<StartNewUserWalletRegistrationResponse>, (StatusCode, String)> {
    if let Some(captcha) = &state.captcha {
        let token = req.captcha_token.as_deref().ok_or((
            StatusCode::BAD_REQUEST,
            "CAPTCHA token required".to_string(),
        ))?;
        captcha.verify(token).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("CAPTCHA verification failed: {e}"),
            )
        })?;
    }

    let wallet_req = StartNewUserWalletRegistrationRequest {
        address: req.address,
        wallet_name: req.wallet_name,
    };

    state
        .service
        .start_new_user_wallet_registration(wallet_req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/wallet/new-user/complete",
    tag = "wallet",
    request_body = CompleteNewUserWalletRegistrationRequest,
    responses(
        (status = 200, description = "Registration complete", body = LoginResponse),
        (status = 400, description = "Invalid signature"),
    )
)]
pub async fn complete_new_user_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteNewUserWalletRegistrationRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .service
        .complete_new_user_wallet_registration(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartRegistrationRequest {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub request: StartWalletRegistrationRequest,
}

#[utoipa::path(
    post,
    path = "/auth/wallet/register/start",
    tag = "wallet",
    request_body = StartRegistrationRequest,
    responses(
        (status = 200, description = "Challenge created", body = StartWalletRegistrationResponse),
        (status = 400, description = "Invalid session"),
    )
)]
pub async fn start_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<StartRegistrationRequest>,
) -> Result<Json<StartWalletRegistrationResponse>, (StatusCode, String)> {
    state
        .service
        .start_wallet_registration(req.session_id, req.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteRegistrationRequest {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub request: CompleteWalletRegistrationRequest,
}

#[utoipa::path(
    post,
    path = "/auth/wallet/register/complete",
    tag = "wallet",
    request_body = CompleteRegistrationRequest,
    responses(
        (status = 200, description = "Wallet registered", body = WalletInfo),
        (status = 400, description = "Invalid signature"),
    )
)]
pub async fn complete_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteRegistrationRequest>,
) -> Result<Json<WalletInfo>, (StatusCode, String)> {
    state
        .service
        .complete_wallet_registration(req.session_id, req.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/wallet/login/start",
    tag = "wallet",
    request_body = StartWalletLoginRequest,
    responses(
        (status = 200, description = "Challenge created", body = StartWalletLoginResponse),
        (status = 400, description = "Wallet not registered"),
    )
)]
pub async fn start_login<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<StartWalletLoginRequest>,
) -> Result<Json<StartWalletLoginResponse>, (StatusCode, String)> {
    state
        .service
        .start_wallet_login(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/wallet/login/complete",
    tag = "wallet",
    request_body = CompleteWalletLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid signature"),
    )
)]
pub async fn complete_login<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteWalletLoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .service
        .complete_wallet_login(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}
