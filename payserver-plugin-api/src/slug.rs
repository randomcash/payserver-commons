//! The name a plugin's pages live under in a URL.
//!
//! Separate from [`PluginId`](crate::PluginId), and the split is the point.
//! An id is an identity: reverse-DNS, globally unique by construction, and
//! what a schema, an artifact and a set of grants are keyed on. It is exactly
//! what you want for those and exactly what you do not want in a URL a person
//! reads - `cash.random.billing` in an address bar is an implementation
//! detail leaking into the product.
//!
//! A slug is the other half: short, lowercase, and chosen to read well.
//! `/billing/subscriptions`.
//!
//! # What makes a slug safe
//!
//! The id was safe to put in a path *by construction* - nothing else could
//! collide with `cash.random.billing`. A short slug gives that up, so the
//! safety has to be enforced instead:
//!
//! - it matches a strict shape, so it cannot contain a path separator, a
//!   dot segment, or an encoded surprise;
//! - it is not one of the names the host has reserved for itself;
//! - it is unique among installed plugins, which the host checks at install
//!   because only the host knows what else is installed.
//!
//! The first two are here. The third cannot be: this type validates one slug
//! and knows nothing about any other.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Names a plugin may not take, because the client already routes them.
///
/// Deliberately generous. The cost of reserving a word nobody wanted is that
/// a plugin picks a different one; the cost of missing one is that adding a
/// core route later breaks an installed plugin's URLs, which is a far worse
/// trade. `plugins` is here so the old `/plugins/{id}/...` shape can never be
/// shadowed by a plugin calling itself that.
pub const RESERVED_SLUGS: &[&str] = &[
    "admin",
    "api",
    "auth",
    "billing-admin",
    "checkout",
    "dashboard",
    "evm",
    "health",
    "invoices",
    "login",
    "logout",
    "me",
    "payments",
    "payouts",
    "plugins",
    "rates",
    "refunds",
    "register",
    "settings",
    "static",
    "stores",
    "users",
    "wallets",
    "webhooks",
    "ws",
];

/// Why a slug was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidSlug {
    #[error("a plugin slug must be between 2 and 30 characters, got {0}")]
    Length(usize),
    #[error(
        "a plugin slug must be lowercase letters, digits and single hyphens, \
         starting with a letter and not ending in a hyphen: {0:?}"
    )]
    Shape(String),
    #[error("{0:?} is reserved by the host and cannot be a plugin slug")]
    Reserved(String),
}

/// A validated URL name for a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PluginSlug(String);

impl PluginSlug {
    /// # Errors
    /// See [`InvalidSlug`].
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidSlug> {
        let raw = raw.into();

        if !(2..=30).contains(&raw.chars().count()) {
            return Err(InvalidSlug::Length(raw.chars().count()));
        }

        // Shape, spelled out rather than by regex so each rule is visible:
        // ASCII lowercase and digits and hyphen only, must start with a
        // letter, must not end with a hyphen, and no two hyphens in a row.
        // No dots, so a slug can never be a `.` or `..` segment; no slashes,
        // so it can never be more than one segment; no uppercase, so two
        // slugs cannot differ only by case on a case-insensitive comparison
        // somewhere downstream.
        let first_is_letter = raw.chars().next().is_some_and(|c| c.is_ascii_lowercase());
        let ends_well = !raw.ends_with('-');
        let chars_ok = raw
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        let no_double_hyphen = !raw.contains("--");

        if !(first_is_letter && ends_well && chars_ok && no_double_hyphen) {
            return Err(InvalidSlug::Shape(raw));
        }

        if RESERVED_SLUGS.contains(&raw.as_str()) {
            return Err(InvalidSlug::Reserved(raw));
        }

        Ok(Self(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PluginSlug {
    type Err = InvalidSlug;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<'de> Deserialize<'de> for PluginSlug {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_names_a_plugin_would_actually_want() {
        for good in ["billing", "tax-report", "kyc2", "a1"] {
            assert!(PluginSlug::new(good).is_ok(), "{good} should be accepted");
        }
    }

    /// Each of these is a way a slug could stop being one path segment, or
    /// could collide with another slug that only looks different.
    #[test]
    fn refuses_anything_that_is_not_one_plain_segment() {
        for bad in [
            "bil/ling",   // a separator - two segments
            "..",         // a dot segment
            ".billing",   // a dot at all
            "Billing",    // case, so two slugs cannot differ only by it
            "1billing",   // must start with a letter
            "billing-",   // trailing hyphen
            "bil--ling",  // doubled hyphen
            "bil ling",   // whitespace
            "billing%2f", // an encoded separator is still not allowed shape
            "a",          // too short to be a name
        ] {
            assert!(
                PluginSlug::new(bad).is_err(),
                "{bad:?} should have been refused"
            );
        }
        assert!(PluginSlug::new("a".repeat(31)).is_err(), "too long");
    }

    /// The whole reason a short slug needs validating at all: it shares a
    /// namespace with the client's own routes, which a reverse-DNS id never
    /// did.
    #[test]
    fn refuses_names_the_host_routes_itself() {
        for reserved in ["invoices", "stores", "settings", "admin", "plugins", "evm"] {
            assert_eq!(
                PluginSlug::new(reserved),
                Err(InvalidSlug::Reserved(reserved.to_string())),
                "{reserved} is a core route and must not be takeable"
            );
        }
    }

    #[test]
    fn deserialises_through_the_same_gate_as_the_constructor() {
        assert!(serde_json::from_str::<PluginSlug>(r#""billing""#).is_ok());
        assert!(
            serde_json::from_str::<PluginSlug>(r#""invoices""#).is_err(),
            "a reserved slug must not slip in through a manifest"
        );
        assert!(serde_json::from_str::<PluginSlug>(r#""bil/ling""#).is_err());
    }
}
