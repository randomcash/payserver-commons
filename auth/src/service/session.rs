//! Session management methods.

use chrono::Utc;

use crate::error::{AuthError, Result};
#[cfg(feature = "metrics")]
use crate::metrics;
use crate::models::{Session, SessionId, UserInfo};
use crate::repository::{
    ChallengeRepository, DeviceRepository, PasskeyRepository, SessionRepository, UserRepository,
    WalletRepository,
};

use super::WebAuthnAuthService;

impl<R> WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository,
{
    /// Validate a session and return sanitized user info.
    ///
    /// Checks both absolute expiration and idle timeout (if configured).
    /// Returns UserInfo (without sensitive fields) instead of full User.
    pub async fn validate_session(&self, session_id: SessionId) -> Result<(UserInfo, Session)> {
        let session = self
            .repo
            .get_session(session_id)
            .await?
            .ok_or(AuthError::SessionInvalid)?;

        // Check absolute expiration
        if session.is_expired() {
            self.repo.delete_session(session_id).await?;
            return Err(AuthError::SessionInvalid);
        }

        // Check idle timeout if configured
        if let Some(idle_timeout) = self.config.idle_timeout {
            let idle_deadline = session.last_activity_at + idle_timeout;
            if Utc::now() > idle_deadline {
                self.repo.delete_session(session_id).await?;
                return Err(AuthError::SessionInvalid);
            }
        }

        let user = self
            .repo
            .get_user(session.user_id)
            .await?
            .ok_or(AuthError::SessionInvalid)?;

        // Update session activity
        let mut updated_session = session.clone();
        updated_session.touch();
        self.repo.update_session(&updated_session).await?;

        Ok((UserInfo::from(&user), updated_session))
    }

    /// Logout - invalidate a session.
    pub async fn logout(&self, session_id: SessionId) -> Result<()> {
        self.repo.delete_session(session_id).await?;
        #[cfg(feature = "metrics")]
        metrics::record_user_logout();
        Ok(())
    }

    /// Logout from all devices.
    ///
    /// Requires a valid session to prove ownership. The provided session
    /// will also be invalidated along with all other sessions.
    pub async fn logout_all(&self, session_id: SessionId) -> Result<()> {
        // Validate the caller's session to get user_id
        let session = self
            .repo
            .get_session(session_id)
            .await?
            .ok_or(AuthError::SessionInvalid)?;

        // Check absolute expiration
        if session.is_expired() {
            self.repo.delete_session(session_id).await?;
            return Err(AuthError::SessionInvalid);
        }

        // Check idle timeout if configured
        if let Some(idle_timeout) = self.config.idle_timeout {
            let idle_deadline = session.last_activity_at + idle_timeout;
            if Utc::now() > idle_deadline {
                self.repo.delete_session(session_id).await?;
                return Err(AuthError::SessionInvalid);
            }
        }

        // Delete all sessions for this user (including the current one)
        self.repo.delete_all_sessions_for_user(session.user_id).await?;
        #[cfg(feature = "metrics")]
        metrics::record_user_logout();
        Ok(())
    }

    /// Clean up stale sessions (call periodically).
    ///
    /// Removes sessions that are either:
    /// - Absolutely expired (past expires_at)
    /// - Idle-timed-out (no activity for idle_timeout duration)
    pub async fn cleanup_stale_sessions(&self) -> Result<u64> {
        self.repo.delete_stale_sessions(self.config.idle_timeout).await
    }
}
