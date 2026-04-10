//! Pluggable CAPTCHA verification for registration endpoints.
//!
//! Defines the [`CaptchaProvider`] trait that abstracts over different CAPTCHA services.
//! Enable the `captcha` feature to get the built-in [`CloudflareTurnstile`] implementation.
//!
//! # Adding a new provider
//!
//! Implement [`CaptchaProvider`] for your type:
//!
//! ```rust,ignore
//! use auth::captcha::{CaptchaProvider, CaptchaError};
//!
//! struct MyProvider { /* ... */ }
//!
//! #[async_trait::async_trait]
//! impl CaptchaProvider for MyProvider {
//!     async fn verify(&self, token: &str) -> Result<(), CaptchaError> { /* ... */ }
//!     fn site_key(&self) -> &str { /* ... */ }
//!     fn provider_name(&self) -> &str { "my-provider" }
//! }
//! ```

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// Errors that can occur during CAPTCHA verification.
#[derive(Debug, Error)]
pub enum CaptchaError {
    /// The CAPTCHA token was rejected by the provider.
    #[error("CAPTCHA verification failed")]
    VerificationFailed,

    /// The upstream CAPTCHA service is unreachable or returned an unexpected response.
    #[error("CAPTCHA service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Trait for CAPTCHA verification providers.
///
/// Each provider wraps a specific service (Cloudflare Turnstile, hCaptcha, reCAPTCHA, etc.)
/// and exposes a uniform `verify` interface. The server selects the provider at startup
/// based on the `CAPTCHA_PROVIDER` environment variable.
#[async_trait]
pub trait CaptchaProvider: Send + Sync {
    /// Verify a CAPTCHA response token submitted by the client.
    ///
    /// Returns `Ok(())` if the token is valid. Returns an error if verification
    /// fails or the upstream service is unreachable.
    async fn verify(&self, token: &str) -> Result<(), CaptchaError>;

    /// The public site key for client-side widget rendering.
    fn site_key(&self) -> &str;

    /// Provider identifier (e.g. `"turnstile"`, `"recaptcha"`, `"hcaptcha"`).
    fn provider_name(&self) -> &str;
}

/// CAPTCHA configuration returned to the frontend so it can render the correct widget.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CaptchaConfigResponse {
    /// Whether CAPTCHA is enabled on this server.
    pub enabled: bool,
    /// Provider name (e.g. `"turnstile"`). `None` when disabled.
    pub provider: Option<String>,
    /// Public site key for the widget. `None` when disabled.
    pub site_key: Option<String>,
}

// ---------------------------------------------------------------------------
// Cloudflare Turnstile implementation (requires `captcha` feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "captcha")]
mod turnstile {
    use super::*;
    use serde::Deserialize;

    /// Cloudflare Turnstile CAPTCHA provider.
    ///
    /// Privacy-first CAPTCHA that doesn't use tracking cookies.
    /// See <https://developers.cloudflare.com/turnstile/>.
    pub struct CloudflareTurnstile {
        secret_key: String,
        site_key: String,
        client: reqwest::Client,
    }

    #[derive(Deserialize)]
    struct SiteverifyResponse {
        success: bool,
        #[serde(rename = "error-codes", default)]
        error_codes: Vec<String>,
    }

    impl CloudflareTurnstile {
        /// Create a new Turnstile provider.
        ///
        /// - `secret_key`: Server-side secret from the Cloudflare dashboard.
        /// - `site_key`: Client-side site key for widget rendering.
        pub fn new(secret_key: String, site_key: String) -> Self {
            Self {
                secret_key,
                site_key,
                client: reqwest::Client::new(),
            }
        }
    }

    #[async_trait]
    impl CaptchaProvider for CloudflareTurnstile {
        async fn verify(&self, token: &str) -> Result<(), CaptchaError> {
            let resp = self
                .client
                .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
                .form(&[
                    ("secret", self.secret_key.as_str()),
                    ("response", token),
                ])
                .send()
                .await
                .map_err(|e| CaptchaError::ServiceUnavailable(e.to_string()))?;

            if !resp.status().is_success() {
                return Err(CaptchaError::ServiceUnavailable(format!(
                    "Turnstile returned HTTP {}",
                    resp.status()
                )));
            }

            let result: SiteverifyResponse = resp
                .json()
                .await
                .map_err(|e| CaptchaError::ServiceUnavailable(e.to_string()))?;

            if result.success {
                Ok(())
            } else {
                tracing::debug!(error_codes = ?result.error_codes, "Turnstile verification failed");
                Err(CaptchaError::VerificationFailed)
            }
        }

        fn site_key(&self) -> &str {
            &self.site_key
        }

        fn provider_name(&self) -> &str {
            "turnstile"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn turnstile_provider_name() {
            let provider = CloudflareTurnstile::new("secret".into(), "site_key".into());
            assert_eq!(provider.provider_name(), "turnstile");
            assert_eq!(provider.site_key(), "site_key");
        }
    }
}

#[cfg(feature = "captcha")]
pub use turnstile::CloudflareTurnstile;
