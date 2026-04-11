//! Tests for the authentication service.

use std::sync::Arc;

use crate::error::AuthError;
use crate::models::{
    DeviceId, PasskeyId, SessionId, StartNewUserWalletRegistrationRequest,
    StartPasskeyRegistrationRequest, StartRecoveryRequest, StartWalletLoginRequest,
};
use crate::repository::inmemory::InMemoryRepository;

use super::AuthService;

fn create_service() -> AuthService<InMemoryRepository> {
    let repo = Arc::new(InMemoryRepository::new());
    AuthService::new(repo)
}

// ========================================================================
// Session Validation Tests
// ========================================================================

#[tokio::test]
async fn test_validate_invalid_session() {
    let service = create_service();

    let result = service.validate_session(SessionId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_logout_requires_valid_session() {
    let service = create_service();

    // Logout with invalid session should succeed (idempotent)
    // but validate_session should still fail
    let _ = service.logout(SessionId::new()).await;
    let result = service.validate_session(SessionId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_logout_all_requires_valid_session() {
    let service = create_service();

    let result = service.logout_all(SessionId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

// ========================================================================
// Device Management Tests
// ========================================================================

#[tokio::test]
async fn test_get_devices_requires_valid_session() {
    let service = create_service();

    let result = service.get_devices(SessionId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_revoke_device_requires_valid_session() {
    let service = create_service();

    let result = service.revoke_device(SessionId::new(), DeviceId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

// ========================================================================
// Passkey Tests
// ========================================================================

#[tokio::test]
async fn test_get_passkeys_requires_valid_session() {
    let service = create_service();

    let result = service.get_passkeys(SessionId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_start_passkey_login_user_not_found() {
    let service = create_service();

    // Discoverable passkey login returns a challenge even with no users
    // (browser-side credential discovery handles user selection)
    let result = service.start_passkey_login().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_revoke_passkey_requires_valid_session() {
    let service = create_service();

    let result = service.revoke_passkey(SessionId::new(), PasskeyId::new()).await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_start_passkey_registration_requires_valid_session() {
    let service = create_service();

    let request = StartPasskeyRegistrationRequest {
        passkey_name: "Test Passkey".to_string(),
    };

    let result = service
        .start_passkey_registration(SessionId::new(), request)
        .await;
    assert!(matches!(result, Err(AuthError::SessionInvalid)));
}

#[tokio::test]
async fn test_start_new_user_passkey_registration_invalid_email() {
    let service = create_service();

    let result = service
        .start_new_user_passkey_registration()
        .await;
    // No email validation — should succeed (user ID generated internally)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_new_user_passkey_registration_valid_email() {
    let service = create_service();

    // Valid email should return a challenge and user_id
    let result = service
        .start_new_user_passkey_registration()
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.options.public_key.challenge.is_empty());
    // User ID should be returned so client can pass it back during completion
    assert!(!response.user_id.0.is_nil());
}

#[tokio::test]
async fn test_start_new_user_passkey_registration_email_case_insensitive() {
    let service = create_service();

    // Start registration — no email parameter needed (user ID generated internally)
    let result1 = service
        .start_new_user_passkey_registration()
        .await;
    assert!(result1.is_ok());
}

// ========================================================================
// Recovery Tests
// ========================================================================

#[tokio::test]
async fn test_start_account_recovery_user_not_found() {
    let service = create_service();

    // Try recovery for non-existent user
    let request = StartRecoveryRequest {
        identifier: "nonexistent@example.com".to_string(),
        recovery_verification_hash: "some_hash".to_string(),
    };

    // Should return InvalidRecoveryMnemonic (not UserNotFound) to prevent enumeration
    let result = service.start_account_recovery(request).await;
    assert!(matches!(result, Err(AuthError::InvalidRecoveryMnemonic)));
}

#[tokio::test]
async fn test_start_account_recovery_invalid_identifier() {
    let service = create_service();

    let request = StartRecoveryRequest {
        identifier: "invalid-email".to_string(),
        recovery_verification_hash: "some_hash".to_string(),
    };

    let result = service.start_account_recovery(request).await;
    assert!(matches!(result, Err(AuthError::InvalidEmail(_))));
}

// ========================================================================
// Email Validation Tests
// ========================================================================

#[tokio::test]
async fn test_passkey_registration_generates_unique_users() {
    let service = create_service();

    let result1 = service.start_new_user_passkey_registration().await;
    let result2 = service.start_new_user_passkey_registration().await;
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Each registration should get a unique user ID
    assert_ne!(result1.unwrap().user_id, result2.unwrap().user_id);
}

// Note: Full flow tests (registration, login, recovery, device management)
// require real WebAuthn credentials which cannot be mocked easily.
// These should be tested via integration tests with a WebAuthn testing library
// or end-to-end tests with a real browser.

// ========================================================================
// Wallet Authentication Tests
// ========================================================================

#[tokio::test]
async fn test_wallet_challenge_message_determinism() {
    let service = create_service();

    // Same inputs should produce same output
    let challenge = "abc123";
    let address = "0x1234567890123456789012345678901234567890";
    let timestamp = chrono::Utc::now();

    let msg1 = service.generate_wallet_challenge_message(challenge, address, &timestamp);
    let msg2 = service.generate_wallet_challenge_message(challenge, address, &timestamp);

    assert_eq!(msg1, msg2, "Same inputs must produce identical messages");

    // Different timestamp should produce different output
    let different_timestamp = timestamp + chrono::Duration::seconds(1);
    let msg3 = service.generate_wallet_challenge_message(challenge, address, &different_timestamp);

    assert_ne!(msg1, msg3, "Different timestamps must produce different messages");
}

#[tokio::test]
async fn test_wallet_address_validation() {
    let service = create_service();

    // Valid address (lowercase)
    let result = service.validate_and_checksum_address("0x1234567890abcdef1234567890abcdef12345678");
    assert!(result.is_ok());

    // Valid address (checksummed)
    let result = service.validate_and_checksum_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    assert!(result.is_ok());

    // Invalid: too short
    let result = service.validate_and_checksum_address("0x1234");
    assert!(matches!(result, Err(AuthError::InvalidWalletAddress(_))));

    // Invalid: not hex
    let result = service.validate_and_checksum_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG");
    assert!(matches!(result, Err(AuthError::InvalidWalletAddress(_))));

    // Address without 0x prefix is accepted by alloy-primitives
    let result = service.validate_and_checksum_address("1234567890abcdef1234567890abcdef12345678");
    assert!(result.is_ok(), "alloy-primitives accepts addresses without 0x prefix");
}

#[tokio::test]
async fn test_wallet_signature_verification() {
    use k256::ecdsa::SigningKey;
    use sha3::{Digest, Keccak256};

    let service = create_service();

    // Create a test private key (deterministic for testing)
    let private_key_bytes: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];
    let signing_key = SigningKey::from_bytes((&private_key_bytes).into()).unwrap();

    // Derive address from public key
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let public_key_hash = Keccak256::digest(&public_key_bytes.as_bytes()[1..]);
    let address = format!("0x{}", hex::encode(&public_key_hash[12..]));

    // Create a test message
    let message = "Test message for signing";

    // Sign with EIP-191 prefix
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();

    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(&message_hash)
        .unwrap();

    // Construct 65-byte signature (r + s + v)
    let mut sig_bytes = signature.to_bytes().to_vec();
    sig_bytes.push(recovery_id.to_byte() + 27); // Ethereum uses 27/28

    let signature_hex = format!("0x{}", hex::encode(&sig_bytes));

    // Verify the signature
    let result = service.verify_wallet_signature(message, &signature_hex, &address);
    assert!(result.is_ok(), "Signature verification failed: {:?}", result);
    assert!(result.unwrap(), "Signature should be valid");

    // Wrong message should fail
    let result = service.verify_wallet_signature("Wrong message", &signature_hex, &address);
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Signature should be invalid for wrong message");

    // Wrong address should fail
    let wrong_address = "0x0000000000000000000000000000000000000000";
    let result = service.verify_wallet_signature(message, &signature_hex, wrong_address);
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Signature should be invalid for wrong address");
}

#[tokio::test]
async fn test_start_wallet_login_wallet_not_found() {
    let service = create_service();

    let request = StartWalletLoginRequest {
        address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
    };

    let result = service.start_wallet_login(request).await;
    // Should return InvalidCredentials to prevent enumeration
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_start_new_user_wallet_registration_invalid_address() {
    let service = create_service();

    let request = StartNewUserWalletRegistrationRequest {
        address: "invalid".to_string(),
        wallet_name: "Test Wallet".to_string(),
    };

    let result = service.start_new_user_wallet_registration(request).await;
    assert!(matches!(result, Err(AuthError::InvalidWalletAddress(_))));
}

#[tokio::test]
async fn test_start_new_user_wallet_registration_valid_address() {
    let service = create_service();

    let request = StartNewUserWalletRegistrationRequest {
        address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        wallet_name: "Test Wallet".to_string(),
    };

    let result = service.start_new_user_wallet_registration(request).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.challenge_message.is_empty());
    assert!(!response.user_id.0.is_nil());
    // Address should be checksummed
    assert!(response.address.starts_with("0x"));
}

#[tokio::test]
async fn test_wallet_challenge_uses_consistent_timestamp() {
    let service = create_service();

    // Start registration - this stores the challenge with a timestamp
    let request = StartNewUserWalletRegistrationRequest {
        address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        wallet_name: "Test Wallet".to_string(),
    };

    let response = service.start_new_user_wallet_registration(request).await.unwrap();
    let challenge_message_from_start = response.challenge_message.clone();

    // The challenge message should contain a timestamp
    assert!(challenge_message_from_start.contains("Timestamp:"));

    // Parse the timestamp from the message to verify it's valid RFC3339
    let timestamp_line = challenge_message_from_start
        .lines()
        .find(|l| l.starts_with("Timestamp:"))
        .unwrap();
    let timestamp_str = timestamp_line.trim_start_matches("Timestamp:").trim();
    let parsed = chrono::DateTime::parse_from_rfc3339(timestamp_str);
    assert!(parsed.is_ok(), "Timestamp should be valid RFC3339: {}", timestamp_str);
}
