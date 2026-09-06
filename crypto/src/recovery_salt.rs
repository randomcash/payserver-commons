//! The identifier the account recovery KDF is salted with.
//!
//! This convention had three independent implementations and no test proving
//! they agreed (RCS-200):
//!
//! 1. `auth::models::User::new_*` pins it at registration into
//!    `users.kdf_salt_identifier`.
//! 2. `ui-kit`'s registration flow rebuilds it to derive the verification hash.
//! 3. The recovery page needs a third copy to rebuild it from what a user types.
//!
//! Nothing enforced that those matched. If they ever disagree the stored
//! `recovery_verification_hash` cannot be reproduced and the account is
//! permanently unrecoverable — and it fails silently, as the same generic
//! `InvalidRecoveryMnemonic` a wrong phrase produces. This module is the single
//! definition, so a change to the rule cannot land in one copy alone.

/// How an account is identified for the purpose of salting its recovery key.
///
/// The variant is fixed at registration and pinned; it is deliberately not
/// recomputed from the account's current state. An account that gains an email
/// after registering with a wallet must keep salting with the wallet, or its
/// stored hash becomes unreproducible (RCS-201).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaltIdentity {
    /// Email account. Lowercased — `auth::models` lowercases before pinning, so
    /// anything rebuilding this must too or the salt differs by case alone.
    Email(String),
    /// Wallet-only account. The address must be EIP-55 checksummed, matching
    /// what `validate_and_checksum_address` pinned at registration.
    Wallet(String),
    /// Passkey-only account, identified by its user id.
    Passkey(String),
}

impl SaltIdentity {
    /// Render the pinned identifier string stored in `users.kdf_salt_identifier`.
    pub fn as_identifier(&self) -> String {
        match self {
            Self::Email(email) => email.to_lowercase(),
            Self::Wallet(address) => format!("wallet:{address}"),
            Self::Passkey(user_id) => format!("passkey:{user_id}"),
        }
    }

    /// The Argon2id salt input for recovery-key derivation.
    ///
    /// Derived from the identifier rather than random, so recovery can rebuild
    /// it from the phrase and the identifier alone — there is nothing else for
    /// the user to have kept.
    pub fn recovery_salt(&self) -> String {
        recovery_salt_for(&self.as_identifier())
    }
}

/// The Argon2id salt input for an already-rendered identifier.
///
/// Prefer [`SaltIdentity::recovery_salt`]. This exists for the server, which
/// holds the pinned identifier string and must not re-derive the variant from
/// mutable account state.
pub fn recovery_salt_for(identifier: &str) -> String {
    format!("payserver-recovery:{identifier}")
}

/// The Argon2id cost for recovery-key derivation.
///
/// A third duplication lived here too: `mnemonic::derive_recovery_key` hardcoded
/// these, `KdfParams::default()` declared them, and the registration flow
/// restated them again. The recovery hash depends on the cost, so a mismatch
/// between any two makes the stored hash unreproducible — the same silent,
/// permanent unrecoverability the salt duplication risked.
///
/// Raising these is a breaking change for every existing account: their stored
/// hash was built with the old cost. It cannot be done by editing these
/// constants alone — accounts carry their own `kdf_params`, and the recovery
/// flow must derive with the account's pinned values, not with these.
pub const RECOVERY_MEMORY_KB: u32 = 65536; // 64 MB
pub const RECOVERY_ITERATIONS: u32 = 3;
pub const RECOVERY_PARALLELISM: u32 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    /// The three shapes, pinned. Changing any of these makes every existing
    /// account of that kind unrecoverable, so this test is the tripwire.
    #[test]
    fn identifier_shapes_are_pinned() {
        assert_eq!(
            SaltIdentity::Email("user@example.com".into()).as_identifier(),
            "user@example.com"
        );
        assert_eq!(
            SaltIdentity::Wallet("0xAbC0000000000000000000000000000000000001".into())
                .as_identifier(),
            "wallet:0xAbC0000000000000000000000000000000000001"
        );
        assert_eq!(
            SaltIdentity::Passkey("2f1c...".into()).as_identifier(),
            "passkey:2f1c..."
        );
    }

    /// Email is lowercased at registration (`auth::models`), so a recovery form
    /// that passes through whatever the user typed must land on the same salt.
    /// Without this, "Foo@Bar.com" derives a different key from "foo@bar.com"
    /// and the account looks unrecoverable to its owner.
    #[test]
    fn email_case_does_not_change_the_salt() {
        let typed = SaltIdentity::Email("Foo@Bar.COM".into());
        let stored = SaltIdentity::Email("foo@bar.com".into());
        assert_eq!(typed.as_identifier(), stored.as_identifier());
        assert_eq!(typed.recovery_salt(), stored.recovery_salt());
    }

    /// Wallet addresses are NOT case-normalised: the checksum is meaningful and
    /// the server pins the checksummed form. Lowercasing here would silently
    /// break every wallet account, so this pins the opposite of the email rule.
    #[test]
    fn wallet_address_case_is_preserved() {
        let checksummed = "0xAbC0000000000000000000000000000000000001";
        assert_eq!(
            SaltIdentity::Wallet(checksummed.into()).as_identifier(),
            format!("wallet:{checksummed}")
        );
        assert_ne!(
            SaltIdentity::Wallet(checksummed.into()).as_identifier(),
            SaltIdentity::Wallet(checksummed.to_lowercase()).as_identifier()
        );
    }

    #[test]
    fn recovery_salt_carries_the_domain_prefix() {
        assert_eq!(
            SaltIdentity::Email("user@example.com".into()).recovery_salt(),
            "payserver-recovery:user@example.com"
        );
        assert_eq!(
            recovery_salt_for("wallet:0xAbC"),
            "payserver-recovery:wallet:0xAbC"
        );
    }

    /// The server holds the pinned string, the client holds the variant. Both
    /// must reach the same salt or recovery cannot work.
    #[test]
    fn client_variant_and_server_string_agree() {
        for id in [
            SaltIdentity::Email("user@example.com".into()),
            SaltIdentity::Wallet("0xAbC0000000000000000000000000000000000001".into()),
            SaltIdentity::Passkey("2f1c8f4e".into()),
        ] {
            assert_eq!(id.recovery_salt(), recovery_salt_for(&id.as_identifier()));
        }
    }

    /// The constants above and `KdfParams::default()` must not drift apart:
    /// registration stores the latter while derivation uses the former, so a
    /// mismatch produces a hash the server can never reproduce.
    #[test]
    fn default_kdf_params_match_the_derivation_constants() {
        let d = crate::types::KdfParams::default();
        assert_eq!(d.memory_kb, RECOVERY_MEMORY_KB);
        assert_eq!(d.iterations, RECOVERY_ITERATIONS);
        assert_eq!(d.parallelism, RECOVERY_PARALLELISM);
        assert_eq!(d.algorithm, "argon2id");
    }
}

/// End-to-end proof of the recovery contract (RCS-200 / RCS-205).
///
/// These use the real primitives rather than mocks, because the failure they
/// guard against is silent: if registration and recovery derive different keys,
/// the server returns the same generic "invalid recovery phrase" a wrong phrase
/// gives, and nobody finds out until a merchant genuinely needs to recover.
#[cfg(test)]
mod recovery_round_trip {
    use crate::mnemonic::RecoveryMnemonic;
    use crate::{kdf, symmetric};
    use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
    use sha2::{Digest, Sha256};

    fn verification_hash(key: &crate::SymmetricKey) -> String {
        B64.encode(Sha256::digest(key.as_bytes()))
    }

    /// Registration and recovery must derive the SAME verification hash from the
    /// same phrase and identifier. This is the whole basis of recovery working.
    #[test]
    fn recovery_reproduces_the_registration_hash() {
        let identifier = super::SaltIdentity::Passkey("2f1c8f4e".into()).as_identifier();
        let mnemonic = RecoveryMnemonic::generate().expect("generate");

        // At registration.
        let at_registration = mnemonic.derive_recovery_key(&identifier).expect("derive");
        let stored_hash = verification_hash(&at_registration);

        // At recovery: same phrase typed back, same identifier rebuilt.
        let typed_back = RecoveryMnemonic::from_phrase(mnemonic.phrase()).expect("parse");
        let at_recovery = typed_back.derive_recovery_key(&identifier).expect("derive");

        assert_eq!(
            verification_hash(&at_recovery),
            stored_hash,
            "recovery must reproduce the hash registration stored"
        );
    }

    /// A different identifier must NOT reproduce the hash - the phrase is bound
    /// to one account, so the same words on another account are useless.
    #[test]
    fn the_phrase_is_bound_to_its_account() {
        let mnemonic = RecoveryMnemonic::generate().expect("generate");
        let mine = super::SaltIdentity::Email("me@example.com".into()).as_identifier();
        let theirs = super::SaltIdentity::Email("you@example.com".into()).as_identifier();

        let mine_key = mnemonic.derive_recovery_key(&mine).expect("derive");
        let theirs_key = mnemonic.derive_recovery_key(&theirs).expect("derive");

        assert_ne!(
            verification_hash(&mine_key),
            verification_hash(&theirs_key),
            "the same phrase must not unlock a different account"
        );
    }

    /// The data-preserving half (RCS-200): recovery must carry the account's
    /// EXISTING symmetric key across to the new phrase.
    ///
    /// If a recovery implementation generates a fresh key instead - which is
    /// what it was forced to do before the wrapped key was returned - everything
    /// the merchant encrypted becomes unreadable, even though they held the
    /// right phrase and did everything correctly.
    #[test]
    fn rewrapping_preserves_the_account_key_across_a_new_phrase() {
        let identifier = super::SaltIdentity::Email("merchant@example.com".into()).as_identifier();

        // Registration: a data key wrapped under the original phrase.
        let old_phrase = RecoveryMnemonic::generate().expect("generate");
        let old_recovery = old_phrase.derive_recovery_key(&identifier).expect("derive");
        let old_stretched = kdf::stretch_master_key(&old_recovery).expect("stretch");
        let account_key = kdf::generate_symmetric_key();
        let stored = symmetric::encrypt_key(&account_key, &old_stretched).expect("wrap");

        // Recovery: unwrap with the old phrase, re-wrap under a new one.
        let unwrapped = symmetric::decrypt_key(&stored, &old_stretched).expect("unwrap");
        assert_eq!(
            unwrapped.as_bytes(),
            account_key.as_bytes(),
            "the old phrase must unwrap the stored key"
        );

        let new_phrase = RecoveryMnemonic::generate().expect("generate");
        let new_recovery = new_phrase.derive_recovery_key(&identifier).expect("derive");
        let new_stretched = kdf::stretch_master_key(&new_recovery).expect("stretch");
        let rewrapped = symmetric::encrypt_key(&unwrapped, &new_stretched).expect("re-wrap");

        // The merchant's data must still open, now under the new phrase.
        let after_recovery = symmetric::decrypt_key(&rewrapped, &new_stretched).expect("unwrap");
        assert_eq!(
            after_recovery.as_bytes(),
            account_key.as_bytes(),
            "the account key must survive recovery - a fresh key would orphan all encrypted data"
        );

        // And the old phrase must no longer open the new wrapping.
        assert!(
            symmetric::decrypt_key(&rewrapped, &old_stretched).is_err(),
            "the superseded phrase must not unwrap the re-wrapped key"
        );
    }
}
