//! Passkey authentication component.

use leptos::prelude::*;

use crate::auth::webauthn::is_webauthn_available;

/// State of the passkey authentication.
#[derive(Debug, Clone, PartialEq)]
pub enum PasskeyState {
    /// WebAuthn not available.
    NotAvailable,
    /// Ready for input.
    Ready,
    /// Waiting for authenticator.
    Authenticating,
    /// Authentication succeeded.
    Success,
    /// Authentication failed.
    Error(String),
}

/// Passkey authentication form component.
///
/// Uses discoverable credentials - no email required for either login or registration.
/// For login: Shows "Sign in with Passkey" button (authenticator shows available passkeys).
/// For registration: Shows "Create Passkey" button (creates a new passkey).
#[component]
pub fn PasskeyAuthForm(
    /// Whether this is for registration (true) or login (false).
    #[prop(optional, default = false)]
    is_registration: bool,
    /// Callback when authentication starts. Receives empty string (email no longer needed).
    on_submit: Callback<String>,
    /// Current state (managed by parent).
    state: ReadSignal<PasskeyState>,
    /// Error message to display (managed by parent).
    #[prop(optional)]
    error: Option<ReadSignal<Option<String>>>,
) -> impl IntoView {
    let (local_state, _set_local_state) = signal(if is_webauthn_available() {
        PasskeyState::Ready
    } else {
        PasskeyState::NotAvailable
    });

    // Use provided state, falling back to local for WebAuthn availability check
    let current_state = if is_webauthn_available() { state } else { local_state };

    let handle_click = move |_| {
        // No email needed - pass empty string for compatibility
        on_submit.run(String::new());
    };

    let button_text = if is_registration {
        "Create Passkey"
    } else {
        "Sign in with Passkey"
    };

    view! {
        <div class="ps-passkey-auth">
            {move || match current_state.get() {
                PasskeyState::NotAvailable => view! {
                    <div class="ps-passkey-not-available">
                        <p class="ps-passkey-error">
                            "Passkeys are not supported in this browser. "
                            "Please use a modern browser like Chrome, Safari, or Firefox."
                        </p>
                    </div>
                }.into_any(),

                _ => view! {
                    <div class="ps-passkey-form">
                        {move || {
                            if let Some(error_signal) = error {
                                if let Some(err) = error_signal.get() {
                                    return view! {
                                        <p class="ps-passkey-error">{err}</p>
                                    }.into_any();
                                }
                            }
                            if let PasskeyState::Error(ref msg) = current_state.get() {
                                return view! {
                                    <p class="ps-passkey-error">{msg.clone()}</p>
                                }.into_any();
                            }
                            view! { <></> }.into_any()
                        }}

                        <button
                            type="button"
                            class="ps-passkey-button"
                            disabled=move || current_state.get() == PasskeyState::Authenticating
                            on:click=handle_click
                        >
                            {move || {
                                if current_state.get() == PasskeyState::Authenticating {
                                    view! {
                                        <span class="ps-spinner"></span>
                                        <span>"Waiting for authenticator..."</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <PasskeyIcon />
                                        <span>{button_text}</span>
                                    }.into_any()
                                }
                            }}
                        </button>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

/// Simple passkey/fingerprint icon component.
#[component]
fn PasskeyIcon() -> impl IntoView {
    view! {
        <svg
            class="ps-passkey-icon"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M12 10a2 2 0 0 0-2 2c0 1.02.76 2 2 2" />
            <path d="M12 14c1.24 0 2-.98 2-2a2 2 0 0 0-2-2" />
            <path d="M17 12a5 5 0 0 0-5-5" />
            <path d="M12 17a5 5 0 0 0 5-5" />
            <path d="M7 12a5 5 0 0 1 5-5" />
            <path d="M12 17a5 5 0 0 1-5-5" />
            <path d="M12 2a10 10 0 0 0-7.07 17.07" />
            <path d="M12 2a10 10 0 0 1 7.07 17.07" />
            <path d="M12 22v-5" />
        </svg>
    }
}
