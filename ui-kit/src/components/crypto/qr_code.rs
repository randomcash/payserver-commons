//! QR code generation and display.

use leptos::prelude::*;
use qrcode::{EcLevel, QrCode};

/// QR code display component.
#[component]
pub fn QrCodeDisplay(
    data: String,
    #[prop(default = 200)] size: u32,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let svg = if data.is_empty() {
        String::new()
    } else {
        generate_qr_svg(&data, size).unwrap_or_default()
    };

    view! {
        <div
            class=format!("ps-qr-code {}", class)
            inner_html=svg
        />
    }
}

/// QR code with copy button.
#[component]
pub fn QrCodeCard(
    data: String,
    #[prop(optional)] label: Option<&'static str>,
    #[prop(default = 200)] size: u32,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let data_for_copy = data.clone();

    view! {
        <div class="ps-qr-card">
            {label.map(|l| view! { <span class="ps-qr-label">{l}</span> })}
            <QrCodeDisplay data=data size=size />
            <button
                class="ps-qr-copy-btn"
                on:click=move |_| {
                    copy_to_clipboard(&data_for_copy);
                    set_copied.set(true);
                    gloo_timers::callback::Timeout::new(2000, move || {
                        set_copied.set(false);
                    }).forget();
                }
            >
                {move || if copied.get() { "Copied!" } else { "Copy" }}
            </button>
        </div>
    }
}

/// Copy text to clipboard.
fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

/// Generate SVG representation of QR code.
fn generate_qr_svg(data: &str, size: u32) -> Result<String, qrcode::types::QrError> {
    let code = QrCode::with_error_correction_level(data, EcLevel::M)?;
    let module_count = code.width();
    let module_size = size as f64 / module_count as f64;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}">"#
    );

    // Background
    svg.push_str(&format!(
        r#"<rect width="{size}" height="{size}" fill="white"/>"#
    ));

    // Modules
    for (y, row) in code.to_colors().chunks(module_count).enumerate() {
        for (x, &module) in row.iter().enumerate() {
            if module == qrcode::Color::Dark {
                let px = x as f64 * module_size;
                let py = y as f64 * module_size;
                svg.push_str(&format!(
                    r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="black"/>"#,
                    px,
                    py,
                    module_size + 0.5,
                    module_size + 0.5
                ));
            }
        }
    }

    svg.push_str("</svg>");
    Ok(svg)
}

/// QR code styles CSS.
pub const QR_CODE_STYLES: &str = r#"
.ps-qr-code {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--ps-spacing-md);
    background-color: white;
    border-radius: var(--ps-radius-lg);
}

.ps-qr-code svg {
    display: block;
}

.ps-qr-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--ps-spacing-md);
    padding: var(--ps-spacing-lg);
    background-color: var(--ps-surface);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-lg);
}

.ps-qr-label {
    font-size: var(--ps-font-sm);
    color: var(--ps-text-muted);
}

.ps-qr-copy-btn {
    padding: var(--ps-spacing-sm) var(--ps-spacing-md);
    font-size: var(--ps-font-sm);
    background-color: var(--ps-background);
    border: 1px solid var(--ps-border);
    border-radius: var(--ps-radius-md);
    cursor: pointer;
    transition: background-color 0.15s ease;
}

.ps-qr-copy-btn:hover {
    background-color: var(--ps-surface);
}
"#;
