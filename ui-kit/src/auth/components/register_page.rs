//! Register page component.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use std::sync::Arc;

use crate::auth::{
    components::{
        PasskeyAuthForm, PasskeyState, RecoverySetup, TurnstileWidget, WalletConnectButton,
    },
    session::get_device_name,
    types::{
        CaptchaConfigResponse, CompleteNewUserPasskeyRegistrationRequest,
        CompleteNewUserWalletRegistrationRequest, DeviceType, EncryptedBlob, KdfParams,
    },
    wallet::sign_message,
    webauthn::create_credential,
};
use crate::hooks::use_api::ApiClient;
use crate::hooks::use_auth::use_auth;

/// Tab selection for registration method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterTab {
    Wallet,
    Passkey,
}

/// Registration step.
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterStep {
    /// Choose method and connect.
    Connect,
    /// Recovery phrase setup (optional).
    Recovery,
    /// Registration complete.
    Complete,
}

/// Registration state for tracking the flow.
///
/// Deliberately NOT `Debug`: `mnemonic_words` holds the account's real recovery
/// phrase for the lifetime of the page, and a derived `Debug` means any
/// `log!("{:?}", state)` added later prints it to the console (RCS-193).
#[derive(Clone, Default)]
struct RegistrationState {
    wallet_address: Option<String>,
    user_id: Option<crate::auth::types::UserId>,
    signature: Option<String>,
    passkey_credential: Option<serde_json::Value>,
    mnemonic_words: Vec<String>,
}

/// Register page component.
#[component]
pub fn RegisterPage(
    #[prop(optional)] api_url: Option<String>,
    #[prop(optional, default = "/".to_string())] redirect_to: String,
    #[prop(optional, default = "/login".to_string())] login_url: String,
    /// Whether the recovery step must be completed to finish registration.
    ///
    /// Defaults to `true`: skipping binds the account to a phrase the user
    /// explicitly declined to record, and the phrase is the account recovery
    /// mechanism - the thing `/auth/recovery/start` redeems - not merely a data
    /// key. There is no password to fall back on, so a merchant who skips has
    /// no route back in once they lose their passkey or wallet (RCS-214).
    ///
    /// Recovery redemption has no UI yet (RCS-205), so nobody can redeem the
    /// phrase today either way. That is exactly why the default matters now:
    /// every account registered before that ships is enrolled either with a
    /// recorded phrase or without one, and the ones without cannot be fixed
    /// retroactively.
    #[prop(optional, default = true)]
    require_recovery: bool,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(RegisterTab::Wallet);
    let (step, set_step) = signal(RegisterStep::Connect);
    let (error, set_error) = signal::<Option<String>>(None);
    let (loading, set_loading) = signal(false);
    let (passkey_state, set_passkey_state) = signal(PasskeyState::Ready);
    let (reg_state, set_reg_state) = signal(RegistrationState::default());
    let (captcha_config, set_captcha_config) = signal::<Option<CaptchaConfigResponse>>(None);
    let (captcha_token, set_captcha_token) = signal::<Option<String>>(None);

    let auth = use_auth();
    let api = StoredValue::new(ApiClient::new(
        api_url.unwrap_or_else(|| "/api".to_string()),
    ));
    let navigate = use_navigate();
    let redirect = StoredValue::new(redirect_to.clone());

    // Fetch CAPTCHA configuration from the server
    {
        let api = api.get_value();
        leptos::task::spawn_local(async move {
            if let Ok(config) = api.get_captcha_config().await {
                set_captcha_config.set(Some(config));
            }
        });
    }

    // Bounce a visitor who is ALREADY authenticated off the register page.
    //
    // The `step` guard is the fix for RCS-220, and it is not incidental. On the
    // success path below, `save_login` makes this true while the writes to
    // `step` and `loading` are still queued: signal writes mark subscribers
    // dirty and their effects run on the next tick, not inline. Without the
    // guard this Effect navigated first, disposal tore the component down, and
    // then those two queued view updates ran and read signals that no longer
    // existed - "Tried to access a reactive value that has already been
    // disposed", twice on every registration, one per pending write.
    //
    // Ordering the writes around `save_login` cannot fix that, which is why the
    // earlier attempt at this (#15) changed nothing: the reads that panic are
    // the queued effects, and they flush after this Effect either way.
    //
    // It was also silently eating the success screen. Reaching `Complete` is
    // meant to show it for 1500ms before the Timeout redirects; this Effect
    // navigated away immediately instead, so nobody ever saw it.
    {
        let navigate = navigate.clone();
        Effect::new(move || {
            if auth.is_authenticated() && step.get() != RegisterStep::Complete {
                let url = redirect.get_value();
                navigate(&url, Default::default());
            }
        });
    }

    // Wallet connect handler
    let on_wallet_connect = Callback::new(move |address: String| {
        let api = api.get_value();
        let token = captcha_token.get();
        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match api
                .start_wallet_register(&address, "Primary Wallet", token.as_deref())
                .await
            {
                Ok(response) => match sign_message(&address, &response.challenge_message).await {
                    Ok(signature) => {
                        match generate_recovery_mnemonic() {
                            Ok(mnemonic_words) => {
                                set_reg_state.set(RegistrationState {
                                    wallet_address: Some(response.address),
                                    user_id: Some(response.user_id),
                                    signature: Some(signature),
                                    mnemonic_words,
                                    ..Default::default()
                                });
                                set_step.set(RegisterStep::Recovery);
                            }
                            // Fail closed: never show a recovery screen we
                            // cannot back with real entropy.
                            Err(e) => set_error.set(Some(e)),
                        }
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to sign: {}", e)));
                        set_loading.set(false);
                    }
                },
                Err(e) => {
                    set_error.set(Some(format!("Registration failed: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    // Passkey submit handler - no email required
    let on_passkey_submit = Callback::new(move |_: String| {
        let api = api.get_value();
        let token = captcha_token.get();
        set_passkey_state.set(PasskeyState::Authenticating);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match api.start_passkey_register(token.as_deref()).await {
                Ok(response) => match create_credential(&response.options).await {
                    Ok(credential) => {
                        match generate_recovery_mnemonic() {
                            Ok(mnemonic_words) => {
                                set_reg_state.set(RegistrationState {
                                    user_id: Some(response.user_id),
                                    passkey_credential: Some(credential),
                                    mnemonic_words,
                                    ..Default::default()
                                });
                                set_step.set(RegisterStep::Recovery);
                            }
                            Err(e) => set_error.set(Some(e)),
                        }
                        set_passkey_state.set(PasskeyState::Ready);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to create passkey: {}", e)));
                        set_passkey_state.set(PasskeyState::Error(e.to_string()));
                    }
                },
                Err(e) => {
                    set_error.set(Some(format!("Registration failed: {}", e)));
                    set_passkey_state.set(PasskeyState::Error(e.to_string()));
                }
            }
        });
    });

    // Complete registration helper - wrapped in Arc for sharing
    let do_complete_registration = {
        Arc::new(move || {
            let api = api.get_value();
            // No `navigate` here on purpose: the redirect is a gloo Timeout
            // below, because save_login disposes this component (RCS-220).
            let redirect = redirect.get_value();
            let state = reg_state.get();

            set_loading.set(true);
            set_error.set(None);

            leptos::task::spawn_local(async move {
                // Argon2id below is 64 MiB / t=3 and runs synchronously on the
                // main thread. `spawn_local` schedules on a microtask, so without
                // an explicit macrotask yield the browser never paints between
                // `set_loading(true)` and the KDF — the button keeps reading
                // "Complete Setup" while the tab locks up, for a second on
                // desktop and far longer on a low-end phone. One frame is enough
                // to let the loading state render first.
                gloo_timers::future::TimeoutFuture::new(16).await;

                let (kdf_params, encrypted_key, recovery_hash) =
                    match derive_recovery_crypto(&state) {
                        Ok(v) => v,
                        Err(e) => {
                            set_error.set(Some(e));
                            set_loading.set(false);
                            return;
                        }
                    };

                let result = if let (Some(user_id), Some(address), Some(signature)) = (
                    state.user_id,
                    state.wallet_address.as_ref(),
                    state.signature.as_ref(),
                ) {
                    let request = CompleteNewUserWalletRegistrationRequest {
                        user_id,
                        address: address.clone(),
                        signature: signature.clone(),
                        wallet_name: "Primary Wallet".to_string(),
                        kdf_params,
                        encrypted_symmetric_key: encrypted_key,
                        recovery_verification_hash: recovery_hash,
                        device_name: get_device_name(),
                        device_type: DeviceType::Browser,
                    };
                    api.complete_wallet_register(request).await
                } else if let (Some(user_id), Some(credential)) =
                    (state.user_id, state.passkey_credential.as_ref())
                {
                    // Passkey registration - no email required
                    let request = CompleteNewUserPasskeyRegistrationRequest {
                        user_id,
                        credential: credential.clone(),
                        kdf_params,
                        encrypted_symmetric_key: encrypted_key,
                        recovery_verification_hash: recovery_hash,
                        device_name: get_device_name(),
                        device_type: DeviceType::Browser,
                        passkey_name: "Primary Passkey".to_string(),
                    };
                    api.complete_passkey_register(request).await
                } else {
                    set_error.set(Some("Invalid registration state".to_string()));
                    set_loading.set(false);
                    return;
                };

                match result {
                    Ok(response) => {
                        // `Complete` before `save_login`, and it has to be this
                        // way round: the redirect Effect above reads `step` to
                        // decide whether to bounce, so it must already say
                        // Complete by the time authenticating wakes it.
                        // No `set_loading.set(false)` here, and that omission is
                        // the fix for RCS-220.
                        //
                        // `loading` is handed to RecoverySetup below as
                        // `loading=loading.into()`, so its subscribers live in
                        // that subtree - and moving to `Complete` disposes that
                        // subtree. Writing it here queues an update for signals
                        // that the very next line destroys: the writes land, the
                        // Show swaps the branch away, and then the queued reads
                        // run against a disposed scope. Two subscribers in
                        // RecoverySetup, two panics, every registration:
                        //
                        //   At recovery_setup.rs:54:38, you tried to access a
                        //   reactive value which was defined at
                        //   register_page.rs:378:57, but it has already been
                        //   disposed.
                        //
                        // Nothing needs the write. The Complete branch does not
                        // read `loading`, and this path always ends in a full
                        // page load. Confirmed by isolation against a local
                        // debug build: removed, 0 panics; restored, 2.
                        set_step.set(RegisterStep::Complete);

                        // This redirect is the only one now. A gloo Timeout is a
                        // browser callback rather than a reactive one, so it is
                        // unaffected by the reactive graph, and set_href is a
                        // full navigation - the freshly stored session is read
                        // back from localStorage on load.
                        gloo_timers::callback::Timeout::new(1500, move || {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(&redirect);
                            }
                        })
                        .forget();

                        auth.save_login(&response);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Registration failed: {}", e)));
                        set_loading.set(false);
                    }
                }
            });
        })
    };

    // Recovery handlers
    let complete_fn = do_complete_registration.clone();
    let on_recovery_confirm = Callback::new(move |_| {
        complete_fn();
    });

    let complete_fn = do_complete_registration.clone();
    let on_recovery_skip = Callback::new(move |_| {
        complete_fn();
    });

    view! {
        <div class="ps-auth-page">
            <div class="ps-auth-card">
                <Show
                    when=move || step.get() == RegisterStep::Connect
                    fallback=move || {
                        view! {
                            <Show
                                when=move || step.get() == RegisterStep::Recovery
                                fallback=move || view! {
                                    <div class="ps-auth-header">
                                        <div class="ps-auth-success-icon">
                                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <circle cx="12" cy="12" r="10" />
                                                <path d="m9 12 2 2 4-4" />
                                            </svg>
                                        </div>
                                        <h1 class="ps-auth-title">"Account Created!"</h1>
                                        <p class="ps-auth-subtitle">"Redirecting you to the dashboard..."</p>
                                    </div>
                                }
                            >
                                <div class="ps-auth-header">
                                    <h1 class="ps-auth-title">"Account Recovery"</h1>
                                    <p class="ps-auth-subtitle">"Secure your account with a recovery phrase"</p>
                                </div>

                                {move || error.get().map(|e| view! {
                                    <div class="ps-auth-error"><p>{e}</p></div>
                                })}

                                <div class="ps-auth-content">
                                    <RecoverySetup
                                        // Only for passkey-only accounts: a wallet
                                        // registration already has an identifier the
                                        // merchant knows (their address), so showing an
                                        // account id there is noise. A passkey-only
                                        // account has nothing else (RCS-201/205).
                                        account_id={
                                            let st = reg_state.get();
                                            st.wallet_address
                                                .is_none()
                                                .then(|| st.user_id.map(|id| id.to_string()))
                                                .flatten()
                                        }
                                        mnemonic_words=reg_state.get().mnemonic_words.clone()
                                        on_confirm=on_recovery_confirm
                                        on_skip=on_recovery_skip
                                        allow_skip=!require_recovery
                                        loading=loading.into()
                                    />
                                </div>
                            </Show>
                        }
                    }
                >
                    <div class="ps-auth-header">
                        <h1 class="ps-auth-title">"Create Account"</h1>
                        <p class="ps-auth-subtitle">"Choose how you want to sign in"</p>
                    </div>

                    <div class="ps-auth-tabs">
                        <button
                            type="button"
                            class=move || if active_tab.get() == RegisterTab::Wallet { "ps-auth-tab ps-auth-tab-active" } else { "ps-auth-tab" }
                            on:click=move |_| set_active_tab.set(RegisterTab::Wallet)
                        >
                            "Wallet"
                        </button>
                        <button
                            type="button"
                            class=move || if active_tab.get() == RegisterTab::Passkey { "ps-auth-tab ps-auth-tab-active" } else { "ps-auth-tab" }
                            on:click=move |_| set_active_tab.set(RegisterTab::Passkey)
                        >
                            "Passkey"
                        </button>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div class="ps-auth-error"><p>{e}</p></div>
                    })}

                    <div class="ps-auth-content">
                        <Show
                            when=move || active_tab.get() == RegisterTab::Wallet
                            fallback=move || view! {
                                <div class="ps-auth-passkey-tab">
                                    <p class="ps-auth-tab-description">
                                        "Create a passkey for secure, passwordless sign-in."
                                    </p>
                                    <PasskeyAuthForm
                                        is_registration=true
                                        on_submit=on_passkey_submit
                                        state=passkey_state
                                    />
                                </div>
                            }
                        >
                            <div class="ps-auth-wallet-tab">
                                <p class="ps-auth-tab-description">
                                    "Connect your Ethereum wallet to create an account."
                                </p>
                                <WalletConnectButton
                                    on_connect=on_wallet_connect
                                    button_text="Create Account with Wallet".to_string()
                                />
                                <Show when=move || loading.get()>
                                    <div class="ps-auth-loading">
                                        <span class="ps-spinner"></span>
                                        <span>"Creating account..."</span>
                                    </div>
                                </Show>
                            </div>
                        </Show>
                    </div>

                    // CAPTCHA widget (shown when enabled by server)
                    {move || {
                        captcha_config.get().and_then(|config| {
                            if config.enabled {
                                config.site_key.map(|key| view! {
                                    <div class="ps-auth-captcha">
                                        <TurnstileWidget
                                            site_key=key
                                            on_token=Callback::new(move |token: String| {
                                                set_captcha_token.set(Some(token));
                                            })
                                        />
                                    </div>
                                })
                            } else {
                                None
                            }
                        })
                    }}

                    <div class="ps-auth-footer">
                        <p>
                            "Already have an account? "
                            <a href=login_url.clone() class="ps-auth-link">"Sign in"</a>
                        </p>
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// Generate a real 24-word BIP-39 recovery phrase.
///
/// Entropy comes from `crypto`'s `RecoveryMnemonic::generate`, which on wasm32
/// routes `getrandom` to `crypto.getRandomValues` via the `js` feature. That
/// dependency fails to *compile* for wasm without an explicit entropy source
/// rather than silently degrading to a weak PRNG, so there is no path here that
/// produces predictable words.
///
/// 24 words, not 12: the crate generates 256 bits of entropy and the recovery
/// derivation is built around that.
fn generate_recovery_mnemonic() -> Result<Vec<String>, String> {
    crypto::mnemonic::RecoveryMnemonic::generate()
        .map(|m| m.words().into_iter().map(str::to_string).collect())
        .map_err(|e| format!("Could not generate a recovery phrase: {e}"))
}

/// The identifier the recovery KDF is salted with.
///
/// Delegates to `crypto::SaltIdentity`, which is the single definition shared
/// with the server's `auth::models::User` (RCS-200). This used to reimplement
/// the rule, with a comment warning that the two "must be changed together" -
/// nothing enforced it, and a divergence would have made accounts permanently
/// unrecoverable while failing silently as a wrong-phrase error.
///
/// Registration here has no email path, so only the wallet and passkey variants
/// arise.
fn kdf_salt_identifier(state: &RegistrationState) -> Result<String, String> {
    if let Some(ref wallet) = state.wallet_address {
        Ok(crypto::SaltIdentity::Wallet(wallet.clone()).as_identifier())
    } else if let Some(user_id) = state.user_id {
        Ok(crypto::SaltIdentity::Passkey(user_id.to_string()).as_identifier())
    } else {
        Err("No wallet address or user id to bind the recovery key to".to_string())
    }
}

/// Derive the account's recovery material from the phrase shown to the user.
///
/// Replaces the former `generate_placeholder_crypto`, which returned literal
/// base64 of "placeholder_salt"/"placeholder_ciphertext"/... for every account
/// (RCS-193). Critically, that function also ignored the displayed phrase
/// entirely — the words on screen and the stored crypto were independent
/// placeholders, so even real word generation alone would not have made the
/// phrase able to decrypt anything.
///
/// The shape the server expects (`auth::service::recovery`):
///   recovery_key  = Argon2id(BIP39-seed(phrase), "payserver-recovery:{identifier}")
///   recovery_hash = base64(SHA-256(recovery_key))
///
/// The user's symmetric key is generated fresh and wrapped with the stretched
/// recovery key, so the phrase — and only the phrase — can unwrap it. The phrase
/// is never sent to the server.
fn derive_recovery_crypto(
    state: &RegistrationState,
) -> Result<(KdfParams, EncryptedBlob, String), String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
    use sha2::{Digest, Sha256};

    if state.mnemonic_words.is_empty() {
        return Err("No recovery phrase was generated for this registration".to_string());
    }
    let identifier = kdf_salt_identifier(state)?;
    let phrase = state.mnemonic_words.join(" ");

    let mnemonic = crypto::mnemonic::RecoveryMnemonic::from_phrase(&phrase)
        .map_err(|e| format!("Recovery phrase failed validation: {e}"))?;

    // Bound to the account identifier so the same phrase on another account
    // yields a different key.
    let recovery_key = mnemonic
        .derive_recovery_key(&identifier)
        .map_err(|e| format!("Could not derive the recovery key: {e}"))?;

    // What the server stores and compares against on a recovery attempt. It
    // reveals nothing about the phrase.
    let recovery_hash = B64.encode(Sha256::digest(recovery_key.as_bytes()));

    let stretched = crypto::kdf::stretch_master_key(&recovery_key)
        .map_err(|e| format!("Could not stretch the recovery key: {e}"))?;
    let symmetric_key = crypto::kdf::generate_symmetric_key();
    let wrapped = crypto::symmetric::encrypt_key(&symmetric_key, &stretched)
        .map_err(|e| format!("Could not wrap the account key: {e}"))?;

    // Mirrors the constants inside `derive_recovery_key`. The salt is derived
    // from the identifier rather than random, so recovery can reproduce it from
    // the phrase and the identifier alone — there is nothing else to remember.
    let kdf_params = KdfParams {
        algorithm: "argon2id".to_string(),
        memory_kb: crypto::RECOVERY_MEMORY_KB,
        iterations: crypto::RECOVERY_ITERATIONS,
        parallelism: crypto::RECOVERY_PARALLELISM,
        salt: B64.encode(crypto::recovery_salt_for(&identifier).as_bytes()),
    };

    Ok((
        kdf_params,
        EncryptedBlob {
            ciphertext: B64.encode(&wrapped.ciphertext),
            iv: B64.encode(&wrapped.iv),
            mac: B64.encode(&wrapped.mac),
        },
        recovery_hash,
    ))
}
