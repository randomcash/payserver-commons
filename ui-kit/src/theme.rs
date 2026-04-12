//! Theme system for UI Kit.
//!
//! Provides CSS custom properties for consistent styling across components.

use crate::types::Theme;
use leptos::prelude::*;

/// CSS custom properties for theming.
pub mod css_vars {
    pub const PRIMARY: &str = "--ps-primary";
    pub const PRIMARY_HOVER: &str = "--ps-primary-hover";
    pub const SECONDARY: &str = "--ps-secondary";
    pub const BACKGROUND: &str = "--ps-background";
    pub const SURFACE: &str = "--ps-surface";
    pub const TEXT: &str = "--ps-text";
    pub const TEXT_MUTED: &str = "--ps-text-muted";
    pub const BORDER: &str = "--ps-border";
    pub const ERROR: &str = "--ps-error";
    pub const WARNING: &str = "--ps-warning";
    pub const SUCCESS: &str = "--ps-success";
    pub const INFO: &str = "--ps-info";
}

/// Light theme CSS values.
const LIGHT_THEME: &str = r#"
:root {
    --ps-primary: #2563eb;
    --ps-primary-hover: #1d4ed8;
    --ps-secondary: #64748b;
    --ps-background: #ffffff;
    --ps-surface: #f8fafc;
    --ps-text: #0f172a;
    --ps-text-muted: #64748b;
    --ps-border: #e2e8f0;
    --ps-error: #dc2626;
    --ps-warning: #d97706;
    --ps-success: #16a34a;
    --ps-info: #0284c7;
    --ps-spacing-xs: 0.25rem;
    --ps-spacing-sm: 0.5rem;
    --ps-spacing-md: 1rem;
    --ps-spacing-lg: 1.5rem;
    --ps-spacing-xl: 2rem;
    --ps-radius-sm: 0.25rem;
    --ps-radius-md: 0.375rem;
    --ps-radius-lg: 0.5rem;
    --ps-radius-full: 9999px;
    --ps-font-xs: 0.75rem;
    --ps-font-sm: 0.875rem;
    --ps-font-md: 1rem;
    --ps-font-lg: 1.125rem;
    --ps-font-xl: 1.25rem;
}
"#;

/// Dark theme CSS values.
const DARK_THEME: &str = r#"
:root {
    --ps-primary: #3b82f6;
    --ps-primary-hover: #60a5fa;
    --ps-secondary: #94a3b8;
    --ps-background: #0f172a;
    --ps-surface: #1e293b;
    --ps-text: #f8fafc;
    --ps-text-muted: #94a3b8;
    --ps-border: #334155;
    --ps-error: #ef4444;
    --ps-warning: #f59e0b;
    --ps-success: #22c55e;
    --ps-info: #0ea5e9;
    --ps-spacing-xs: 0.25rem;
    --ps-spacing-sm: 0.5rem;
    --ps-spacing-md: 1rem;
    --ps-spacing-lg: 1.5rem;
    --ps-spacing-xl: 2rem;
    --ps-radius-sm: 0.25rem;
    --ps-radius-md: 0.375rem;
    --ps-radius-lg: 0.5rem;
    --ps-radius-full: 9999px;
    --ps-font-xs: 0.75rem;
    --ps-font-sm: 0.875rem;
    --ps-font-md: 1rem;
    --ps-font-lg: 1.125rem;
    --ps-font-xl: 1.25rem;
}
"#;

/// Apply a theme to the document.
pub fn apply_theme(theme: Theme) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    let actual_theme = match theme {
        Theme::Light => Theme::Light,
        Theme::Dark => Theme::Dark,
        Theme::System => {
            // Check system preference
            let prefers_dark = window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .map(|mq| mq.matches())
                .unwrap_or(false);

            if prefers_dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        }
    };

    inject_theme_styles(&document, actual_theme);

    // Set data attribute for CSS selectors
    if let Some(root) = document.document_element() {
        let theme_name = match actual_theme {
            Theme::Light | Theme::System => "light",
            Theme::Dark => "dark",
        };
        let _ = root.set_attribute("data-theme", theme_name);
    }
}

fn inject_theme_styles(document: &web_sys::Document, theme: Theme) {
    const STYLE_ID: &str = "ps-theme-styles";

    // Remove existing theme styles
    if let Some(existing) = document.get_element_by_id(STYLE_ID) {
        existing.remove();
    }

    // Create and inject new styles
    let css = match theme {
        Theme::Light | Theme::System => LIGHT_THEME,
        Theme::Dark => DARK_THEME,
    };

    if let Ok(style) = document.create_element("style") {
        let _ = style.set_attribute("id", STYLE_ID);
        style.set_text_content(Some(css));

        if let Some(head) = document.head() {
            let _ = head.append_child(&style);
        }
    }
}

/// Theme provider component.
#[component]
pub fn ThemeProvider(
    #[prop(default = Theme::System)] initial: Theme,
    children: Children,
) -> impl IntoView {
    let (theme, set_theme) = signal(initial);

    // Apply theme on changes
    Effect::new(move |_| {
        apply_theme(theme.get());
    });

    // Provide theme context
    provide_context((theme, set_theme));

    children()
}

/// Hook to access the current theme.
pub fn use_theme() -> (ReadSignal<Theme>, WriteSignal<Theme>) {
    use_context::<(ReadSignal<Theme>, WriteSignal<Theme>)>()
        .expect("use_theme must be used within ThemeProvider")
}
