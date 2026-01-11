//! Authentication module for client-side auth flows.
//!
//! This module provides:
//! - Auth API client methods for wallet and passkey authentication
//! - Session persistence (localStorage)
//! - JavaScript bindings for wallet (MetaMask) and WebAuthn
//! - Login/Register page components

pub mod api;
pub mod components;
pub mod session;
pub mod types;
pub mod wallet;
pub mod webauthn;

// Re-export commonly used items
pub use components::*;
pub use session::*;
pub use types::*;
