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
}
