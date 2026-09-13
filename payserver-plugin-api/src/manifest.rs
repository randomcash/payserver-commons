//! The plugin manifest: what a plugin declares about itself before any of
//! its code runs.

use std::str::FromStr;

use semver::Version;
use serde::Deserialize;

use crate::{Dependency, FailureMode, PluginId, PluginKind};

/// A parsed and validated plugin manifest.
///
/// Construct via [`Manifest::from_str`] (or `str::parse`), never by
/// constructing the fields directly — that is what applies the
/// `failure_mode` default for filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub id: PluginId,
    pub version: Version,
    pub dependencies: Vec<Dependency>,
    pub kind: PluginKind,
    /// `None` for a non-filter plugin that did not declare one. Always
    /// `Some` for a filter: [`FailureMode::Closed`] if the manifest omitted
    /// it.
    pub failure_mode: Option<FailureMode>,
    pub ui_schema: Option<u32>,
}

impl Manifest {
    /// The manifest's dependency on `name`, if it declares one.
    pub fn dependency(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

/// The manifest exactly as written, before the `failure_mode` default for
/// filters is applied.
#[derive(Debug, Deserialize)]
struct RawManifest {
    id: PluginId,
    version: Version,
    #[serde(default)]
    dependencies: Vec<Dependency>,
    kind: PluginKind,
    #[serde(default)]
    failure_mode: Option<FailureMode>,
    #[serde(default)]
    ui_schema: Option<u32>,
}

impl FromStr for Manifest {
    type Err = InvalidManifest;

    fn from_str(toml: &str) -> Result<Self, Self::Err> {
        let raw: RawManifest = toml::from_str(toml).map_err(|e| InvalidManifest(e.to_string()))?;

        // A filter that does not say how to fail is deliberately treated as
        // closed: silently letting an unvetted action through is worse than
        // blocking one that a healthy filter would have allowed.
        let failure_mode = if raw.kind.is_filter() {
            Some(raw.failure_mode.unwrap_or(FailureMode::Closed))
        } else {
            raw.failure_mode
        };

        Ok(Manifest {
            id: raw.id,
            version: raw.version,
            dependencies: raw.dependencies,
            kind: raw.kind,
            failure_mode,
            ui_schema: raw.ui_schema,
        })
    }
}

/// A manifest that failed to parse or validate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid plugin manifest: {0}")]
pub struct InvalidManifest(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_complete_manifest() {
        let manifest: Manifest = r#"
            id = "cash.random.billing"
            version = "0.1.0"
            dependencies = ["ethpayserver:^1.2.0"]
            kind = "action"
            ui_schema = 1
        "#
        .parse()
        .unwrap();

        assert_eq!(manifest.id.as_str(), "cash.random.billing");
        assert_eq!(manifest.version, Version::new(0, 1, 0));
        assert_eq!(manifest.kind, PluginKind::Action);
        assert_eq!(manifest.failure_mode, None);
        assert_eq!(manifest.ui_schema, Some(1));
        assert!(
            manifest
                .dependency("ethpayserver")
                .unwrap()
                .is_satisfied_by(&Version::new(1, 2, 5))
        );
    }

    /// Ticket test 2: a filter manifest with no `failure_mode` parses as
    /// closed. Before this default was wired in, an absent `failure_mode`
    /// deserialized as `None` for a filter too, so this test failed red
    /// against that version.
    #[test]
    fn filter_without_failure_mode_defaults_to_closed() {
        let manifest: Manifest = r#"
            id = "cash.random.riskfilter"
            version = "0.1.0"
            dependencies = ["ethpayserver:^1.2.0"]
            kind = "filter"
        "#
        .parse()
        .unwrap();

        assert_eq!(manifest.failure_mode, Some(FailureMode::Closed));
    }

    #[test]
    fn filter_can_declare_open() {
        let manifest: Manifest = r#"
            id = "cash.random.riskfilter"
            version = "0.1.0"
            dependencies = ["ethpayserver:^1.2.0"]
            kind = "filter"
            failure_mode = "open"
        "#
        .parse()
        .unwrap();

        assert_eq!(manifest.failure_mode, Some(FailureMode::Open));
    }

    /// Ticket test 3: a plugin id containing a path separator or `..` is
    /// refused at parse time, not deferred to route registration.
    #[test]
    fn rejects_unsafe_id_at_parse_time() {
        let err = r#"
            id = "cash.random/../etc"
            version = "0.1.0"
            dependencies = ["ethpayserver:^1.2.0"]
            kind = "action"
        "#
        .parse::<Manifest>()
        .unwrap_err();

        assert!(err.to_string().contains("path separator"));
    }

    #[test]
    fn rejects_malformed_toml() {
        assert!("not a manifest".parse::<Manifest>().is_err());
    }
}
