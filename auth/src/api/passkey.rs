//! Passkey (WebAuthn) authentication handlers.

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    AuthenticationService, CompleteNewUserPasskeyRegistrationRequest, CompletePasskeyLoginRequest,
    CompletePasskeyRegistrationRequest, LoginResponse, PasskeyInfo, SessionId,
    StartNewUserPasskeyRegistrationResponse, StartPasskeyLoginResponse,
    StartPasskeyRegistrationRequest, StartPasskeyRegistrationResponse,
};

use super::AuthState;

/// Request body for starting new-user passkey registration.
/// Only required when CAPTCHA is enabled on the server.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct StartNewUserRequest {
    /// CAPTCHA response token from the client widget.
    /// Required when the server has CAPTCHA enabled.
    pub captcha_token: Option<String>,
}

#[utoipa::path(
    post,
    path = "/auth/passkey/new-user/start",
    tag = "passkey",
    request_body(content = Option<StartNewUserRequest>, description = "CAPTCHA token (required when CAPTCHA is enabled)"),
    responses(
        (status = 200, description = "Challenge created", body = StartNewUserPasskeyRegistrationResponse),
        (status = 400, description = "Error creating challenge or CAPTCHA failed"),
    )
)]
pub async fn start_new_user_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    body: Option<Json<StartNewUserRequest>>,
) -> Result<Json<StartNewUserPasskeyRegistrationResponse>, (StatusCode, String)> {
    if let Some(captcha) = &state.captcha {
        let token = body
            .as_ref()
            .and_then(|b| b.captcha_token.as_deref())
            .ok_or((
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

    state
        .service
        .start_new_user_passkey_registration()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/new-user/complete",
    tag = "passkey",
    request_body = CompleteNewUserPasskeyRegistrationRequest,
    responses(
        (status = 200, description = "Registration complete", body = LoginResponse),
        (status = 400, description = "Invalid credential"),
    )
)]
pub async fn complete_new_user_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteNewUserPasskeyRegistrationRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .service
        .complete_new_user_passkey_registration(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartRegistrationRequest {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub request: StartPasskeyRegistrationRequest,
}

#[utoipa::path(
    post,
    path = "/auth/passkey/register/start",
    tag = "passkey",
    request_body = StartRegistrationRequest,
    responses(
        (status = 200, description = "Challenge created", body = StartPasskeyRegistrationResponse),
        (status = 400, description = "Invalid session"),
    )
)]
pub async fn start_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<StartRegistrationRequest>,
) -> Result<Json<StartPasskeyRegistrationResponse>, (StatusCode, String)> {
    state
        .service
        .start_passkey_registration(req.session_id, req.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteRegistrationRequest {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub request: CompletePasskeyRegistrationRequest,
}

#[utoipa::path(
    post,
    path = "/auth/passkey/register/complete",
    tag = "passkey",
    request_body = CompleteRegistrationRequest,
    responses(
        (status = 200, description = "Passkey registered", body = PasskeyInfo),
        (status = 400, description = "Invalid credential"),
    )
)]
pub async fn complete_registration<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteRegistrationRequest>,
) -> Result<Json<PasskeyInfo>, (StatusCode, String)> {
    state
        .service
        .complete_passkey_registration(req.session_id, req.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/start",
    tag = "passkey",
    responses(
        (status = 200, description = "Challenge created", body = StartPasskeyLoginResponse),
        (status = 400, description = "Error creating challenge"),
    )
)]
pub async fn start_login<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
) -> Result<Json<StartPasskeyLoginResponse>, (StatusCode, String)> {
    state
        .service
        .start_passkey_login()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/complete",
    tag = "passkey",
    request_body = CompletePasskeyLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid credential"),
    )
)]
pub async fn complete_login<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompletePasskeyLoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .service
        .complete_passkey_login(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}
