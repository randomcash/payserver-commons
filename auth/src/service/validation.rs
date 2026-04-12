//! Input validation utilities.

use crate::error::{AuthError, Result};

/// Validates email format.
/// Checks for basic structure: non-empty local part, @, non-empty domain with at least one dot.
pub fn validate_email(email: &str) -> Result<()> {
    let email = email.trim();

    if email.is_empty() {
        return Err(AuthError::InvalidEmail("email cannot be empty".into()));
    }

    // Split at @ and validate parts
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(AuthError::InvalidEmail(
            "email must contain exactly one @".into(),
        ));
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() {
        return Err(AuthError::InvalidEmail("local part cannot be empty".into()));
    }

    if domain.is_empty() {
        return Err(AuthError::InvalidEmail("domain cannot be empty".into()));
    }

    // Domain must have at least one dot (e.g., example.com)
    if !domain.contains('.') {
        return Err(AuthError::InvalidEmail("domain must contain a dot".into()));
    }

    // Domain cannot start or end with a dot
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(AuthError::InvalidEmail(
            "domain cannot start or end with a dot".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user@sub.domain.com").is_ok());
        assert!(validate_email("a@b.co").is_ok());
    }

    #[test]
    fn test_invalid_emails() {
        assert!(validate_email("").is_err());
        assert!(validate_email("noatsign").is_err());
        assert!(validate_email("@nodomain.com").is_err());
        assert!(validate_email("nolocal@").is_err());
        assert!(validate_email("nodot@localhost").is_err());
        assert!(validate_email("test@.example.com").is_err());
        assert!(validate_email("test@example.").is_err());
    }
}
