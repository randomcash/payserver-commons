//! Plugin identifiers.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

/// A plugin's reverse-DNS identifier, e.g. `cash.random.billing`.
///
/// This is also the plugin's reserved HTTP route prefix
/// (`/plugins/{id}/...`), so it is validated rather than merely parsed: an id
/// containing a path separator or an empty label (`..`, a leading or
/// trailing `.`) could otherwise be used to escape that prefix and shadow a
/// core route.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct PluginId(String);

impl PluginId {
    /// Validates and wraps a reverse-DNS plugin id.
    pub fn new(id: impl Into<String>) -> Result<Self, InvalidPluginId> {
        let id = id.into();
        if id.is_empty() {
            return Err(InvalidPluginId::Empty);
        }
        if id.contains('/') || id.contains('\\') {
            return Err(InvalidPluginId::PathSeparator(id));
        }
        if id.split('.').any(str::is_empty) {
            return Err(InvalidPluginId::EmptyLabel(id));
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(InvalidPluginId::InvalidCharacter(id));
        }
        Ok(Self(id))
    }

    /// The id as the route-safe string it was validated from.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        PluginId::new(raw).map_err(serde::de::Error::custom)
    }
}

/// A plugin id that failed validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPluginId {
    #[error("plugin id must not be empty")]
    Empty,
    #[error("plugin id {0:?} must not contain a path separator")]
    PathSeparator(String),
    #[error(
        "plugin id {0:?} must not contain an empty label (e.g. \"..\", or a leading/trailing \".\")"
    )]
    EmptyLabel(String),
    #[error(
        "plugin id {0:?} must be reverse-DNS: only ASCII letters, digits, '.', '-' and '_' are allowed"
    )]
    InvalidCharacter(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_reverse_dns_id() {
        assert!(PluginId::new("cash.random.billing").is_ok());
    }

    #[test]
    fn rejects_path_separator() {
        assert_eq!(
            PluginId::new("cash.random/../etc").unwrap_err(),
            InvalidPluginId::PathSeparator("cash.random/../etc".to_string())
        );
    }

    #[test]
    fn rejects_dot_dot_without_a_slash() {
        assert_eq!(
            PluginId::new("cash..random").unwrap_err(),
            InvalidPluginId::EmptyLabel("cash..random".to_string())
        );
    }

    #[test]
    fn rejects_leading_and_trailing_dot() {
        assert!(PluginId::new(".cash.random").is_err());
        assert!(PluginId::new("cash.random.").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(PluginId::new("").unwrap_err(), InvalidPluginId::Empty);
    }
}
