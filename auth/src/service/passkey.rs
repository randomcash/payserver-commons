//! Passkey/WebAuthn authentication methods.

use chrono::Utc;
use uuid::Uuid;
use webauthn_rs::prelude::DiscoverableKey;

use crate::error::{AuthError, Result};
#[cfg(feature = "metrics")]
use crate::metrics;
use crate::models::{
    CompleteNewUserPasskeyRegistrationRequest, CompletePasskeyLoginRequest,
    CompletePasskeyRegistrationRequest, Device, LoginResponse, PasskeyCredential, PasskeyId,
    PasskeyInfo, Session, SessionId, StartNewUserPasskeyRegistrationResponse,
    StartPasskeyLoginResponse, StartPasskeyRegistrationRequest, StartPasskeyRegistrationResponse,
    UserId,
};
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
    // ========================================================================
    // New User Registration
    // ========================================================================

    /// Register a new user with passkey (RECOMMENDED registration flow).
    ///
    /// This is a two-step process:
    /// 1. Client calls start_new_user_passkey_registration to get challenge + user_id
    /// 2. Client calls complete_new_user_passkey_registration with credential + user_id
    ///
    /// No email required - the user_id is used as the unique identifier.
    /// The client must generate the symmetric key and encrypt it appropriately.
    /// Since passkeys don't directly provide a decryption key, the client should:
    /// - Use the BIP39 mnemonic as the primary key source
    /// - Derive: recovery_key = Argon2id(mnemonic, "passkey:{user_id}")
    /// - Use recovery_key to encrypt the symmetric key
    pub async fn start_new_user_passkey_registration(
        &self,
    ) -> Result<StartNewUserPasskeyRegistrationResponse> {
        // Generate a temporary user ID for the registration
        let user_id = UserId::new();

        // Use user_id as the WebAuthn user identifier
        let user_identifier = format!("passkey:{}", user_id);

        // Generate WebAuthn registration challenge with discoverable credential
        let (ccr, passkey_registration) = self
            .webauthn
            .start_passkey_registration(
                Uuid::from(user_id.0),
                &user_identifier,
                &user_identifier,
                None, // No excluded credentials for new user
            )
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state with user_identifier for consistency verification
        self.repo
            .store_registration_challenge(user_id, &user_identifier, passkey_registration)
            .await?;

        Ok(StartNewUserPasskeyRegistrationResponse {
            options: ccr,
            user_id,
        })
    }

    /// Complete new user registration with passkey.
    ///
    /// Creates the user account and passkey credential.
    /// The recovery_verification_hash is required for account recovery.
    ///
    /// # Atomicity
    /// This method creates user, passkey, device, and session in sequence.
    /// The repository implementation should use a transaction to ensure
    /// atomicity - if any step fails, all changes should be rolled back.
    pub async fn complete_new_user_passkey_registration(
        &self,
        request: CompleteNewUserPasskeyRegistrationRequest,
    ) -> Result<LoginResponse> {
        use crate::models::User;

        // Expected user identifier format for passkey-only users
        let user_identifier = format!("passkey:{}", request.user_id);

        // Retrieve the stored challenge state and verify identifier matches
        let (passkey_registration, stored_identifier) = self
            .repo
            .take_registration_challenge(request.user_id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Verify the identifier matches what was used in start_new_user_passkey_registration
        if stored_identifier != user_identifier {
            return Err(AuthError::PasskeyChallengeExpired);
        }

        // Complete WebAuthn registration
        let passkey = self
            .webauthn
            .finish_passkey_registration(&request.credential, &passkey_registration)
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Create passkey-only user with required recovery verification hash
        let user = User::new_passkey_only(
            request.user_id,
            request.kdf_params.clone(),
            request.encrypted_symmetric_key.clone(),
            request.recovery_verification_hash.clone(),
        );

        self.repo.create_user(&user).await?;

        // Store the passkey credential
        let passkey_cred = PasskeyCredential::new(user.id, request.passkey_name.clone(), passkey);
        self.repo.create_passkey(&passkey_cred).await?;

        // Create first device
        let device = Device::new(
            user.id,
            request.device_name.clone(),
            request.device_type,
            request.encrypted_symmetric_key.clone(),
            request.kdf_params.clone(),
        );
        let device_id = device.id;
        self.repo.create_device(&device).await?;

        // Create session
        let expires_at = Utc::now() + self.config.session_duration;
        let session = Session::with_expiration(user.id, device_id, expires_at);
        let session_id = session.id;
        self.repo.create_session(&session).await?;

        #[cfg(feature = "metrics")]
        metrics::record_user_registration();

        Ok(LoginResponse {
            session_id,
            device_id,
            encrypted_symmetric_key: request.encrypted_symmetric_key.clone(),
            kdf_params: request.kdf_params.clone(),
            email: None, // Passkey-only registration, no email
            primary_wallet_address: None, // No wallet either
            expires_at,
        })
    }

    // ========================================================================
    // Passkey Login (Discoverable Credentials)
    // ========================================================================

    /// Start passkey authentication using discoverable credentials.
    ///
    /// No email or user identifier is needed - the authenticator will present
    /// all available passkeys for this relying party and let the user choose.
    /// Returns WebAuthn challenge options and a challenge_id that must be sent
    /// back when completing authentication.
    pub async fn start_passkey_login(&self) -> Result<StartPasskeyLoginResponse> {
        // Generate a random challenge ID to track this authentication attempt
        let challenge_id = Uuid::new_v4();

        // Generate WebAuthn discoverable authentication challenge
        // This creates a challenge without specifying any allowCredentials,
        // allowing the authenticator to use any resident/discoverable credential
        let (rcr, passkey_authentication) = self
            .webauthn
            .start_discoverable_authentication()
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state with the random challenge_id
        self.repo
            .store_discoverable_authentication_challenge(challenge_id, passkey_authentication)
            .await?;

        Ok(StartPasskeyLoginResponse {
            options: rcr,
            challenge_id,
        })
    }

    /// Complete passkey authentication using discoverable credentials.
    ///
    /// The credential response from the authenticator contains the credential ID,
    /// which we use to look up the user's passkey and verify the authentication.
    pub async fn complete_passkey_login(
        &self,
        request: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse> {
        // Retrieve the stored challenge state using the challenge_id
        let passkey_authentication = self
            .repo
            .take_discoverable_authentication_challenge(request.challenge_id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Get the credential ID from the response to look up the passkey
        let credential_id = request.credential.id.as_ref();

        // Find the passkey by credential ID
        let passkey_cred = self
            .repo
            .get_passkey_by_credential_id(credential_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Get the user
        let user = self
            .repo
            .get_user(passkey_cred.user_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is locked
        if user.is_locked() {
            #[cfg(feature = "metrics")]
            metrics::record_login_failure();
            return Err(AuthError::AccountLocked);
        }

        // Get all user's passkeys for verification
        let passkeys = self.repo.get_passkeys_for_user(user.id).await?;
        let mut active_passkeys: Vec<_> = passkeys.into_iter().filter(|p| p.is_active).collect();
        // Convert Passkey to DiscoverableKey for discoverable authentication
        let discoverable_keys: Vec<DiscoverableKey> = active_passkeys
            .iter()
            .map(|p| DiscoverableKey::from(&p.passkey))
            .collect();

        // Complete WebAuthn discoverable authentication
        let auth_result = self
            .webauthn
            .finish_discoverable_authentication(&request.credential, passkey_authentication, &discoverable_keys)
            .map_err(|_| {
                // WebAuthn verification failed - could be a technical issue or attack
                // Don't increment failed logins (passkey failures can't be brute-forced)
                #[cfg(feature = "metrics")]
                metrics::record_login_failure();
                AuthError::PasskeyVerificationFailed
            })?;

        // Update the passkey counter and last_used_at
        if let Some(cred) = active_passkeys
            .iter_mut()
            .find(|p| p.passkey.cred_id() == auth_result.cred_id())
        {
            cred.passkey.update_credential(&auth_result);
            cred.last_used_at = Some(Utc::now());
            self.repo.update_passkey(cred).await?;
        }

        // Reset failed login attempts on successful passkey auth
        self.repo.reset_failed_logins(user.id).await?;

        // Find existing device by ID, or create new device
        let device_id = if let Some(provided_device_id) = request.device_id {
            // Client provided a device ID - verify it belongs to this user and is active
            let device = self
                .repo
                .get_device(provided_device_id)
                .await?
                .ok_or(AuthError::DeviceNotFound(provided_device_id.to_string()))?;

            if device.user_id != user.id {
                // Device belongs to a different user - treat as not found
                return Err(AuthError::DeviceNotFound(provided_device_id.to_string()));
            }

            if !device.is_active {
                // Device was revoked - treat as not found, client should create new
                return Err(AuthError::DeviceNotFound(provided_device_id.to_string()));
            }

            // Update last_used_at
            let mut updated = device.clone();
            updated.last_used_at = Some(Utc::now());
            self.repo.update_device(&updated).await?;
            provided_device_id
        } else {
            // No device ID provided - create new device
            let active_count = self.repo.count_active_devices(user.id).await?;
            if active_count >= self.config.max_devices_per_user {
                return Err(AuthError::MaxDevicesReached(self.config.max_devices_per_user));
            }

            let new_device = Device::new(
                user.id,
                request.device_name.clone(),
                request.device_type,
                user.encrypted_symmetric_key.clone(),
                user.kdf_params.clone(),
            );
            let id = new_device.id;
            self.repo.create_device(&new_device).await?;
            id
        };

        // Update last login
        let mut updated_user = user.clone();
        updated_user.last_login_at = Some(Utc::now());
        self.repo.update_user(&updated_user).await?;

        // Create session
        let expires_at = Utc::now() + self.config.session_duration;
        let session = Session::with_expiration(user.id, device_id, expires_at);
        let session_id = session.id;
        self.repo.create_session(&session).await?;

        #[cfg(feature = "metrics")]
        metrics::record_user_login();

        Ok(LoginResponse {
            session_id,
            device_id,
            encrypted_symmetric_key: user.encrypted_symmetric_key,
            kdf_params: user.kdf_params,
            email: user.email,
            primary_wallet_address: user.primary_wallet_address,
            expires_at,
        })
    }

    // ========================================================================
    // Passkey Management (Existing User)
    // ========================================================================

    /// Start passkey registration for an existing user.
    ///
    /// Requires a valid session. Returns WebAuthn challenge options for the client
    /// to pass to the authenticator (e.g., browser's `navigator.credentials.create()`).
    pub async fn start_passkey_registration(
        &self,
        session_id: SessionId,
        request: StartPasskeyRegistrationRequest,
    ) -> Result<StartPasskeyRegistrationResponse> {
        // Validate session
        let (user_info, _session) = self.validate_session(session_id).await?;

        // Check passkey limit
        let passkey_count = self.repo.count_active_passkeys(user_info.id).await?;
        if passkey_count >= self.config.max_passkeys_per_user {
            return Err(AuthError::MaxPasskeysReached(self.config.max_passkeys_per_user));
        }

        // Get existing passkeys to exclude from registration
        let existing_passkeys = self.repo.get_passkeys_for_user(user_info.id).await?;
        let excluded_credentials: Vec<_> = existing_passkeys
            .iter()
            .filter(|p| p.is_active)
            .map(|p| p.passkey.cred_id().clone())
            .collect();

        // Get user identifier (email or wallet address) for WebAuthn registration
        let user_identifier = user_info
            .email
            .clone()
            .or_else(|| user_info.primary_wallet_address.clone().map(|w| format!("wallet:{}", w)))
            .ok_or_else(|| {
                AuthError::Repository("User has neither email nor wallet address".into())
            })?;

        // Generate WebAuthn registration challenge
        let (ccr, passkey_registration) = self
            .webauthn
            .start_passkey_registration(
                Uuid::from(user_info.id.0),
                &user_identifier,
                &user_identifier, // Display name same as identifier
                Some(excluded_credentials),
            )
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state with identifier for consistency verification
        self.repo
            .store_registration_challenge(user_info.id, &user_identifier, passkey_registration)
            .await?;

        // Note: passkey_name from request is intentionally unused here.
        // Client provides it again in CompletePasskeyRegistrationRequest.
        let _ = request.passkey_name;

        Ok(StartPasskeyRegistrationResponse { options: ccr })
    }

    /// Complete passkey registration.
    ///
    /// Validates the credential from the authenticator and stores the passkey.
    pub async fn complete_passkey_registration(
        &self,
        session_id: SessionId,
        request: CompletePasskeyRegistrationRequest,
    ) -> Result<PasskeyInfo> {
        // Validate session
        let (user_info, _session) = self.validate_session(session_id).await?;

        // Re-check passkey limit (race condition protection)
        let passkey_count = self.repo.count_active_passkeys(user_info.id).await?;
        if passkey_count >= self.config.max_passkeys_per_user {
            return Err(AuthError::MaxPasskeysReached(self.config.max_passkeys_per_user));
        }

        // Retrieve the stored challenge state and verify identifier consistency
        let (passkey_registration, stored_identifier) = self
            .repo
            .take_registration_challenge(user_info.id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Get current user identifier for verification
        let user_identifier = user_info
            .email
            .clone()
            .or_else(|| user_info.primary_wallet_address.clone().map(|w| format!("wallet:{}", w)))
            .ok_or_else(|| {
                AuthError::Repository("User has neither email nor wallet address".into())
            })?;

        // Verify the identifier matches (should always match for existing user, but verify anyway)
        if stored_identifier != user_identifier {
            return Err(AuthError::PasskeyChallengeExpired);
        }

        // Complete WebAuthn registration
        let passkey = self
            .webauthn
            .finish_passkey_registration(&request.credential, &passkey_registration)
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the passkey credential
        let credential = PasskeyCredential::new(user_info.id, request.passkey_name, passkey);
        let passkey_info = PasskeyInfo::from(&credential);
        self.repo.create_passkey(&credential).await?;

        Ok(passkey_info)
    }

    /// Get all passkeys for the current user.
    ///
    /// Requires a valid session. Returns sanitized PasskeyInfo (without actual key material).
    pub async fn get_passkeys(&self, session_id: SessionId) -> Result<Vec<PasskeyInfo>> {
        let (user_info, _session) = self.validate_session(session_id).await?;

        let passkeys = self.repo.get_passkeys_for_user(user_info.id).await?;

        Ok(passkeys
            .iter()
            .filter(|p| p.is_active)
            .map(PasskeyInfo::from)
            .collect())
    }

    /// Revoke a passkey.
    ///
    /// Requires a valid session for authentication.
    pub async fn revoke_passkey(&self, session_id: SessionId, passkey_id: PasskeyId) -> Result<()> {
        let (user_info, _session) = self.validate_session(session_id).await?;

        // Verify the passkey belongs to this user
        let passkey = self
            .repo
            .get_passkey(passkey_id)
            .await?
            .ok_or(AuthError::PasskeyNotFound(passkey_id.to_string()))?;

        if passkey.user_id != user_info.id {
            return Err(AuthError::PasskeyNotFound(passkey_id.to_string()));
        }

        // Deactivate the passkey
        self.repo.deactivate_passkey(passkey_id).await
    }
}
