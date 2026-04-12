//! Login page component.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::auth::{
    components::{PasskeyAuthForm, PasskeyState, WalletConnectButton},
    session::get_device_name,
    types::{CompletePasskeyLoginRequest, CompleteWalletLoginRequest, DeviceType},
    wallet::sign_message,
    webauthn::get_credential,
};
use crate::hooks::use_api::ApiClient;
use crate::hooks::use_auth::use_auth;

/// Tab selection for login method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginTab {
    Wallet,
    Passkey,
}

/// Login page component.
///
/// Provides wallet and passkey authentication options in a tabbed interface.
#[component]
pub fn LoginPage(
    /// API base URL.
    #[prop(optional)]
    api_url: Option<String>,
    /// URL to redirect to after successful login.
    #[prop(optional, default = "/".to_string())]
    redirect_to: String,
    /// URL for the register page link.
    #[prop(optional, default = "/register".to_string())]
    register_url: String,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(LoginTab::Wallet);
    let (error, set_error) = signal::<Option<String>>(None);
    let (loading, set_loading) = signal(false);
    let (passkey_state, set_passkey_state) = signal(PasskeyState::Ready);

    let auth = use_auth();
    let api = StoredValue::new(ApiClient::new(
        api_url.unwrap_or_else(|| "/api".to_string()),
    ));
    let navigate = use_navigate();
    let redirect = StoredValue::new(redirect_to.clone());

    // Redirect to dashboard if already authenticated
    {
        let navigate = navigate.clone();
        let redirect = redirect.clone();
        Effect::new(move || {
            let state = auth.state.get();
            web_sys::console::log_1(&format!("[LoginPage] Auth state: {:?}", state).into());
            if matches!(state, crate::types::AuthState::Authenticated(_)) {
                let url = redirect.get_value();
                web_sys::console::log_1(&format!("[LoginPage] Redirecting to: {}", url).into());
                navigate(&url, Default::default());
            }
        });
    }

    // Wallet login flow
    let on_wallet_connect = {
        let navigate = navigate.clone();
        Callback::new(move |address: String| {
            let api = api.get_value();
            let navigate = navigate.clone();
            let redirect = redirect.get_value();

            set_loading.set(true);
            set_error.set(None);

            leptos::task::spawn_local(async move {
                // Step 1: Get challenge from server
                let challenge_response = match api.start_wallet_login(&address).await {
                    Ok(r) => r,
                    Err(e) => {
                        set_error.set(Some(format!("Failed to start login: {}", e)));
                        set_loading.set(false);
                        return;
                    }
                };

                // Step 2: Sign the challenge with wallet
                let signature =
                    match sign_message(&address, &challenge_response.challenge_message).await {
                        Ok(sig) => sig,
                        Err(e) => {
                            set_error.set(Some(format!("Failed to sign message: {}", e)));
                            set_loading.set(false);
                            return;
                        }
                    };

                // Step 3: Complete login with signature
                let device_id = crate::auth::session::get_device_id();
                let complete_request = CompleteWalletLoginRequest {
                    user_id: challenge_response.user_id,
                    address: address.clone(),
                    signature,
                    device_id,
                    device_name: get_device_name(),
                    device_type: DeviceType::Browser,
                };

                match api.complete_wallet_login(complete_request).await {
                    Ok(response) => {
                        // Update AuthContext (saves to localStorage and updates state)
                        auth.save_login(&response);
                        set_loading.set(false);
                        navigate(&redirect, Default::default());
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Login failed: {}", e)));
                        set_loading.set(false);
                    }
                }
            });
        })
    };

    // Passkey login flow - uses discoverable credentials (no email needed)
    let on_passkey_submit = {
        let navigate = navigate.clone();
        Callback::new(move |_: String| {
            let api = api.get_value();
            let navigate = navigate.clone();
            let redirect = redirect.get_value();

            set_passkey_state.set(PasskeyState::Authenticating);
            set_error.set(None);

            leptos::task::spawn_local(async move {
                // Step 1: Get WebAuthn challenge from server (discoverable credentials)
                let challenge_response = match api.start_passkey_login().await {
                    Ok(r) => r,
                    Err(e) => {
                        set_error.set(Some(format!("Failed to start login: {}", e)));
                        set_passkey_state.set(PasskeyState::Error(e.to_string()));
                        return;
                    }
                };

                // Step 2: Get credential from authenticator (shows available passkeys)
                let credential = match get_credential(&challenge_response.options).await {
                    Ok(cred) => cred,
                    Err(e) => {
                        set_error.set(Some(format!("Authentication failed: {}", e)));
                        set_passkey_state.set(PasskeyState::Error(e.to_string()));
                        return;
                    }
                };

                // Step 3: Complete login with credential and challenge_id
                let device_id = crate::auth::session::get_device_id();
                let complete_request = CompletePasskeyLoginRequest {
                    challenge_id: challenge_response.challenge_id,
                    credential,
                    device_id,
                    device_name: get_device_name(),
                    device_type: DeviceType::Browser,
                };

                match api.complete_passkey_login(complete_request).await {
                    Ok(response) => {
                        // Update AuthContext (saves to localStorage and updates state)
                        auth.save_login(&response);
                        set_passkey_state.set(PasskeyState::Success);
                        navigate(&redirect, Default::default());
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Login failed: {}", e)));
                        set_passkey_state.set(PasskeyState::Error(e.to_string()));
                    }
                }
            });
        })
    };

    view! {
        <div class="ps-auth-page">
            <div class="ps-auth-card">
                <div class="ps-auth-header">
                    <h1 class="ps-auth-title">"Sign In"</h1>
                    <p class="ps-auth-subtitle">"Choose your preferred sign-in method"</p>
                </div>

                // Tab switcher
                <div class="ps-auth-tabs">
                    <button
                        type="button"
                        class=move || {
                            if active_tab.get() == LoginTab::Wallet {
                                "ps-auth-tab ps-auth-tab-active"
                            } else {
                                "ps-auth-tab"
                            }
                        }
                        on:click=move |_| set_active_tab.set(LoginTab::Wallet)
                    >
                        "Wallet"
                    </button>
                    <button
                        type="button"
                        class=move || {
                            if active_tab.get() == LoginTab::Passkey {
                                "ps-auth-tab ps-auth-tab-active"
                            } else {
                                "ps-auth-tab"
                            }
                        }
                        on:click=move |_| set_active_tab.set(LoginTab::Passkey)
                    >
                        "Passkey"
                    </button>
                </div>

                // Error display
                {move || error.get().map(|e| view! {
                    <div class="ps-auth-error">
                        <p>{e}</p>
                    </div>
                })}

                // Tab content
                <div class="ps-auth-content">
                    {move || match active_tab.get() {
                        LoginTab::Wallet => view! {
                            <div class="ps-auth-wallet-tab">
                                <p class="ps-auth-tab-description">
                                    "Connect your Ethereum wallet to sign in."
                                </p>
                                <WalletConnectButton
                                    on_connect=on_wallet_connect
                                    button_text="Sign in with Wallet".to_string()
                                />
                                {move || if loading.get() {
                                    view! {
                                        <div class="ps-auth-loading">
                                            <span class="ps-spinner"></span>
                                            <span>"Signing in..."</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <></> }.into_any()
                                }}
                            </div>
                        }.into_any(),

                        LoginTab::Passkey => view! {
                            <div class="ps-auth-passkey-tab">
                                <p class="ps-auth-tab-description">
                                    "Use your passkey to sign in securely."
                                </p>
                                <PasskeyAuthForm
                                    is_registration=false
                                    on_submit=on_passkey_submit
                                    state=passkey_state
                                />
                            </div>
                        }.into_any(),
                    }}
                </div>

                // Register link
                <div class="ps-auth-footer">
                    <p>
                        "Don't have an account? "
                        <a href=register_url.clone() class="ps-auth-link">"Create one"</a>
                    </p>
                </div>
            </div>
        </div>
    }
}
