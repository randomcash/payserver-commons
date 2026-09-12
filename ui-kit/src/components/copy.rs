//! Copy-to-clipboard, written once.
//!
//! This existed five times across the two repositories — twice here as private
//! functions a consumer could not reach, three times in payserver-client — and
//! the copies did not agree. One confirmed with a browser `alert()`, the rest
//! with inline text. One copied a masked key where the user had asked for the
//! real one.
//!
//! # Why this one handles failure
//!
//! Every one of those five discarded the result with `let _ =`. The Clipboard
//! API returns a promise, and it rejects: outside a user gesture, without a
//! secure context, and in several embedded browsers — including the in-app
//! browsers a merchant is quite likely to open a payment link in. All five
//! copies told the user "Copied!" and then did nothing, which is worse than
//! refusing, because the user pastes a stale buffer somewhere that matters.
//!
//! An address the merchant believes they copied is how funds go to the wrong
//! place, so this awaits the promise and says so when it fails.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_futures::JsFuture;

/// How long the confirmation stays up before the control returns to rest.
const FEEDBACK_MS: u32 = 2_000;

/// What the button is currently saying.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CopyState {
    /// At rest, offering to copy.
    Idle,
    /// The clipboard accepted the text.
    Copied,
    /// The clipboard refused, or there was no clipboard to ask.
    Failed,
}

impl CopyState {
    /// The full class list for the button in this state.
    ///
    /// Returns the base class as well as the modifier, so `class="ps-copy-button"`
    /// cannot be written by hand and silently lose the state styling — the same
    /// reasoning as `TimelineState::row_class`, and the reason the timeline dots
    /// were wrong on four rows.
    pub fn button_class(self) -> &'static str {
        match self {
            CopyState::Idle => "ps-copy-button",
            CopyState::Copied => "ps-copy-button ps-copy-button-copied",
            CopyState::Failed => "ps-copy-button ps-copy-button-failed",
        }
    }
}

/// Write `text` to the clipboard, reporting whether it actually landed.
///
/// Public because a few call sites need the behaviour without the button —
/// copying as a side effect of another action, for instance. They get the same
/// failure reporting rather than writing a sixth silent version.
pub async fn copy_to_clipboard(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let promise = window.navigator().clipboard().write_text(text);
    JsFuture::from(promise).await.is_ok()
}

/// A button that copies `text` and says what happened.
#[component]
pub fn CopyButton(
    /// The text placed on the clipboard.
    text: String,
    /// Label at rest. Defaults to "Copy".
    #[prop(optional, into)]
    label: Option<String>,
    /// Extra classes appended to the component's own.
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let (state, set_state) = signal(CopyState::Idle);
    let text = StoredValue::new(text);
    let label = label.unwrap_or_else(|| "Copy".to_string());
    let extra = class.unwrap_or_default();

    let on_click = move |_| {
        let value = text.get_value();
        spawn_local(async move {
            let ok = copy_to_clipboard(&value).await;

            // `try_set` throughout, not `set`. A copy button lives on pages that
            // navigate — the checkout in particular — and writing a disposed
            // signal is a panic, which is how the client crashed during passkey
            // registration. Being load-bearing and hand-copied is what made that
            // possible; here it is written once, structurally.
            let _ = set_state.try_set(if ok {
                CopyState::Copied
            } else {
                CopyState::Failed
            });

            gloo_timers::callback::Timeout::new(FEEDBACK_MS, move || {
                let _ = set_state.try_set(CopyState::Idle);
            })
            .forget();
        });
    };

    view! {
        <button
            type="button"
            class=move || {
                let base = state.get().button_class();
                if extra.is_empty() { base.to_string() } else { format!("{base} {extra}") }
            }
            // Announce the outcome to assistive technology. The visual label
            // change is invisible to a screen reader without it.
            aria-live="polite"
            on:click=on_click
        >
            {move || match state.get() {
                CopyState::Idle => label.clone(),
                CopyState::Copied => "Copied!".to_string(),
                CopyState::Failed => "Press Ctrl+C".to_string(),
            }}
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_carries_the_base_class() {
        // The point of returning the full list: a caller cannot write the base
        // class by hand and lose the modifier, and cannot write a modifier that
        // has lost its base.
        for state in [CopyState::Idle, CopyState::Copied, CopyState::Failed] {
            assert!(
                state.button_class().starts_with("ps-copy-button"),
                "{state:?} must carry the base class"
            );
        }
    }

    #[test]
    fn the_crate_root_exports_the_helper() {
        // Names it through the crate root exactly as a consumer does. A `pub`
        // item can be untouchable from outside while every in-crate test passes:
        // `payment_request_uri` was added to a module's re-export list but not
        // the root's, and six passing tests said nothing, because they all
        // called it by its in-crate path.
        // An async fn's return type is opaque, so this names the paths rather
        // than coercing them - which is all reachability needs.
        let _ = crate::copy_to_clipboard;
        let _ = crate::CopyButton;
        let _ = crate::CopyState::Idle;
    }

    #[test]
    fn the_two_outcomes_look_different_from_rest_and_from_each_other() {
        // A failure that styles like a success is the bug this component exists
        // to remove, so it is worth an assertion rather than an eyeball.
        let idle = CopyState::Idle.button_class();
        let copied = CopyState::Copied.button_class();
        let failed = CopyState::Failed.button_class();
        assert_ne!(idle, copied);
        assert_ne!(idle, failed);
        assert_ne!(copied, failed);
    }
}
