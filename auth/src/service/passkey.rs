//! Passkey/WebAuthn authentication methods.

use chrono::Utc;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use crate::error::{AuthError, Result};
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

use super::validation::validate_email;
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
    /// The client must generate the symmetric key and encrypt it appropriately.
    /// Since passkeys don't directly provide a decryption key, the client should:
    /// - Use the BIP39 mnemonic as the primary key source
    /// - Derive: recovery_key = Argon2id(mnemonic, email)
    /// - Use recovery_key to encrypt the symmetric key
    pub async fn start_new_user_passkey_registration(
        &self,
        email: &str,
    ) -> Result<StartNewUserPasskeyRegistrationResponse> {
        // Validate email
        validate_email(email)?;
        let email_lower = email.to_lowercase();

        // Check if user already exists
        if self.repo.get_user_by_email(&email_lower).await?.is_some() {
            return Err(AuthError::UserExists(email_lower));
        }

        // Generate a temporary user ID for the registration
        let user_id = UserId::new();

        // Generate WebAuthn registration challenge
        let (ccr, passkey_registration) = self
            .webauthn
            .start_passkey_registration(
                Uuid::from(user_id.0),
                &email_lower,
                &email_lower,
                None, // No excluded credentials for new user
            )
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state with email for consistency verification
        self.repo
            .store_registration_challenge(user_id, &email_lower, passkey_registration)
            .await?;

        Ok(StartNewUserPasskeyRegistrationResponse {
            options: ccr,
            user_id,
            email: email_lower,
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

        // Validate email
        validate_email(&request.email)?;
        let email_lower = request.email.to_lowercase();

        // Check if user already exists (race condition check)
        if self.repo.get_user_by_email(&email_lower).await?.is_some() {
            return Err(AuthError::UserExists(email_lower));
        }

        // Retrieve the stored challenge state and verify email matches
        let (passkey_registration, stored_email) = self
            .repo
            .take_registration_challenge(request.user_id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Verify the email matches what was used in start_new_user_passkey_registration
        // This prevents an attacker from starting with email_A and completing with email_B
        if stored_email != email_lower {
            return Err(AuthError::PasskeyChallengeExpired);
        }

        // Complete WebAuthn registration
        let passkey = self
            .webauthn
            .finish_passkey_registration(&request.credential, &passkey_registration)
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Create user with required recovery verification hash
        let mut user = User::new(
            email_lower.clone(),
            request.kdf_params.clone(),
            request.encrypted_symmetric_key.clone(),
            request.recovery_verification_hash.clone(),
        );
        // Use the user_id from the start request so it matches the passkey's user handle
        user.id = request.user_id;

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

        Ok(LoginResponse {
            session_id,
            device_id,
            encrypted_symmetric_key: request.encrypted_symmetric_key.clone(),
            kdf_params: request.kdf_params.clone(),
            email: Some(email_lower),
            primary_wallet_address: None, // Email-based registration, no wallet
            expires_at,
        })
    }

    // ========================================================================
    // Passkey Login
    // ========================================================================

    /// Start passkey authentication.
    ///
    /// Returns WebAuthn challenge options for the client to pass to the authenticator.
    /// The user must provide their email to look up their registered passkeys.
    pub async fn start_passkey_login(&self, email: &str) -> Result<StartPasskeyLoginResponse> {
        validate_email(email)?;
        let email_lower = email.to_lowercase();

        // Find user - return generic error to prevent enumeration
        let user = self
            .repo
            .get_user_by_email(&email_lower)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is locked - fail early before WebAuthn flow
        if user.is_locked() {
            return Err(AuthError::AccountLocked);
        }

        // Get user's passkeys
        let passkeys = self.repo.get_passkeys_for_user(user.id).await?;
        let active_passkeys: Vec<Passkey> = passkeys
            .iter()
            .filter(|p| p.is_active)
            .map(|p| p.passkey.clone())
            .collect();

        if active_passkeys.is_empty() {
            // User has no passkeys - return same error as invalid credentials
            return Err(AuthError::InvalidCredentials);
        }

        // Generate WebAuthn authentication challenge
        let (rcr, passkey_authentication) = self
            .webauthn
            .start_passkey_authentication(&active_passkeys)
            .map_err(|e| AuthError::WebAuthn(e.to_string()))?;

        // Store the challenge state
        self.repo
            .store_authentication_challenge(user.id, passkey_authentication)
            .await?;

        Ok(StartPasskeyLoginResponse { options: rcr })
    }

    /// Complete passkey authentication.
    ///
    /// Validates the credential from the authenticator and creates a session.
    pub async fn complete_passkey_login(
        &self,
        request: CompletePasskeyLoginRequest,
    ) -> Result<LoginResponse> {
        validate_email(&request.email)?;
        let email_lower = request.email.to_lowercase();

        // Find user
        let user = self
            .repo
            .get_user_by_email(&email_lower)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is locked
        if user.is_locked() {
            return Err(AuthError::AccountLocked);
        }

        // Retrieve the stored challenge state
        let passkey_authentication = self
            .repo
            .take_authentication_challenge(user.id)
            .await?
            .ok_or(AuthError::PasskeyChallengeExpired)?;

        // Get user's passkeys for verification
        let passkeys = self.repo.get_passkeys_for_user(user.id).await?;
        let mut active_passkeys: Vec<_> = passkeys.into_iter().filter(|p| p.is_active).collect();

        // Complete WebAuthn authentication
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&request.credential, &passkey_authentication)
            .map_err(|_| {
                // WebAuthn verification failed - could be a technical issue or attack
                // Don't increment failed logins (passkey failures can't be brute-forced)
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
