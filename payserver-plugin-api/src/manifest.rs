//! The plugin manifest: what a plugin declares about itself before any of
//! its code runs.

use std::str::FromStr;

use semver::Version;
use serde::{Deserialize, Serialize};

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
    /// The pages this plugin serves, in the order it wants them listed.
    ///
    /// Declared rather than discovered, because the client has to build
    /// navigation *before* it asks for a page - and a host that had to call
    /// every plugin to find out what to put in a menu would run wasm to draw
    /// a sidebar.
    ///
    /// It also keeps a plugin's own vocabulary out of the client. The client
    /// renders whatever is declared here; it never knows that one of these
    /// happens to be billing.
    pub pages: Vec<PageDeclaration>,
}

/// One page a plugin says it serves.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PageDeclaration {
    /// The path under `/plugins/{id}/pages/`. No leading slash.
    pub path: String,
    /// What to call it in navigation.
    pub label: String,
    /// Whether only a server admin should be offered it.
    ///
    /// Navigation only. A plugin still decides what to return for the
    /// [`Viewer`](crate::page::Viewer) it is handed, and the host still
    /// resolves that viewer from the session - so this hides a menu entry
    /// and is not what stops a merchant reading an operator's page. A
    /// manifest is the plugin's own claim about itself, and claims are not
    /// access control.
    #[serde(default)]
    pub admin_only: bool,
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
    #[serde(default)]
    pages: Vec<PageDeclaration>,
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
            pages: raw.pages,
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

    /// Pages are optional, and a manifest that declares none must still
    /// parse - every plugin that existed before pages did declares none.
    #[test]
    fn a_manifest_without_pages_parses_with_an_empty_list() {
        let manifest: Manifest = r#"
            id = "cash.random.quiet"
            version = "0.1.0"
            kind = "action"
        "#
        .parse()
        .unwrap();
        assert!(manifest.pages.is_empty());
    }

    #[test]
    fn declared_pages_keep_their_order_and_default_to_everyone() {
        let manifest: Manifest = r#"
            id = "cash.random.billing"
            version = "0.1.0"
            kind = "filter"
            ui_schema = 1

            [[pages]]
            path = "subscription"
            label = "Subscription"

            [[pages]]
            path = "subscriptions"
            label = "Subscriptions"
            admin_only = true
        "#
        .parse()
        .unwrap();

        assert_eq!(manifest.pages.len(), 2);
        assert_eq!(
            manifest.pages[0].path, "subscription",
            "order is the plugin's, and it is what a menu is built from"
        );
        assert!(
            !manifest.pages[0].admin_only,
            "a page that says nothing is offered to everyone"
        );
        assert!(manifest.pages[1].admin_only);
    }
}
