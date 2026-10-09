//! What a filter allow rested on, as a plugin reports it to the host.
//!
//! A billing-style filter plugin attaches this to every allow so the host can
//! tell a fail-open allow (standing never received, or old) from an ordinary
//! one. The plugin serialises it and the host deserialises it, so both sides
//! name the same type: renaming a basis here is a compile error in whichever
//! side has not moved its pin, rather than a string that silently stops
//! matching and is filed as an unreadable verdict.

use chrono::{DateTime, Utc};
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
    /// the sender last confirmed at this instant. Whether that is recent
    /// enough is the host's call.
    ///
    /// Typed rather than a string so a sender cannot emit an instant the
    /// receiver's RFC 3339 parse will reject. That failure mode is the reason
    /// this type exists: a malformed timestamp does not surface as a loud
    /// error, it is filed as an unreadable verdict - one of the legitimate
    /// readings of this enum - so the drift reappears disguised as a valid
    /// answer. A `String` closes the shape and leaves the value open.
    Confirmed { last_heard_at: DateTime<Utc> },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        rfc3339.parse().expect("test timestamp is RFC 3339")
    }

    /// The exact bytes on the wire. If this goes red, a plugin built against
    /// the other revision is now speaking a dialect this host reads as
    /// unreadable.
    ///
    /// Typing `last_heard_at` deliberately did not move these bytes: the
    /// serialised form is the same string a sender already emitted, so a pin
    /// bump is not a wire break.
    #[test]
    fn the_wire_form_is_pinned() {
        assert_eq!(
            serde_json::to_string(&StandingBasis::NeverReceived).unwrap(),
            r#"{"basis":"never_received"}"#
        );
        assert_eq!(
            serde_json::to_string(&StandingBasis::Confirmed {
                last_heard_at: at("2026-10-06T09:00:00Z"),
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
                last_heard_at: at("2026-10-06T09:00:00Z"),
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

    /// The point of the typed field. Each of these deserialised happily while
    /// `last_heard_at` was a `String`, and each then failed the receiver's
    /// RFC 3339 parse - which is filed as an unreadable verdict, so the sender
    /// looked like it was speaking a dialect rather than sending a bad value.
    ///
    /// The bare date is not hypothetical: a recording fake accepted one for a
    /// `paid_through` that the real route rejects with a non-retryable 400,
    /// and that cost a release.
    #[test]
    fn a_confirmed_basis_whose_instant_is_not_rfc3339_does_not_parse() {
        for bad in [
            r#"{"basis":"confirmed","last_heard_at":"yesterday"}"#,
            r#"{"basis":"confirmed","last_heard_at":"2026-10-06"}"#,
            r#"{"basis":"confirmed","last_heard_at":"2026-10-06 09:00:00"}"#,
            r#"{"basis":"confirmed","last_heard_at":""}"#,
            r#"{"basis":"confirmed","last_heard_at":1760000000}"#,
        ] {
            assert!(
                serde_json::from_str::<StandingBasis>(bad).is_err(),
                "a sender must not be able to put this on the wire: {bad}"
            );
        }
    }

    /// An offset that is not `Z` is still RFC 3339 and still names one
    /// instant, so it is accepted and normalised rather than refused. Pinned
    /// because a receiver comparing strings instead of instants would read
    /// these two as different times.
    #[test]
    fn a_non_utc_offset_is_accepted_and_normalised() {
        let parsed: StandingBasis = serde_json::from_str(
            r#"{"basis":"confirmed","last_heard_at":"2026-10-06T11:00:00+02:00"}"#,
        )
        .expect("a non-UTC offset is valid RFC 3339");
        assert_eq!(
            parsed,
            StandingBasis::Confirmed {
                last_heard_at: at("2026-10-06T09:00:00Z"),
            }
        );
    }
}
