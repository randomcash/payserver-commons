//! Account recovery handlers.

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    AuthenticationService, CompleteRecoveryRequest, LoginResponse, StartRecoveryRequest,
    StartRecoveryResponse,
};

use super::AuthState;

/// Request body for starting account recovery.
///
/// Wraps [`StartRecoveryRequest`] with an optional CAPTCHA token. The token is
/// verified through the configured [`crate::captcha::CaptchaProvider`], so it
/// is provider-agnostic - Cloudflare Turnstile, a self-hosted altcha, or any
/// other implementation, chosen by whatever the server was built with.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartRecoveryRequestBody {
    /// CAPTCHA response token from the client widget.
    /// Required when the server has CAPTCHA enabled.
    pub captcha_token: Option<String>,

    #[serde(flatten)]
    pub request: StartRecoveryRequest,
}

#[utoipa::path(
    post,
    path = "/auth/recovery/start",
    tag = "recovery",
    request_body = StartRecoveryRequestBody,
    responses(
        (status = 200, description = "Recovery challenge, KDF params and wrapped account key", body = StartRecoveryResponse),
        (status = 400, description = "Invalid identifier or recovery hash"),
    )
)]
pub async fn start_recovery<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(body): Json<StartRecoveryRequestBody>,
) -> Result<Json<StartRecoveryResponse>, (StatusCode, String)> {
    // Unauthenticated endpoint reachable with a public identifier, and it no
    // longer defends itself by locking the victim's account (RCS-204). CAPTCHA
    // is what makes automated abuse expensive to the caller instead.
    if let Some(captcha) = &state.captcha {
        let token = body.captcha_token.as_deref().ok_or((
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
        .start_account_recovery(body.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteRecoveryRequestBody {
    #[schema(example = "user@example.com")]
    pub identifier: String,
    #[serde(flatten)]
    pub request: CompleteRecoveryRequest,
}

#[utoipa::path(
    post,
    path = "/auth/recovery/complete",
    tag = "recovery",
    request_body = CompleteRecoveryRequestBody,
    responses(
        (status = 200, description = "Recovery complete", body = LoginResponse),
        (status = 400, description = "Invalid credential"),
    )
)]
pub async fn complete_recovery<A: AuthenticationService>(
    State(state): State<AuthState<A>>,
    Json(req): Json<CompleteRecoveryRequestBody>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .service
        .complete_account_recovery(&req.identifier, req.request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}
