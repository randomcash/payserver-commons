//! The plugin manifest: what a plugin declares about itself before any of
//! its code runs.

use std::str::FromStr;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{Dependency, FailureMode, PluginId, PluginKind, PluginSlug};

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
    /// The name this plugin's pages live under in a URL.
    ///
    /// Separate from `id`, which stays the identity: reverse-DNS, unique by
    /// construction, and what the schema, artifact and grants are keyed on.
    /// `cash.random.billing` is the right thing to key a database schema on
    /// and the wrong thing to put in front of a person, so a plugin that
    /// serves pages names itself something readable as well - `billing`.
    ///
    /// `None` for a plugin that serves no pages. A plugin that declares
    /// pages and no slug has nowhere for them to live, which
    /// [`Manifest::from_str`] refuses rather than inventing one from the id.
    pub slug: Option<PluginSlug>,
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

/// Where a plugin's page belongs in the host's own interface.
///
/// A plugin cannot know what the rest of the product looks like, so it says
/// what *kind* of page it is and the host decides where that goes. The two
/// kinds are genuinely different audiences, not two positions in one menu.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PagePlacement {
    /// A page the person whose account it is uses. Belongs in the main
    /// navigation, beside the rest of what they do here.
    #[default]
    Nav,
    /// A page for whoever runs the instance. Belongs with the other
    /// server-wide administration, not in the navigation every merchant
    /// sees.
    ///
    /// Always admin-only regardless of [`PageDeclaration::admin_only`]:
    /// there is nowhere else it is reachable from, and a merchant offered a
    /// link to it would be offered a page about other merchants.
    AdminSettings,
}

impl PagePlacement {
    /// Whether only a server admin may be offered this page.
    #[must_use]
    pub fn is_admin_only(self) -> bool {
        matches!(self, Self::AdminSettings)
    }
}

/// One page a plugin says it serves.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PageDeclaration {
    /// The path under `/plugins/{id}/pages/`. No leading slash.
    pub path: String,
    /// What to call it in navigation.
    pub label: String,
    /// Where this page belongs in the host's interface.
    ///
    /// Defaults to the main navigation, which is what a page about the
    /// caller's own account should be.
    #[serde(default)]
    pub placement: PagePlacement,
    /// Whether only a server admin should be offered it.
    ///
    /// Implied by [`PagePlacement::AdminSettings`], so a page placed there
    /// need not repeat it.
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
    slug: Option<PluginSlug>,
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

        // Pages need somewhere to live. Deriving a slug from the id would
        // give `cash.random.billing` a URL of `cash-random-billing`, which is
        // the ugliness this field exists to remove, and a plugin author would
        // not learn they had to choose until they saw it in an address bar.
        if !raw.pages.is_empty() && raw.slug.is_none() {
            return Err(InvalidManifest(
                "this plugin declares pages but no `slug`; pages are served under \
                 the slug, so one is required"
                    .to_string(),
            ));
        }

        Ok(Manifest {
            id: raw.id,
            version: raw.version,
            dependencies: raw.dependencies,
            kind: raw.kind,
            failure_mode,
            ui_schema: raw.ui_schema,
            slug: raw.slug,
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
            slug = "billing"

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

    #[test]
    fn a_plugin_with_pages_must_name_a_slug() {
        let err = r#"
            id = "cash.random.billing"
            version = "0.1.0"
            kind = "filter"

            [[pages]]
            path = "subscriptions"
            label = "Subscriptions"
        "#
        .parse::<Manifest>()
        .unwrap_err();
        assert!(
            err.to_string().contains("slug"),
            "the refusal must name what is missing: {err}"
        );

        let ok: Manifest = r#"
            id = "cash.random.billing"
            version = "0.1.0"
            kind = "filter"
            slug = "billing"

            [[pages]]
            path = "subscriptions"
            label = "Subscriptions"
        "#
        .parse()
        .unwrap();
        assert_eq!(ok.slug.unwrap().as_str(), "billing");
    }

    /// A plugin that serves no pages needs no URL name, and requiring one
    /// would make every action plugin invent a word it never uses.
    #[test]
    fn a_plugin_without_pages_needs_no_slug() {
        let manifest: Manifest = r#"
            id = "cash.random.quiet"
            version = "0.1.0"
            kind = "action"
        "#
        .parse()
        .unwrap();
        assert!(manifest.slug.is_none());
    }

    /// The slug is validated by the manifest parser, so a reserved or
    /// malformed one is refused before the plugin is ever registered.
    #[test]
    fn a_reserved_slug_is_refused_at_parse_time() {
        let err = r#"
            id = "cash.random.sneaky"
            version = "0.1.0"
            kind = "action"
            slug = "invoices"
        "#
        .parse::<Manifest>()
        .unwrap_err();
        assert!(err.to_string().contains("reserved"), "{err}");
    }

    /// The default has to be the merchant's own page. A plugin that says
    /// nothing about placement is describing something for the person whose
    /// account it is, and defaulting the other way would put a plugin's
    /// operator tooling into every merchant's sidebar.
    #[test]
    fn a_page_belongs_in_the_navigation_unless_it_says_otherwise() {
        let manifest: Manifest = r#"
            id = "cash.random.billing"
            version = "0.1.0"
            kind = "filter"
            slug = "billing"

            [[pages]]
            path = "subscription"
            label = "Billing"

            [[pages]]
            path = "subscriptions"
            label = "All merchant subscriptions"
            placement = "admin_settings"
        "#
        .parse()
        .unwrap();

        assert_eq!(manifest.pages[0].placement, PagePlacement::Nav);
        assert!(
            !manifest.pages[0].placement.is_admin_only(),
            "a merchant's own page must not be admin-gated by placement"
        );

        assert_eq!(manifest.pages[1].placement, PagePlacement::AdminSettings);
        assert!(
            manifest.pages[1].placement.is_admin_only(),
            "an admin-settings page is admin-only without having to say so \
             twice - there is nowhere else it is reachable from, and a \
             merchant offered it would be offered a page about other merchants"
        );
    }
}
