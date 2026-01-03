//! Form components.

use leptos::prelude::*;

/// Text input component.
#[component]
pub fn Input(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] label: Option<&'static str>,
    #[prop(default = "text")] input_type: &'static str,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] error: Option<String>,
) -> impl IntoView {
    let has_error = error.is_some();
    let error_clone = error.clone();

    view! {
        <div class="ps-input-wrapper">
            {label.map(|l| view! { <label class="ps-label">{l}</label> })}
            <input
                type=input_type
                class="ps-input"
                class:ps-input-error=has_error
                placeholder=placeholder
                disabled=disabled
                prop:value=move || value.get()
                on:input=move |ev| {
                    let target = event_target::<web_sys::HtmlInputElement>(&ev);
                    value.set(target.value());
                }
            />
            {error_clone.map(|e| view! { <span class="ps-input-error-text">{e}</span> })}
        </div>
    }
}

/// Textarea component for longer text.
#[component]
pub fn Textarea(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] label: Option<&'static str>,
    #[prop(default = 4)] rows: u32,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] error: Option<String>,
) -> impl IntoView {
    let has_error = error.is_some();
    let error_clone = error.clone();

    view! {
        <div class="ps-input-wrapper">
            {label.map(|l| view! { <label class="ps-label">{l}</label> })}
            <textarea
                class="ps-textarea"
                class:ps-input-error=has_error
                placeholder=placeholder
                disabled=disabled
                rows=rows
                prop:value=move || value.get()
                on:input=move |ev| {
                    let target = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                    value.set(target.value());
                }
            />
            {error_clone.map(|e| view! { <span class="ps-input-error-text">{e}</span> })}
        </div>
    }
}

/// Checkbox component.
#[component]
pub fn Checkbox(
    checked: RwSignal<bool>,
    #[prop(optional)] label: Option<&'static str>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    view! {
        <label class="ps-checkbox-wrapper">
            <input
                type="checkbox"
                class="ps-checkbox"
                disabled=disabled
                prop:checked=move || checked.get()
                on:change=move |ev| {
                    let target = event_target::<web_sys::HtmlInputElement>(&ev);
                    checked.set(target.checked());
                }
            />
            {label.map(|l| view! { <span class="ps-checkbox-label">{l}</span> })}
        </label>
    }
}

/// Form styles CSS (inject once).
pub const FORM_STYLES: &str = r#"
.ps-input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
}

.ps-label {
    font-size: var(--ps-font-sm);
    font-weight: 500;
    color: var(--ps-text);
}

.ps-input,
.ps-textarea,
.ps-select {
    padding: var(--ps-spacing-sm) var(--ps-spacing-md);
    font-size: var(--ps-font-md);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-md);
    background-color: var(--ps-background);
    color: var(--ps-text);
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.ps-input:focus,
.ps-textarea:focus,
.ps-select:focus {
    outline: none;
    border-color: var(--ps-primary);
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.ps-input:disabled,
.ps-textarea:disabled,
.ps-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background-color: var(--ps-surface);
}

.ps-input-error {
    border-color: var(--ps-error);
}

.ps-input-error:focus {
    box-shadow: 0 0 0 3px rgba(220, 38, 38, 0.1);
}

.ps-input-error-text {
    font-size: var(--ps-font-sm);
    color: var(--ps-error);
}

.ps-checkbox-wrapper {
    display: inline-flex;
    align-items: center;
    gap: var(--ps-spacing-sm);
    cursor: pointer;
}

.ps-checkbox {
    width: 1rem;
    height: 1rem;
    cursor: pointer;
}

.ps-checkbox-label {
    font-size: var(--ps-font-md);
    color: var(--ps-text);
}

.ps-select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%2364748b' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--ps-spacing-sm) center;
    padding-right: calc(var(--ps-spacing-md) + 1rem);
}
"#;
