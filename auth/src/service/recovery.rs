//! Account recovery methods.

use chrono::Utc;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use uuid::Uuid;

use crate::error::{AuthError, Result};
use crate::models::{
    CompleteRecoveryRequest, Device, LoginResponse, PasskeyCredential, Session,
    StartPasskeyRegistrationResponse, StartRecoveryRequest, User, UserId,
};
use crate::repository::{
    ChallengeRepository, DeviceRepository, PasskeyRepository, SessionRepository, UserRepository,
    WalletRepository,
};

use super::WebAuthnAuthService;
use super::validation::validate_email;

impl<R> WebAuthnAuthService<R>
where
    R: UserRepository
        + DeviceRepository
        + SessionRepository
        + PasskeyRepository
        + WalletRepository
        + ChallengeRepository,
{
    /// Resolve the user a recovery attempt refers to.
    ///
    /// Email, wallet address, or **account id**. The last exists for
    /// passkey-only accounts (RCS-201): they have no email and no wallet, so
    /// without it their recovery phrase is bound to `passkey:{user_id}` and no
    /// endpoint will ever accept it.
    ///
    /// NOTE: the registration screen does not yet surface the account id, so a
    /// passkey-only merchant has no way to learn the value this branch expects.
    /// The path exists; it is not reachable by a user until that lands.
    ///
    /// Returns `None` rather than a distinguishing error so callers keep
    /// answering `InvalidRecoveryMnemonic` and do not leak which accounts exist.
    async fn resolve_recovery_user(&self, identifier: &str) -> Result<Option<User>> {
        let lower = identifier.to_lowercase();

        if lower.contains('@') {
            validate_email(&lower)?;
            return self.repo.get_user_by_email(&lower).await;
        }
        if lower.starts_with("0x") && lower.len() == 42 {
            let checksummed = self.validate_and_checksum_address(identifier)?;
            return self.repo.get_user_by_wallet_address(&checksummed).await;
        }
        if let Ok(id) = Uuid::parse_str(&lower) {
            return self.repo.get_user(UserId(id)).await;
        }

        // Not recognisable as anything: fall through to email validation so the
        // caller gets the same error shape as a malformed address would give.
        validate_email(&lower)?;
        self.repo.get_user_by_email(&lower).await
    }

    /// Start account recovery using BIP39 mnemonic.
    ///
    /// This is step 1 of the recovery process. After verifying the mnemonic,
    /// returns a passkey registration challenge so the user can register a new passkey.
    ///
    /// The client must:
    /// 1. Derive recovery key from mnemonic + the account's pinned
    ///    kdf_salt_identifier (email, `wallet:{addr}` or `passkey:{id}`)
    ///    using Argon2id
    /// 2. Hash recovery key: base64(SHA-256(recovery_key)) for verification
    /// 3. Call this method with the hash
    /// 4. Use the returned challenge to register a new passkey
    /// 5. Call complete_account_recovery with the passkey credential
    ///
    /// # Rate Limiting
    /// Failed recovery attempts are tracked and the account is locked after
    /// too many failures.
    pub async fn start_account_recovery(
        &self,
        request: StartRecoveryRequest,
    ) -> Result<StartPasskeyRegistrationResponse> {
        let user = self.resolve_recovery_user(&request.identifier).await?;

        // Return generic error to prevent user enumeration
        let user = match user {
            Some(u) => u,
            None => return Err(AuthError::InvalidRecoveryMnemonic),
        };

        // CRITICAL: Verify the client knows the recovery mnemonic by comparing hashes.
        // The client derives: base64(SHA-256(Argon2id(mnemonic, email)))
        // We compare this with the stored hash using constant-time comparison.
        let verification_matches = {
            let stored = Sha256::digest(user.recovery_verification_hash.as_bytes());
            let provided = Sha256::digest(request.recovery_verification_hash.as_bytes());
            stored.ct_eq(&provided).unwrap_u8() == 1
        };

        if !verification_matches {
            // Track failed recovery attempts
            let attempts = self.repo.increment_failed_logins(user.id).await?;

            // Lock account if too many failures
            if attempts >= self.config.max_failed_attempts {
                let lock_until = Utc::now() + self.config.lockout_duration;
                self.repo.lock_user(user.id, lock_until).await?;
            }

            // Always return InvalidRecoveryMnemonic to prevent user enumeration
            return Err(AuthError::InvalidRecoveryMnemonic);
        }

        // Recovery hash is correct - NOW check if account is locked
        if user.is_locked() {
            return Err(AuthError::AccountLocked);
        }

        // Verification passed - generate passkey registration challenge
        // We'll use the user's existing ID for the WebAuthn user handle

        // Use the identifier (email or wallet address) for WebAuthn registration
        // The pinned value, not the recomputed one: recomputing prefers email
        // over wallet, so an account that gained an email after registration
        // would derive a different salt than the stored hash was built from and
        // could never be recovered (RCS-201).
        let user_identifier = user.kdf_salt_identifier.clone();

        // Generate WebAuthn registration challenge
        let (ccr, passkey_registration) = self
            .webauthn
            .start_passkey_registration(
                user.id.0,
                &user_identifier,
                &user_identifier,
                None, // Don't exclude existing credentials - they'll be deleted on completion
            )
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state with identifier for consistency verification
        self.repo
            .store_registration_challenge(user.id, &user_identifier, passkey_registration)
            .await?;

        Ok(StartPasskeyRegistrationResponse { options: ccr })
    }

    /// Complete account recovery.
    ///
    /// This is step 2 of the recovery process. After the user registers a new passkey,
    /// this method completes the recovery by:
    /// - Verifying the new passkey credential
    /// - Revoking all old passkeys
    /// - Revoking all existing sessions
    /// - Updating the user's encrypted symmetric key
    /// - Creating a new session
    ///
    /// # Atomicity
    /// This method performs multiple updates. The repository implementation should
    /// use a transaction to ensure atomicity.
    pub async fn complete_account_recovery(
        &self,
        identifier: &str,
        request: CompleteRecoveryRequest,
    ) -> Result<LoginResponse> {
        let user = self.resolve_recovery_user(identifier).await?;

        // Find user - return generic error to prevent user enumeration
        let user = user.ok_or(AuthError::InvalidRecoveryMnemonic)?;

        // Get the user identifier for verification
        // The pinned value, not the recomputed one: recomputing prefers email
        // over wallet, so an account that gained an email after registration
        // would derive a different salt than the stored hash was built from and
        // could never be recovered (RCS-201).
        let user_identifier = user.kdf_salt_identifier.clone();

        // Retrieve the stored challenge state and verify identifier consistency
        let (passkey_registration, stored_identifier) = self
            .repo
            .take_registration_challenge(user.id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Verify the identifier matches what was used in start_account_recovery
        if stored_identifier != user_identifier {
            return Err(AuthError::PasskeyChallengeExpired);
        }

        // Complete WebAuthn registration
        let passkey = self
            .webauthn
            .finish_passkey_registration(&request.credential, &passkey_registration)
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Recovery successful - reset failed attempts
        self.repo.reset_failed_logins(user.id).await?;

        // Revoke all existing passkeys
        self.repo.delete_all_passkeys_for_user(user.id).await?;

        // Revoke all existing devices
        self.repo.delete_all_devices_for_user(user.id).await?;

        // Revoke all existing sessions
        self.repo.delete_all_sessions_for_user(user.id).await?;

        // Update user with new encrypted key and recovery hash
        let mut updated_user = user.clone();
        updated_user.kdf_params = request.new_kdf_params.clone();
        updated_user.encrypted_symmetric_key = request.new_encrypted_symmetric_key.clone();
        updated_user.recovery_verification_hash = request.new_recovery_verification_hash.clone();
        updated_user.failed_login_attempts = 0;
        updated_user.locked_until = None;
        self.repo.update_user(&updated_user).await?;

        // Store the new passkey credential
        let passkey_cred = PasskeyCredential::new(user.id, request.passkey_name.clone(), passkey);
        self.repo.create_passkey(&passkey_cred).await?;

        // Create new device for recovery session
        let device = Device::new(
            user.id,
            request.device_name.clone(),
            request.device_type,
            request.new_encrypted_symmetric_key.clone(),
            request.new_kdf_params.clone(),
        );
        let device_id = device.id;
        self.repo.create_device(&device).await?;

        // Create new session
        let expires_at = Utc::now() + self.config.session_duration;
        let session = Session::with_expiration(user.id, device_id, expires_at);
        let session_id = session.id;
        self.repo.create_session(&session).await?;

        Ok(LoginResponse {
            session_id,
            device_id,
            encrypted_symmetric_key: request.new_encrypted_symmetric_key.clone(),
            kdf_params: request.new_kdf_params.clone(),
            email: user.email.clone(),
            primary_wallet_address: user.primary_wallet_address.clone(),
            expires_at,
        })
    }
}
