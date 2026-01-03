//! Authentication hooks.

use crate::types::{AuthState, User};
use leptos::prelude::*;

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
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide auth context to children.
#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let auth = AuthContext::new();
    provide_context(auth);
    children()
}

/// Access the auth context.
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("use_auth must be used within AuthProvider")
}

/// Guard component that only renders children if authenticated.
#[component]
pub fn AuthGuard(
    children: ChildrenFn,
) -> impl IntoView {
    let auth = use_auth();

    view! {
        {move || match auth.state.get() {
            AuthState::Authenticated(_) => children().into_any(),
            AuthState::Loading => view! { <div class="ps-auth-loading">"Loading..."</div> }.into_any(),
            AuthState::Anonymous => view! { <div class="ps-auth-required">"Authentication required"</div> }.into_any(),
        }}
    }
}
