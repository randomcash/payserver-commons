//! What a filter allow rested on, as a plugin reports it to the host.
//!
//! A billing-style filter plugin attaches this to every allow so the host can
//! tell a fail-open allow (standing never received, or old) from an ordinary
//! one. The plugin serialises it and the host deserialises it, so both sides
//! name the same type: renaming a basis here is a compile error in whichever
//! side has not moved its pin, rather than a string that silently stops
//! matching and is filed as an unreadable verdict.

use serde::{Deserialize, Serialize};

/// What an allowed invoice creation rested on.
///
/// Deliberately a report of the basis, not a judgement: "stale" is a
/// comparison against a freshness bound, and the bound and a clock live on the
/// host side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "basis", rename_all = "snake_case")]
pub enum StandingBasis {
    /// No standing was ever received for this account, and the invoice was
    /// allowed anyway.
    NeverReceived,
    /// Allowed on a standing that says the account is in good standing, as
    /// the sender last confirmed at this instant. `last_heard_at` is an
    /// RFC 3339 timestamp; whether it is recent enough is the host's call.
    Confirmed { last_heard_at: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact bytes on the wire. If this goes red, a plugin built against
    /// the other revision is now speaking a dialect this host reads as
    /// unreadable.
    #[test]
    fn the_wire_form_is_pinned() {
        assert_eq!(
            serde_json::to_string(&StandingBasis::NeverReceived).unwrap(),
            r#"{"basis":"never_received"}"#
        );
        assert_eq!(
            serde_json::to_string(&StandingBasis::Confirmed {
                last_heard_at: "2026-10-06T09:00:00Z".into()
            })
            .unwrap(),
            r#"{"basis":"confirmed","last_heard_at":"2026-10-06T09:00:00Z"}"#
        );
    }

    #[test]
    fn both_forms_round_trip() {
        for basis in [
            StandingBasis::NeverReceived,
            StandingBasis::Confirmed {
                last_heard_at: "2026-10-06T09:00:00Z".into(),
            },
        ] {
            let json = serde_json::to_string(&basis).unwrap();
            assert_eq!(serde_json::from_str::<StandingBasis>(&json).unwrap(), basis);
        }
    }

    #[test]
    fn an_unknown_basis_does_not_parse() {
        assert!(serde_json::from_str::<StandingBasis>(r#"{"basis":"never_heard"}"#).is_err());
        assert!(serde_json::from_str::<StandingBasis>(r#"{"basis":"confirmed"}"#).is_err());
    }
}
