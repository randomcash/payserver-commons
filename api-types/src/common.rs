//! Shapes shared across several endpoints.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// A page of results.
///
/// The list endpoints each return their own named wrapper (`InvoiceListResponse`
/// and friends) because that is what they have always returned and renaming
/// them would break every integrator. This generic exists for the client, which
/// wants one shape to render a table from.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

/// Mask an extended public key for display.
///
/// An xpub is not a secret in the way a private key is - it derives receive
/// addresses and cannot spend - but it does link every address a merchant has
/// issued, so the full value is only ever handed over by the explicit export
/// endpoint. Everything else shows this.
///
/// Lives with the contract rather than in a server: two payservers masking
/// differently would be two different answers to the same question, and the
/// client renders whichever it is given.
#[must_use]
pub fn mask_xpub(xpub: &str) -> String {
    if xpub.len() <= 20 {
        return "****".to_string();
    }
    format!("{}...{}", &xpub[..8], &xpub[xpub.len() - 8..])
}

#[cfg(test)]
mod mask_tests {
    use super::mask_xpub;

    #[test]
    fn keeps_both_ends_and_hides_the_middle() {
        let xpub = "xpub6DCoCpSuQZB2jawqnGMEPS63ePKWkwWPH4TU45Q7LPXWuNd8TMtVxRrgjtEshuqpK3mdhaWHPFsBngh5GFZaM6si3yZdUsT8ddYM3PwnATt";
        let masked = mask_xpub(xpub);
        assert!(masked.starts_with("xpub6DCo"));
        assert!(masked.ends_with("PwnATt"));
        assert!(masked.contains("..."));
        assert!(!masked.contains(&xpub[20..40]));
    }

    /// Anything too short to mask meaningfully is replaced outright rather than
    /// half-shown.
    #[test]
    fn a_short_value_is_not_partially_revealed() {
        assert_eq!(mask_xpub("short"), "****");
        assert_eq!(mask_xpub("12345678901234567890"), "****");
        assert_eq!(mask_xpub("123456789012345678901"), "12345678...45678901");
    }
}
