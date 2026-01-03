//! Loading state components.

use leptos::prelude::*;

/// Spinner for loading states.
#[component]
pub fn Spinner(
    #[prop(default = "md")] size: &'static str,
) -> impl IntoView {
    let size_class = format!("ps-spinner-{}", size);
    view! {
        <div class=format!("ps-spinner {}", size_class)></div>
    }
}

/// Skeleton placeholder for content loading.
#[component]
pub fn Skeleton(
    #[prop(optional)] width: Option<&'static str>,
    #[prop(optional)] height: Option<&'static str>,
    #[prop(default = false)] rounded: bool,
) -> impl IntoView {
    let style = format!(
        "width: {}; height: {};",
        width.unwrap_or("100%"),
        height.unwrap_or("1rem")
    );
    let class = if rounded { "ps-skeleton ps-skeleton-rounded" } else { "ps-skeleton" };

    view! {
        <div class=class style=style></div>
    }
}

/// Full-page loading overlay.
#[component]
pub fn LoadingOverlay(
    #[prop(optional)] message: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="ps-loading-overlay">
            <div class="ps-loading-content">
                <Spinner size="lg" />
                {message.map(|m| view! { <span class="ps-loading-message">{m}</span> })}
            </div>
        </div>
    }
}

/// Progress bar.
#[component]
pub fn Progress(
    value: f64,
    #[prop(default = 100.0)] max: f64,
) -> impl IntoView {
    let percentage = (value / max * 100.0).min(100.0).max(0.0);

    view! {
        <div class="ps-progress">
            <div
                class="ps-progress-bar"
                style=format!("width: {}%", percentage)
            ></div>
        </div>
    }
}

/// Loading styles CSS.
pub const LOADING_STYLES: &str = r#"
.ps-spinner {
    border: 2px solid var(--ps-border);
    border-top-color: var(--ps-primary);
    border-radius: 50%;
    animation: ps-spin 0.6s linear infinite;
}

.ps-spinner-sm {
    width: 1rem;
    height: 1rem;
}

.ps-spinner-md {
    width: 1.5rem;
    height: 1.5rem;
}

.ps-spinner-lg {
    width: 2.5rem;
    height: 2.5rem;
}

@keyframes ps-spin {
    to { transform: rotate(360deg); }
}

.ps-skeleton {
    background: linear-gradient(
        90deg,
        var(--ps-surface) 25%,
        var(--ps-border) 50%,
        var(--ps-surface) 75%
    );
    background-size: 200% 100%;
    animation: ps-shimmer 1.5s infinite;
    border-radius: var(--ps-radius-sm);
}

.ps-skeleton-rounded {
    border-radius: var(--ps-radius-full);
}

@keyframes ps-shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
}

.ps-loading-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 9999;
}

.ps-loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--ps-spacing-md);
    padding: var(--ps-spacing-xl);
    background-color: var(--ps-surface);
    border-radius: var(--ps-radius-lg);
}

.ps-loading-message {
    font-size: var(--ps-font-md);
    color: var(--ps-text-muted);
}

.ps-progress {
    width: 100%;
    height: 0.5rem;
    background-color: var(--ps-surface);
    border-radius: var(--ps-radius-full);
    overflow: hidden;
}

.ps-progress-bar {
    height: 100%;
    background-color: var(--ps-primary);
    border-radius: var(--ps-radius-full);
    transition: width 0.3s ease;
}
"#;
