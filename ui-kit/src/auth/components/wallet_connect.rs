//! Wallet connect button component.

use leptos::prelude::*;

use crate::auth::wallet::{WalletError, connect_wallet, format_address, is_wallet_available};

/// State of the wallet connection.
#[derive(Debug, Clone, PartialEq)]
pub enum WalletState {
    /// No wallet detected.
    NotAvailable,
    /// Wallet available but not connected.
    Disconnected,
    /// Connecting to wallet.
    Connecting,
    /// Connected with address.
    Connected(String),
    /// Connection failed.
    Error(String),
}

/// Wallet connect button component.
///
/// Shows a button to connect an Ethereum wallet (MetaMask, etc.).
/// When connected, shows the connected address.
#[component]
pub fn WalletConnectButton(
    /// Callback when wallet is connected.
    on_connect: Callback<String>,
    /// Callback when connection fails.
    #[prop(optional)]
    on_error: Option<Callback<WalletError>>,
    /// Custom button text (default: "Connect Wallet").
    #[prop(optional)]
    button_text: Option<String>,
    /// Show address when connected (default: true).
    #[prop(optional, default = true)]
    show_address: bool,
) -> impl IntoView {
    let (state, set_state) = signal(if is_wallet_available() {
        WalletState::Disconnected
    } else {
        WalletState::NotAvailable
    });

    let handle_connect = move |_| {
        let on_connect = on_connect.clone();
        let on_error = on_error.clone();

        set_state.set(WalletState::Connecting);

        leptos::task::spawn_local(async move {
            match connect_wallet().await {
                Ok(address) => {
                    set_state.set(WalletState::Connected(address.clone()));
                    on_connect.run(address);
                }
                Err(e) => {
                    set_state.set(WalletState::Error(e.message.clone()));
                    if let Some(on_error) = on_error {
                        on_error.run(e);
                    }
                }
            }
        });
    };

    let button_label = button_text.unwrap_or_else(|| "Connect Wallet".to_string());

    view! {
        <div class="ps-wallet-connect">
            {move || match state.get() {
                WalletState::NotAvailable => view! {
                    <div class="ps-wallet-not-available">
                        <p class="ps-wallet-error">
                            "No Ethereum wallet detected. "
                            <a href="https://metamask.io" target="_blank" rel="noopener">
                                "Install MetaMask"
                            </a>
                        </p>
                    </div>
                }.into_any(),

                WalletState::Disconnected => view! {
                    <button
                        type="button"
                        class="ps-wallet-button"
                        on:click=handle_connect
                    >
                        <WalletIcon />
                        <span>{button_label.clone()}</span>
                    </button>
                }.into_any(),

                WalletState::Connecting => view! {
                    <button
                        type="button"
                        class="ps-wallet-button ps-wallet-button-loading"
                        disabled=true
                    >
                        <span class="ps-spinner"></span>
                        <span>"Connecting..."</span>
                    </button>
                }.into_any(),

                WalletState::Connected(ref address) => {
                    if show_address {
                        let addr = address.clone();
                        view! {
                            <div class="ps-wallet-connected">
                                <WalletIcon />
                                <span class="ps-wallet-address">{format_address(&addr)}</span>
                                <span class="ps-wallet-connected-badge">"Connected"</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="ps-wallet-connected">
                                <span class="ps-wallet-connected-badge">"Wallet Connected"</span>
                            </div>
                        }.into_any()
                    }
                }

                WalletState::Error(ref message) => view! {
                    <div class="ps-wallet-error-container">
                        <p class="ps-wallet-error">{message.clone()}</p>
                        <button
                            type="button"
                            class="ps-wallet-button ps-wallet-button-retry"
                            on:click=handle_connect
                        >
                            "Try Again"
                        </button>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

/// Simple wallet icon component.
#[component]
fn WalletIcon() -> impl IntoView {
    view! {
        <svg
            class="ps-wallet-icon"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M21 12V7H5a2 2 0 0 1 0-4h14v4" />
            <path d="M3 5v14a2 2 0 0 0 2 2h16v-5" />
            <path d="M18 12a2 2 0 0 0 0 4h4v-4Z" />
        </svg>
    }
}
