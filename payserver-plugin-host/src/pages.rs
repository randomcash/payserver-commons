//! Host-side page resolution for `GET /plugins/{id}/pages/{path}`.
//!
//! A plugin cannot ship Rust into an already-compiled client, so it ships a
//! [`PageElement`] tree and the client draws it. This module is the host
//! half: which plugin answers for an id, and what happens when it cannot.
//!
//! # Why rendering is async
//!
//! Because a real renderer runs wasm. A plugin's page comes from calling an
//! export on its instance, which goes through `spawn_blocking` and is
//! therefore a future - and a sync trait would leave a caller inside an
//! async handler with nowhere to put it. Blocking on a runtime thread to
//! keep the signature tidy is how an executor deadlocks, so the seam is
//! async and the cost is an `async_trait` box per page request, which is
//! nothing beside the wasm call it precedes.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use payserver_plugin_api::PluginId;

use payserver_plugin_api::page::{PageElement, Viewer};

/// What a plugin implements to answer a page request.
///
/// Returns `None` when `path` is not one of the plugin's pages - distinct
/// from the plugin id itself being unregistered, which [`PageHost`] handles
/// before ever calling this, and distinct again from the plugin being unable
/// to answer at all, which is [`PageRenderError`].
#[async_trait]
pub trait PageRenderer: Send + Sync {
    /// # Errors
    /// The plugin exists and claims the page, but could not produce it -
    /// it trapped, timed out, is disabled, or answered something that is not
    /// a page.
    async fn render_page(
        &self,
        path: &str,
        viewer: Viewer,
    ) -> Result<Option<PageElement>, PageRenderError>;
}

/// A renderer had the request and could not answer it. Carries the host's
/// own message; a caller decides how much of it an end user sees.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct PageRenderError(pub String);

impl PageRenderError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// Why a page request was refused.
///
/// Three outcomes, not two, and the third is the one worth keeping separate:
/// a plugin that trapped is not a page that does not exist. Collapsing
/// `Unavailable` into `PageNotFound` would answer 404 for a billing page
/// that is merely broken, and "there is no such page" is a much more
/// convincing lie than "this did not work".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PageError {
    #[error("plugin not found")]
    PluginNotFound,
    #[error("page not found")]
    PageNotFound,
    #[error("the plugin could not render this page: {0}")]
    Unavailable(PageRenderError),
}

/// The host's page-rendering registry, keyed by plugin id.
///
/// Deliberately separate from [`super::PluginRegistry`]: that registry is
/// the load-time manifest gate, decided once at startup before any plugin
/// code runs. This one is consulted per request and holds the thing that
/// actually produces a page - a different lifecycle, a different concern.
#[derive(Clone, Default)]
pub struct PageHost {
    renderers: HashMap<PluginId, Arc<dyn PageRenderer>>,
}

impl PageHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: PluginId, renderer: Arc<dyn PageRenderer>) {
        self.renderers.insert(id, renderer);
    }

    /// Resolves a page request. Same method for merchant and admin: the
    /// plugin decides what to return for the [`Viewer`] it is handed, and
    /// this host never substitutes its own judgment for the plugin's - it
    /// only decides who is asking, not what they should see.
    pub async fn render(
        &self,
        id: &PluginId,
        path: &str,
        viewer: Viewer,
    ) -> Result<PageElement, PageError> {
        let renderer = Arc::clone(self.renderers.get(id).ok_or(PageError::PluginNotFound)?);
        renderer
            .render_page(path, viewer)
            .await
            .map_err(PageError::Unavailable)?
            .ok_or(PageError::PageNotFound)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use payserver_plugin_api::page::{Badge, Card, Tone};

    fn plugin_id(s: &str) -> PluginId {
        PluginId::new(s).unwrap()
    }

    /// A plugin that answers differently for a merchant and an admin, and
    /// asserts it is never asked to choose - it just returns what it is told
    /// to return for the `Viewer` it is handed.
    struct RoleAwareRenderer;

    #[async_trait]
    impl PageRenderer for RoleAwareRenderer {
        async fn render_page(
            &self,
            path: &str,
            viewer: Viewer,
        ) -> Result<Option<PageElement>, PageRenderError> {
            if path != "dashboard" {
                return Ok(None);
            }
            let text = match viewer {
                Viewer::Merchant => "Your balance",
                Viewer::Admin => "All merchant balances",
            };
            Ok(Some(PageElement::Card(Card {
                title: None,
                children: vec![PageElement::Badge(Badge {
                    text: text.to_string(),
                    tone: Tone::Info,
                })],
            })))
        }
    }

    /// A renderer that claims every page and answers none of them, so the
    /// difference between "no such page" and "this broke" has something to
    /// be tested against.
    struct BrokenRenderer;

    #[async_trait]
    impl PageRenderer for BrokenRenderer {
        async fn render_page(
            &self,
            _path: &str,
            _viewer: Viewer,
        ) -> Result<Option<PageElement>, PageRenderError> {
            Err(PageRenderError::new("call timed out"))
        }
    }

    fn badge_text(element: &PageElement) -> &str {
        let PageElement::Card(card) = element else {
            panic!("expected a card");
        };
        let [PageElement::Badge(badge)] = card.children.as_slice() else {
            panic!("expected exactly one badge child");
        };
        &badge.text
    }

    #[tokio::test]
    async fn refuses_an_unregistered_plugin() {
        let host = PageHost::new();
        let err = host
            .render(
                &plugin_id("cash.random.billing"),
                "dashboard",
                Viewer::Merchant,
            )
            .await
            .unwrap_err();
        assert_eq!(err, PageError::PluginNotFound);
    }

    #[tokio::test]
    async fn refuses_a_path_the_plugin_does_not_serve() {
        let mut host = PageHost::new();
        host.register(
            plugin_id("cash.random.billing"),
            Arc::new(RoleAwareRenderer),
        );

        let err = host
            .render(
                &plugin_id("cash.random.billing"),
                "not-a-real-page",
                Viewer::Merchant,
            )
            .await
            .unwrap_err();
        assert_eq!(err, PageError::PageNotFound);
    }

    /// Ticket requirement: a page requested by a merchant and by an admin
    /// can differ, and the plugin sees the role it was given rather than one
    /// it chose - `PageHost` never inspects or rewrites what comes back.
    #[tokio::test]
    async fn the_same_plugin_and_path_can_differ_by_viewer() {
        let mut host = PageHost::new();
        host.register(
            plugin_id("cash.random.billing"),
            Arc::new(RoleAwareRenderer),
        );

        let merchant_page = host
            .render(
                &plugin_id("cash.random.billing"),
                "dashboard",
                Viewer::Merchant,
            )
            .await
            .unwrap();
        let admin_page = host
            .render(
                &plugin_id("cash.random.billing"),
                "dashboard",
                Viewer::Admin,
            )
            .await
            .unwrap();

        assert_eq!(badge_text(&merchant_page), "Your balance");
        assert_eq!(badge_text(&admin_page), "All merchant balances");
        assert_ne!(merchant_page, admin_page);
    }

    /// A plugin that cannot answer must not be reported as a page that does
    /// not exist. Delete `PageError::Unavailable` and fold it into
    /// `PageNotFound` and this is the test that goes red.
    #[tokio::test]
    async fn a_renderer_that_fails_is_not_a_missing_page() {
        let mut host = PageHost::new();
        host.register(plugin_id("cash.random.billing"), Arc::new(BrokenRenderer));

        let err = host
            .render(
                &plugin_id("cash.random.billing"),
                "subscriptions",
                Viewer::Merchant,
            )
            .await
            .unwrap_err();

        assert_eq!(
            err,
            PageError::Unavailable(PageRenderError::new("call timed out"))
        );
        assert_ne!(err, PageError::PageNotFound);
        assert!(
            err.to_string().contains("call timed out"),
            "the host's own reason must survive for an operator to read: {err}"
        );
    }
}
