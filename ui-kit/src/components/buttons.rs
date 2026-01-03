//! Button components.

use leptos::prelude::*;

/// Button variants for different use cases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
}

impl ButtonVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Primary => "ps-btn ps-btn-primary",
            Self::Secondary => "ps-btn ps-btn-secondary",
            Self::Outline => "ps-btn ps-btn-outline",
            Self::Ghost => "ps-btn ps-btn-ghost",
            Self::Danger => "ps-btn ps-btn-danger",
        }
    }
}

/// Button sizes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    fn class(&self) -> &'static str {
        match self {
            Self::Small => "ps-btn-sm",
            Self::Medium => "ps-btn-md",
            Self::Large => "ps-btn-lg",
        }
    }
}

/// A styled button component.
#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let class_name = format!(
        "{} {} {}",
        variant.class(),
        size.class(),
        class
    );

    let is_disabled = disabled || loading;

    view! {
        <button
            class=class_name
            disabled=is_disabled
            on:click=move |_| {
                if !is_disabled {
                    if let Some(cb) = &on_click {
                        cb.run(());
                    }
                }
            }
        >
            {if loading {
                view! {
                    <span class="ps-btn-spinner"></span>
                }.into_any()
            } else {
                children().into_any()
            }}
        </button>
    }
}

/// Icon button for toolbar actions.
#[component]
pub fn IconButton(
    #[prop(default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] title: &'static str,
    children: Children,
) -> impl IntoView {
    let class_name = format!(
        "{} {} ps-btn-icon",
        variant.class(),
        size.class()
    );

    view! {
        <button
            class=class_name
            disabled=disabled
            title=title
            on:click=move |_| {
                if !disabled {
                    if let Some(cb) = &on_click {
                        cb.run(());
                    }
                }
            }
        >
            {children()}
        </button>
    }
}

/// Button styles CSS (inject once).
pub const BUTTON_STYLES: &str = r#"
.ps-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--ps-spacing-sm);
    font-weight: 500;
    border-radius: var(--ps-radius-md);
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
}

.ps-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.ps-btn-sm {
    padding: var(--ps-spacing-xs) var(--ps-spacing-sm);
    font-size: var(--ps-font-sm);
}

.ps-btn-md {
    padding: var(--ps-spacing-sm) var(--ps-spacing-md);
    font-size: var(--ps-font-md);
}

.ps-btn-lg {
    padding: var(--ps-spacing-md) var(--ps-spacing-lg);
    font-size: var(--ps-font-lg);
}

.ps-btn-primary {
    background-color: var(--ps-primary);
    color: white;
}

.ps-btn-primary:hover:not(:disabled) {
    background-color: var(--ps-primary-hover);
}

.ps-btn-secondary {
    background-color: var(--ps-secondary);
    color: white;
}

.ps-btn-outline {
    background-color: transparent;
    border-color: var(--ps-border);
    color: var(--ps-text);
}

.ps-btn-outline:hover:not(:disabled) {
    background-color: var(--ps-surface);
}

.ps-btn-ghost {
    background-color: transparent;
    color: var(--ps-text);
}

.ps-btn-ghost:hover:not(:disabled) {
    background-color: var(--ps-surface);
}

.ps-btn-danger {
    background-color: var(--ps-error);
    color: white;
}

.ps-btn-icon {
    padding: var(--ps-spacing-sm);
    border-radius: var(--ps-radius-md);
}

.ps-btn-spinner {
    width: 1em;
    height: 1em;
    border: 2px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: ps-spin 0.6s linear infinite;
}

@keyframes ps-spin {
    to { transform: rotate(360deg); }
}
"#;
