//! Card components.

use leptos::prelude::*;

/// Basic card container.
#[component]
pub fn Card(
    #[prop(optional)] class: &'static str,
    #[prop(default = true)] padding: bool,
    children: Children,
) -> impl IntoView {
    let padding_class = if padding { "ps-card-padded" } else { "" };

    view! {
        <div class=format!("ps-card {} {}", padding_class, class)>
            {children()}
        </div>
    }
}

/// Card with header.
#[component]
pub fn CardWithHeader(
    title: &'static str,
    #[prop(optional)] subtitle: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="ps-card">
            <div class="ps-card-header">
                <div class="ps-card-header-text">
                    <h3 class="ps-card-title">{title}</h3>
                    {subtitle.map(|s| view! { <p class="ps-card-subtitle">{s}</p> })}
                </div>
            </div>
            <div class="ps-card-body">
                {children()}
            </div>
        </div>
    }
}

/// Stat card for dashboards.
#[component]
pub fn StatCard(
    label: &'static str,
    value: String,
    #[prop(optional)] change: Option<String>,
    #[prop(default = true)] positive: bool,
) -> impl IntoView {
    let change_class = if positive { "ps-stat-change-positive" } else { "ps-stat-change-negative" };

    view! {
        <div class="ps-stat-card">
            <span class="ps-stat-label">{label}</span>
            <span class="ps-stat-value">{value}</span>
            {change.map(|c| view! {
                <span class=format!("ps-stat-change {}", change_class)>
                    {c}
                </span>
            })}
        </div>
    }
}

/// Card styles CSS.
pub const CARD_STYLES: &str = r#"
.ps-card {
    background-color: var(--ps-surface);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-lg);
    overflow: hidden;
}

.ps-card-padded {
    padding: var(--ps-spacing-lg);
}

.ps-card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: var(--ps-spacing-md) var(--ps-spacing-lg);
    border-bottom: 1px solid var(--ps-border);
}

.ps-card-header-text {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
}

.ps-card-title {
    margin: 0;
    font-size: var(--ps-font-lg);
    font-weight: 600;
    color: var(--ps-text);
}

.ps-card-subtitle {
    margin: 0;
    font-size: var(--ps-font-sm);
    color: var(--ps-text-muted);
}

.ps-card-actions {
    display: flex;
    gap: var(--ps-spacing-sm);
}

.ps-card-body {
    padding: var(--ps-spacing-lg);
}

.ps-stat-card {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
    padding: var(--ps-spacing-lg);
    background-color: var(--ps-surface);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-lg);
}

.ps-stat-label {
    font-size: var(--ps-font-sm);
    color: var(--ps-text-muted);
}

.ps-stat-value {
    font-size: var(--ps-font-xl);
    font-weight: 600;
    color: var(--ps-text);
}

.ps-stat-change {
    font-size: var(--ps-font-sm);
}

.ps-stat-change-positive {
    color: var(--ps-success);
}

.ps-stat-change-negative {
    color: var(--ps-error);
}
"#;
