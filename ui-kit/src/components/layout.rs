//! Layout components.

use leptos::prelude::*;

/// Main container with max-width.
#[component]
pub fn Container(#[prop(optional)] class: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class=format!("ps-container {}", class)>
            {children()}
        </div>
    }
}

/// Flex stack layout.
#[component]
pub fn Stack(
    #[prop(default = "column")] direction: &'static str,
    #[prop(default = "md")] gap: &'static str,
    #[prop(optional)] align: Option<&'static str>,
    #[prop(optional)] justify: Option<&'static str>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let gap_class = format!("ps-gap-{}", gap);
    let dir_class = if direction == "row" {
        "ps-stack-row"
    } else {
        "ps-stack-col"
    };
    let align_style = align
        .map(|a| format!("align-items: {};", a))
        .unwrap_or_default();
    let justify_style = justify
        .map(|j| format!("justify-content: {};", j))
        .unwrap_or_default();

    view! {
        <div
            class=format!("ps-stack {} {} {}", dir_class, gap_class, class)
            style=format!("{} {}", align_style, justify_style)
        >
            {children()}
        </div>
    }
}

/// Grid layout.
#[component]
pub fn Grid(
    #[prop(default = 1)] cols: u8,
    #[prop(default = "md")] gap: &'static str,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let gap_class = format!("ps-gap-{}", gap);
    let cols_style = format!("grid-template-columns: repeat({}, minmax(0, 1fr));", cols);

    view! {
        <div
            class=format!("ps-grid {} {}", gap_class, class)
            style=cols_style
        >
            {children()}
        </div>
    }
}

/// Page header with title and actions.
#[component]
pub fn PageHeader(
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] actions: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="ps-page-header">
            <div class="ps-page-header-text">
                <h1 class="ps-page-title">{title}</h1>
                {description.map(|d| view! { <p class="ps-page-description">{d}</p> })}
            </div>
            {actions.map(|a| view! { <div class="ps-page-actions">{a}</div> })}
        </div>
    }
}

/// Divider line.
#[component]
pub fn Divider(#[prop(optional)] vertical: bool) -> impl IntoView {
    let class = if vertical {
        "ps-divider-v"
    } else {
        "ps-divider-h"
    };
    view! { <div class=class></div> }
}

/// Layout styles CSS.
pub const LAYOUT_STYLES: &str = r#"
.ps-container {
    width: 100%;
    max-width: 1280px;
    margin: 0 auto;
    padding: 0 var(--ps-spacing-lg);
}

.ps-stack {
    display: flex;
}

.ps-stack-col {
    flex-direction: column;
}

.ps-stack-row {
    flex-direction: row;
}

.ps-grid {
    display: grid;
}

.ps-gap-xs { gap: var(--ps-spacing-xs); }
.ps-gap-sm { gap: var(--ps-spacing-sm); }
.ps-gap-md { gap: var(--ps-spacing-md); }
.ps-gap-lg { gap: var(--ps-spacing-lg); }
.ps-gap-xl { gap: var(--ps-spacing-xl); }

.ps-page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--ps-spacing-xl);
}

.ps-page-header-text {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
}

.ps-page-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--ps-text);
}

.ps-page-description {
    margin: 0;
    font-size: var(--ps-font-md);
    color: var(--ps-text-muted);
}

.ps-page-actions {
    display: flex;
    gap: var(--ps-spacing-sm);
}

.ps-divider-h {
    width: 100%;
    height: 1px;
    background-color: var(--ps-border);
}

.ps-divider-v {
    width: 1px;
    height: 100%;
    background-color: var(--ps-border);
}
"#;
