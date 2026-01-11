//! UI Kit for random.cash frontends.
//!
//! This crate provides shared Leptos components used across all payment server frontends:
//! - Base components (buttons, forms, cards, etc.)
//! - Crypto-specific components (QR codes, addresses, amounts)
//! - Hooks for common functionality (auth, API, storage)
//! - Types for frontend module interface
//!
//! ## Features
//!
//! - `auth` - Enable authentication components (login, register, wallet connect, passkey)

pub mod components;
pub mod hooks;
pub mod module;
pub mod theme;
pub mod types;

// Auth module (optional feature)
#[cfg(feature = "auth")]
pub mod auth;

pub use components::*;
pub use hooks::*;
pub use module::*;
pub use types::*;

#[cfg(feature = "auth")]
pub use auth::{
    LoginPage, PasskeyAuthForm, PasskeyState, RecoverySetup, RegisterPage, WalletConnectButton,
};
#[cfg(feature = "auth")]
pub use auth::wallet::WalletError;
