//! Recovery phrase setup component.

use leptos::prelude::*;

/// Recovery setup step.
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStep {
    /// Show recovery phrase.
    ShowPhrase,
    /// Confirm user has saved phrase.
    Confirm,
    /// Setup complete.
    Complete,
}

/// Recovery setup component.
///
/// Shows the recovery mnemonic phrase and allows the user to confirm
/// they've saved it, or skip for now.
#[component]
pub fn RecoverySetup(
    /// The mnemonic phrase words.
    mnemonic_words: Vec<String>,
    /// Callback when user confirms they've saved the phrase.
    on_confirm: Callback<()>,
    /// Callback when user skips recovery setup.
    on_skip: Callback<()>,
    /// Whether to allow skipping (default: true).
    #[prop(optional, default = true)]
    allow_skip: bool,
    /// Whether a request is in progress (disables buttons).
    #[prop(optional)]
    loading: Signal<bool>,
) -> impl IntoView {
    let (step, set_step) = signal(RecoveryStep::ShowPhrase);
    let (confirmed, set_confirmed) = signal(false);
    let is_loading = move || loading.get();

    let words = mnemonic_words.clone();
    let word_count = words.len();

    let handle_continue = move |_| {
        set_step.set(RecoveryStep::Confirm);
    };

    let handle_confirm = move |_| {
        if confirmed.get() && !is_loading() {
            // Don't set step to Complete here - let parent handle success
            on_confirm.run(());
        }
    };

    let handle_skip = {
        let on_skip = on_skip.clone();
        move |_| {
            on_skip.run(());
        }
    };

    view! {
        <div class="ps-recovery-setup">
            {move || match step.get() {
                RecoveryStep::ShowPhrase => {
                    let words = words.clone();
                    view! {
                        <div class="ps-recovery-show">
                            <h3 class="ps-recovery-title">"Save Your Recovery Phrase"</h3>
                            <p class="ps-recovery-description">
                                "Write down these " {word_count} " words in order. "
                                "This is the only way to recover your account if you lose access."
                            </p>

                            <div class="ps-mnemonic-grid">
                                {words.iter().enumerate().map(|(i, word)| {
                                    let word = word.clone();
                                    view! {
                                        <div class="ps-mnemonic-word">
                                            <span class="ps-mnemonic-index">{i + 1}</span>
                                            <span class="ps-mnemonic-text">{word}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>

                            <div class="ps-recovery-warning">
                                <WarningIcon />
                                <p>
                                    "Never share your recovery phrase. Anyone with these words can access your account."
                                </p>
                            </div>

                            <div class="ps-recovery-actions">
                                <button
                                    type="button"
                                    class="ps-button ps-button-primary"
                                    disabled=is_loading
                                    on:click=handle_continue
                                >
                                    "I've Written It Down"
                                </button>
                                {if allow_skip {
                                    view! {
                                        <button
                                            type="button"
                                            class="ps-button ps-button-ghost"
                                            disabled=is_loading
                                            on:click=handle_skip.clone()
                                        >
                                            {move || if is_loading() { "Processing..." } else { "Skip for Now" }}
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <></> }.into_any()
                                }}
                            </div>
                        </div>
                    }.into_any()
                }

                RecoveryStep::Confirm => view! {
                    <div class="ps-recovery-confirm">
                        <h3 class="ps-recovery-title">"Confirm Your Recovery Phrase"</h3>
                        <p class="ps-recovery-description">
                            "Please confirm that you have saved your recovery phrase securely."
                        </p>

                        <label class="ps-checkbox-label">
                            <input
                                type="checkbox"
                                class="ps-checkbox"
                                prop:checked=move || confirmed.get()
                                on:change=move |ev| {
                                    set_confirmed.set(event_target_checked(&ev));
                                }
                            />
                            <span>
                                "I have written down my recovery phrase and stored it securely. "
                                "I understand that if I lose it, I will not be able to recover my account."
                            </span>
                        </label>

                        <div class="ps-recovery-actions">
                            <button
                                type="button"
                                class="ps-button ps-button-primary"
                                disabled=move || !confirmed.get() || is_loading()
                                on:click=handle_confirm
                            >
                                {move || if is_loading() { "Completing..." } else { "Complete Setup" }}
                            </button>
                            <button
                                type="button"
                                class="ps-button ps-button-ghost"
                                disabled=is_loading
                                on:click=move |_| set_step.set(RecoveryStep::ShowPhrase)
                            >
                                "Back"
                            </button>
                        </div>
                    </div>
                }.into_any(),

                RecoveryStep::Complete => view! {
                    <div class="ps-recovery-complete">
                        <SuccessIcon />
                        <h3 class="ps-recovery-title">"Recovery Setup Complete"</h3>
                        <p class="ps-recovery-description">
                            "Your account is now protected with a recovery phrase."
                        </p>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

/// Warning icon component.
#[component]
fn WarningIcon() -> impl IntoView {
    view! {
        <svg
            class="ps-warning-icon"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
            <path d="M12 9v4" />
            <path d="M12 17h.01" />
        </svg>
    }
}

/// Success/checkmark icon component.
#[component]
fn SuccessIcon() -> impl IntoView {
    view! {
        <svg
            class="ps-success-icon"
            width="48"
            height="48"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <circle cx="12" cy="12" r="10" />
            <path d="m9 12 2 2 4-4" />
        </svg>
    }
}
