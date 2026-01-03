//! Cryptocurrency address display components.

use leptos::prelude::*;

/// Display a truncated crypto address with copy functionality.
#[component]
pub fn Address(
    address: String,
    #[prop(default = 6)] prefix_len: usize,
    #[prop(default = 4)] suffix_len: usize,
    #[prop(default = true)] copyable: bool,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let addr_clone = address.clone();

    let truncated = if address.len() <= prefix_len + suffix_len + 3 {
        address.clone()
    } else {
        format!(
            "{}...{}",
            &address[..prefix_len],
            &address[address.len() - suffix_len..]
        )
    };

    view! {
        <span
            class=format!("ps-address {} {}", if copyable { "ps-address-copyable" } else { "" }, class)
            title=address
            on:click=move |_| {
                if copyable {
                    copy_to_clipboard(&addr_clone);
                    set_copied.set(true);
                    gloo_timers::callback::Timeout::new(2000, move || {
                        set_copied.set(false);
                    }).forget();
                }
            }
        >
            <span class="ps-address-text">{truncated}</span>
            {copyable.then(|| view! {
                <span class="ps-address-icon">
                    {move || if copied.get() { "✓" } else { "📋" }}
                </span>
            })}
        </span>
    }
}

/// Full address display with label and copy button.
#[component]
pub fn AddressDisplay(
    address: String,
    #[prop(optional)] label: Option<&'static str>,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let addr_clone = address.clone();

    view! {
        <div class="ps-address-display">
            {label.map(|l| view! { <label class="ps-label">{l}</label> })}
            <div class="ps-address-box">
                <code class="ps-address-full">{address}</code>
                <button
                    class="ps-address-copy"
                    on:click=move |_| {
                        copy_to_clipboard(&addr_clone);
                        set_copied.set(true);
                        gloo_timers::callback::Timeout::new(2000, move || {
                            set_copied.set(false);
                        }).forget();
                    }
                >
                    {move || if copied.get() { "Copied!" } else { "Copy" }}
                </button>
            </div>
        </div>
    }
}

fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

/// Address styles CSS.
pub const ADDRESS_STYLES: &str = r#"
.ps-address {
    display: inline-flex;
    align-items: center;
    gap: var(--ps-spacing-xs);
    font-family: monospace;
    font-size: var(--ps-font-sm);
    color: var(--ps-text);
}

.ps-address-copyable {
    cursor: pointer;
}

.ps-address-copyable:hover {
    color: var(--ps-primary);
}

.ps-address-icon {
    font-size: 0.875em;
}

.ps-address-display {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
}

.ps-address-box {
    display: flex;
    align-items: center;
    gap: var(--ps-spacing-sm);
    padding: var(--ps-spacing-sm) var(--ps-spacing-md);
    background-color: var(--ps-surface);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-md);
}

.ps-address-full {
    flex: 1;
    font-family: monospace;
    font-size: var(--ps-font-sm);
    word-break: break-all;
    color: var(--ps-text);
}

.ps-address-copy {
    flex-shrink: 0;
    padding: var(--ps-spacing-xs) var(--ps-spacing-sm);
    font-size: var(--ps-font-sm);
    background-color: var(--ps-background);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-sm);
    cursor: pointer;
    transition: background-color 0.15s ease;
}

.ps-address-copy:hover {
    background-color: var(--ps-surface);
}
"#;
