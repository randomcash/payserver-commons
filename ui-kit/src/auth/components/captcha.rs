//! CAPTCHA widget component for Cloudflare Turnstile.
//!
//! Dynamically loads the Turnstile script and renders the verification widget.
//! The widget calls `on_token` when the user completes the challenge.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

/// Load the Turnstile script if not already present.
fn ensure_turnstile_script() {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("document");

    // Check if script already exists
    if document
        .query_selector("script[data-captcha-turnstile]")
        .ok()
        .flatten()
        .is_some()
    {
        return;
    }

    let script = document.create_element("script").expect("create script");
    script
        .set_attribute(
            "src",
            "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit",
        )
        .expect("set src");
    script
        .set_attribute("async", "true")
        .expect("set async");
    script
        .set_attribute("data-captcha-turnstile", "true")
        .expect("set marker");
    document
        .head()
        .expect("head")
        .append_child(&script)
        .expect("append script");
}

/// Try to render the Turnstile widget into the given container.
/// Returns `true` if rendering succeeded (turnstile API is loaded).
fn try_render_turnstile(
    container_id: &str,
    site_key: &str,
    callback: &Closure<dyn Fn(String)>,
) -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    let turnstile = match js_sys::Reflect::get(&window, &"turnstile".into()) {
        Ok(val) if !val.is_undefined() => val,
        _ => return false,
    };

    let render_fn = match js_sys::Reflect::get(&turnstile, &"render".into()) {
        Ok(val) if val.is_function() => js_sys::Function::from(val),
        _ => return false,
    };

    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&options, &"sitekey".into(), &site_key.into());
    let _ = js_sys::Reflect::set(
        &options,
        &"callback".into(),
        callback.as_ref().unchecked_ref(),
    );

    let _ = render_fn.call2(&turnstile, &container_id.into(), &options);
    true
}

/// Cloudflare Turnstile CAPTCHA widget.
///
/// Loads the Turnstile script, renders the widget, and calls `on_token` with the
/// verification token when the user completes the challenge.
#[component]
pub fn TurnstileWidget(
    /// Turnstile site key from the server's captcha config.
    site_key: String,
    /// Callback invoked with the CAPTCHA token when verification succeeds.
    #[prop(into)]
    on_token: Callback<String>,
) -> impl IntoView {
    let rendered = StoredValue::new(false);

    Effect::new(move || {
        if rendered.get_value() {
            return;
        }

        ensure_turnstile_script();

        let site_key = site_key.clone();
        let callback = Closure::wrap(Box::new(move |token: String| {
            on_token.run(token);
        }) as Box<dyn Fn(String)>);

        // Try rendering immediately (script may already be cached)
        if try_render_turnstile("#captcha-container", &site_key, &callback) {
            rendered.set_value(true);
            callback.forget();
            return;
        }

        // Otherwise poll until the turnstile API is available
        let cb = std::rc::Rc::new(std::cell::RefCell::new(Some(callback)));
        let cb_clone = cb.clone();
        let interval = gloo_timers::callback::Interval::new(100, move || {
            if let Some(ref callback) = *cb_clone.borrow() {
                if try_render_turnstile("#captcha-container", &site_key, callback) {
                    rendered.set_value(true);
                    // Take and forget the closure so it persists
                    if let Some(c) = cb_clone.borrow_mut().take() {
                        c.forget();
                    }
                }
            }
        });

        // Stop polling after 10 seconds
        gloo_timers::callback::Timeout::new(10_000, move || {
            drop(interval);
            drop(cb);
        })
        .forget();
    });

    view! {
        <div id="captcha-container" class="ps-captcha-container"></div>
    }
}
