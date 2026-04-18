//! Authentication hooks.

use crate::types::{AuthState, User};
use leptos::prelude::*;

#[cfg(feature = "auth")]
use leptos_router::hooks::use_navigate;

/// Authentication context provided by the dashboard aggregator.
#[derive(Clone, Copy)]
pub struct AuthContext {
    /// Current authentication state.
    pub state: ReadSignal<AuthState>,
    /// Update authentication state.
    pub set_state: WriteSignal<AuthState>,
    /// Current API token.
    pub token: ReadSignal<Option<String>>,
    /// Update API token.
    pub set_token: WriteSignal<Option<String>>,
}

impl AuthContext {
    /// Create a new auth context.
    pub fn new() -> Self {
        let (state, set_state) = signal(AuthState::Loading);
        let (token, set_token) = signal(None::<String>);

        Self {
            state,
            set_state,
            token,
            set_token,
        }
    }

    /// Check if user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state.get(), AuthState::Authenticated(_))
    }

    /// Get the current user if authenticated.
    pub fn user(&self) -> Option<User> {
        match self.state.get() {
            AuthState::Authenticated(user) => Some(user),
            _ => None,
        }
    }

    /// Log out the current user.
    pub fn logout(&self) {
        self.set_state.set(AuthState::Anonymous);
        self.set_token.set(None);

        // Clear session from localStorage when auth feature is enabled
        #[cfg(feature = "auth")]
        crate::auth::session::clear_session();
    }

    /// Save login response to context.
    /// This is called after successful login to update the auth state.
    #[cfg(feature = "auth")]
    pub fn save_login(&self, response: &crate::auth::types::LoginResponse) {
        // Convert auth types to ui-kit User type
        // Use email if available, otherwise fall back to wallet address
        let user = User {
            id: response.session_id.to_string(),
            email: response.email.clone(),
            display_name: response
                .email
                .clone()
                .or(response.primary_wallet_address.clone()),
        };

        // Update state
        self.set_state.set(AuthState::Authenticated(user));
        self.set_token.set(Some(response.session_id.to_string()));

        // Save to localStorage
        if let Err(e) = crate::auth::session::save_session(response) {
            web_sys::console::error_1(&format!("Failed to save session: {}", e).into());
        }
    }

    /// Load session from localStorage and update state.
    /// Returns true if a valid session was found.
    /// Note: This only checks localStorage, not the server. Use `validate_session`
    /// for server-side validation.
    #[cfg(feature = "auth")]
    pub fn load_session(&self) -> bool {
        web_sys::console::log_1(&"[AuthContext] load_session called".into());
        if let Some(session) = crate::auth::session::load_session() {
            web_sys::console::log_1(
                &format!("[AuthContext] Found session: {:?}", session.session_id).into(),
            );
            // Create user from stored session
            let user = User {
                id: session.session_id.to_string(),
                email: session.email.clone(),
                display_name: session.email.or(session.wallet_address),
            };

            self.set_state.set(AuthState::Authenticated(user));
            self.set_token.set(Some(session.session_id.to_string()));
            web_sys::console::log_1(&"[AuthContext] State set to Authenticated".into());
            true
        } else {
            web_sys::console::log_1(&"[AuthContext] No session found, setting Anonymous".into());
            self.set_state.set(AuthState::Anonymous);
            false
        }
    }

    /// Validate the current session with the server.
    /// If the session is invalid or expired on the server, clears the local session.
    /// This should be called after `load_session` to ensure the session is still valid.
    #[cfg(feature = "auth")]
    pub async fn validate_session(&self, api: &crate::hooks::use_api::ApiClient) {
        // Only validate if we think we're authenticated
        if !matches!(self.state.get_untracked(), AuthState::Authenticated(_)) {
            web_sys::console::log_1(
                &"[AuthContext] validate_session: Not authenticated, skipping".into(),
            );
            return;
        }

        web_sys::console::log_1(
            &"[AuthContext] validate_session: Calling get_current_user...".into(),
        );
        match api.get_current_user().await {
            Ok(user_info) => {
                web_sys::console::log_1(
                    &format!(
                        "[AuthContext] validate_session: Got user info: {:?}",
                        user_info.id
                    )
                    .into(),
                );
                // Update user info from server (may have changed)
                let user = User {
                    id: user_info.id.to_string(),
                    email: user_info.email.clone(),
                    display_name: user_info.email.or(user_info.primary_wallet_address),
                };
                self.set_state.set(AuthState::Authenticated(user));
            }
            Err(e) => {
                web_sys::console::log_1(
                    &format!("[AuthContext] validate_session: Error - {}", e).into(),
                );
                // Session is invalid on server, clear local session
                self.logout();
            }
        }
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide auth context to children.
///
/// When the `auth` feature is enabled, this will automatically load
/// the session from localStorage on mount and validate it with the server.
#[component]
pub fn AuthProvider(
    children: Children,
    /// API base URL for session validation (default: "/api").
    #[prop(optional)]
    api_url: Option<String>,
) -> impl IntoView {
    let auth = AuthContext::new();
    provide_context(auth);

    // Load session from localStorage on mount and validate with server (when auth feature is enabled)
    #[cfg(feature = "auth")]
    {
        let api_url = api_url.unwrap_or_else(|| "/api".to_string());
        Effect::new(move || {
            web_sys::console::log_1(&"[AuthProvider] Effect running, loading session...".into());
            // First, load from localStorage for instant UI
            let has_session = auth.load_session();
            web_sys::console::log_1(
                &format!(
                    "[AuthProvider] Session loaded: {}, state: {:?}",
                    has_session,
                    auth.state.get_untracked()
                )
                .into(),
            );

            // Then validate with server in background
            if has_session {
                // Get the session token for the API client
                let token = auth.token.get_untracked();
                web_sys::console::log_1(&format!("[AuthProvider] Token: {:?}", token).into());
                let api = crate::hooks::use_api::ApiClient::new(api_url.clone()).with_token(token);
                leptos::task::spawn_local(async move {
                    web_sys::console::log_1(
                        &"[AuthProvider] Validating session with server...".into(),
                    );
                    auth.validate_session(&api).await;
                    web_sys::console::log_1(
                        &format!(
                            "[AuthProvider] After validation, state: {:?}",
                            auth.state.get_untracked()
                        )
                        .into(),
                    );
                });
            }
        });
    }

    // Suppress unused variable warning when auth feature is not enabled
    #[cfg(not(feature = "auth"))]
    let _ = api_url;

    // When auth feature is not enabled, just set state to Anonymous
    #[cfg(not(feature = "auth"))]
    {
        auth.set_state.set(AuthState::Anonymous);
    }

    children()
}

/// Access the auth context.
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("use_auth must be used within AuthProvider")
}

/// Guard component that only renders children if authenticated.
///
/// When the `auth` feature is enabled, this will redirect to the login page
/// if the user is not authenticated.
#[component]
pub fn AuthGuard(
    children: ChildrenFn,
    /// URL to redirect to if not authenticated (default: "/login").
    /// Only used when `auth` feature is enabled.
    #[prop(optional)]
    redirect_to: Option<String>,
) -> impl IntoView {
    let auth = use_auth();

    // Used only when auth feature is enabled
    #[cfg(not(feature = "auth"))]
    let _ = &redirect_to;
    #[cfg(feature = "auth")]
    let redirect_url = redirect_to.unwrap_or_else(|| "/login".to_string());
    #[cfg(not(feature = "auth"))]
    let _ = redirect_to;

    view! {
        {move || match auth.state.get() {
            AuthState::Authenticated(_) => children().into_any(),
            AuthState::Loading => view! {
                <div class="ps-auth-loading">
                    <div class="ps-spinner"></div>
                    <p>"Loading..."</p>
                </div>
            }.into_any(),
            AuthState::Anonymous => {
                // When auth feature is enabled, redirect to login
                #[cfg(feature = "auth")]
                {
                    let navigate = use_navigate();
                    let url = redirect_url.clone();
                    // Use spawn_local to avoid calling navigate during render
                    leptos::task::spawn_local(async move {
                        navigate(&url, Default::default());
                    });
                }

                view! {
                    <div class="ps-auth-required">
                        <p>"Redirecting to login..."</p>
                    </div>
                }.into_any()
            }
        }}
    }
}

/// Admin guard component that only renders children if user is an admin.
///
/// This is similar to AuthGuard but also checks for admin role.
#[component]
pub fn AdminGuard(
    children: ChildrenFn,
    /// URL to redirect to if not authenticated (default: "/login").
    #[prop(optional)]
    redirect_to: Option<String>,
    /// URL to redirect to if authenticated but not admin (default: "/").
    #[prop(optional)]
    forbidden_redirect: Option<String>,
) -> impl IntoView {
    let auth = use_auth();

    // Used only when auth feature is enabled
    #[cfg(not(feature = "auth"))]
    let _ = (&redirect_to, &forbidden_redirect);
    #[cfg(feature = "auth")]
    let login_url = redirect_to.unwrap_or_else(|| "/login".to_string());
    // Note: forbidden_url would be used for admin role checking (TODO)
    #[cfg(feature = "auth")]
    let _forbidden_url = forbidden_redirect.unwrap_or_else(|| "/".to_string());
    #[cfg(not(feature = "auth"))]
    let _ = (redirect_to, forbidden_redirect);

    view! {
        {move || match auth.state.get() {
            AuthState::Authenticated(ref _user) => {
                // TODO: Check if user has admin role
                // For now, just render children
                children().into_any()
            }
            AuthState::Loading => view! {
                <div class="ps-auth-loading">
                    <div class="ps-spinner"></div>
                    <p>"Loading..."</p>
                </div>
            }.into_any(),
            AuthState::Anonymous => {
                #[cfg(feature = "auth")]
                {
                    let navigate = use_navigate();
                    let url = login_url.clone();
                    leptos::task::spawn_local(async move {
                        navigate(&url, Default::default());
                    });
                }

                view! {
                    <div class="ps-auth-required">
                        <p>"Redirecting to login..."</p>
                    </div>
                }.into_any()
            }
        }}
    }
}
