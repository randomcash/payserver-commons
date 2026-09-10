//! Device management methods.

use crate::error::{AuthError, Result};
use crate::models::{Device, DeviceId, DeviceInfo, SessionId, UserId};
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
    /// Get all active devices for a user.
    ///
    /// Requires a valid session for authentication.
    /// Returns sanitized DeviceInfo (without encrypted keys) for active devices only.
    pub async fn get_devices(&self, session_id: SessionId) -> Result<Vec<DeviceInfo>> {
        let (user_info, _session) = self.validate_session(session_id).await?;

        let devices = self.repo.get_devices_for_user(user_info.id).await?;

        // Filter to active devices only and convert to DeviceInfo
        Ok(devices
            .iter()
            .filter(|d| d.is_active)
            .map(DeviceInfo::from)
            .collect())
    }

    /// Revoke a device (removes its encrypted key, invalidates sessions).
    ///
    /// Requires a valid session for authentication.
    /// Cannot revoke the device associated with the current session - use logout instead.
    pub async fn revoke_device(&self, session_id: SessionId, device_id: DeviceId) -> Result<()> {
        let (user_info, session) = self.validate_session(session_id).await?;

        // Cannot revoke the device you're currently using
        if session.device_id == device_id {
            return Err(AuthError::CannotRevokeCurrentDevice);
        }

        // Verify the device belongs to this user
        let device = self
            .repo
            .get_device(device_id)
            .await?
            .ok_or(AuthError::DeviceNotFound(device_id.to_string()))?;

        if device.user_id != user_info.id {
            return Err(AuthError::DeviceNotFound(device_id.to_string()));
        }

        // Delete all sessions for this device
        self.repo.delete_sessions_for_device(device_id).await?;

        // Deactivate the device
        self.repo.deactivate_device(device_id).await
    }

    /// Resolve the device id a client sent on login into a device we can reuse.
    ///
    /// Returns `None` when the id is unusable — unknown, owned by someone else,
    /// or revoked — so the caller mints a fresh device instead. Deliberately not
    /// an error: by the time login calls this the credential has already been
    /// verified, so the id is only a hint about which device record to attach the
    /// session to. A hint that no longer applies is the same as no hint.
    ///
    /// It used to return `DeviceNotFound` for all three cases, which locked users
    /// out (RCS-248): the client keeps one id per browser in localStorage and
    /// never clears it on rejection, so it resent the same rejected id forever.
    /// Anyone with two accounts in one browser could only recover by clearing
    /// storage by hand.
    pub(crate) async fn reusable_device(
        &self,
        user_id: UserId,
        provided: Option<DeviceId>,
    ) -> Result<Option<Device>> {
        let Some(id) = provided else {
            return Ok(None);
        };

        match self.repo.get_device(id).await? {
            Some(d) if d.user_id == user_id && d.is_active => Ok(Some(d)),
            // Wrong owner and revoked are not distinguished in the log on
            // purpose: both mean "stale hint", and naming the owner of a device
            // id would leak across accounts.
            Some(_) => {
                tracing::info!(device_id = %id, "stale device id on login; creating a new device");
                Ok(None)
            }
            None => {
                tracing::info!(device_id = %id, "unknown device id on login; creating a new device");
                Ok(None)
            }
        }
    }
}
