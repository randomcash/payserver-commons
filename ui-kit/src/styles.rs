//! The stylesheet for the components this crate ships.
//!
//! ui-kit emitted 99 `ps-*` classes and shipped CSS for none of them, so every
//! consumer reimplemented the styling of components it did not write - and 47
//! of those classes ended up with no rule anywhere. A shared component could
//! therefore be visibly broken with nothing wrong in either repository, which
//! is how the checkout QR card came to show a raw browser button beside a
//! styled one.
//!
//! # Why a stylesheet and not a `<style>` per component
//!
//! Per-component style tags duplicate their rules once per render, make a
//! consumer override a specificity fight, and cannot be themed. This is one
//! string, injected once, made of the consumer's own design tokens - so a
//! payserver themes ui-kit by defining variables, and rules it writes after the
//! injection win on equal specificity.
//!
//! The cost is the CSS living in the wasm bundle. It is a few KB against a
//! 2 MB binary; the alternative is an asset the consumer must wire into Trunk
//! for a dependency that lives in the cargo git cache.

use leptos::prelude::*;

/// The raw stylesheet, for a consumer that would rather place it itself.
pub const STYLES: &str = include_str!("../styles/ui-kit.css");

/// Include ui-kit's styles. Render once, at app root.
///
/// Put it before the consumer's own stylesheet so consumer rules override on
/// equal specificity.
#[component]
pub fn UiKitStyles() -> impl IntoView {
    view! { <style>{STYLES}</style> }
}
