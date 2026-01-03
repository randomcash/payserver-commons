//! Shared types for frontend modules.

use serde::{Deserialize, Serialize};

/// User information shared across modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// Authentication state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthState {
    /// Not authenticated.
    Anonymous,
    /// Authenticated with user info.
    Authenticated(User),
    /// Loading authentication state.
    Loading,
}

impl Default for AuthState {
    fn default() -> Self {
        Self::Loading
    }
}

/// Theme variants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

/// Notification severity levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A notification to display to the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub title: String,
    pub message: Option<String>,
    pub dismissible: bool,
    pub duration_ms: Option<u32>,
}

impl Notification {
    pub fn info(title: impl Into<String>) -> Self {
        Self {
            id: uuid(),
            level: NotificationLevel::Info,
            title: title.into(),
            message: None,
            dismissible: true,
            duration_ms: Some(5000),
        }
    }

    pub fn success(title: impl Into<String>) -> Self {
        Self {
            id: uuid(),
            level: NotificationLevel::Success,
            title: title.into(),
            message: None,
            dismissible: true,
            duration_ms: Some(5000),
        }
    }

    pub fn warning(title: impl Into<String>) -> Self {
        Self {
            id: uuid(),
            level: NotificationLevel::Warning,
            title: title.into(),
            message: None,
            dismissible: true,
            duration_ms: Some(8000),
        }
    }

    pub fn error(title: impl Into<String>) -> Self {
        Self {
            id: uuid(),
            level: NotificationLevel::Error,
            title: title.into(),
            message: None,
            dismissible: true,
            duration_ms: None, // Errors persist until dismissed
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn persistent(mut self) -> Self {
        self.duration_ms = None;
        self
    }
}

/// Route information for navigation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteInfo {
    pub path: String,
    pub label: String,
    pub icon: Option<String>,
}

/// Invoice data for checkout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceInfo {
    pub id: String,
    pub amount: String,
    pub currency: String,
    pub crypto_amount: Option<String>,
    pub crypto_currency: Option<String>,
    pub payment_address: Option<String>,
    pub payment_request: Option<String>,
    pub status: String,
    pub expires_at: Option<String>,
    pub network: Option<String>,
    pub chain_id: Option<u64>,
}

/// Configuration for a server module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub id: String,
    pub url: String,
    pub api: String,
}

/// Application configuration loaded at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub modules: Vec<ModuleConfig>,
}

/// Generate a simple UUID v4.
fn uuid() -> String {
    use js_sys::Math;
    let random = || (Math::random() * 16.0) as u32;
    format!(
        "{:08x}-{:04x}-4{:03x}-{:x}{:03x}-{:012x}",
        (random() << 24) | (random() << 16) | (random() << 8) | random(),
        (random() << 8) | random(),
        (random() << 8) | random() & 0xfff,
        8 | (random() & 3),
        (random() << 8) | random() & 0xfff,
        ((random() as u64) << 40) | ((random() as u64) << 32) | ((random() as u64) << 24)
            | ((random() as u64) << 16) | ((random() as u64) << 8) | (random() as u64)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state_default() {
        assert_eq!(AuthState::default(), AuthState::Loading);
    }

    #[test]
    fn test_theme_default() {
        assert_eq!(Theme::default(), Theme::Light);
    }

    #[test]
    fn test_user_equality() {
        let user1 = User {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
            display_name: Some("Test User".to_string()),
        };
        let user2 = User {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
            display_name: Some("Test User".to_string()),
        };
        let user3 = User {
            id: "456".to_string(),
            email: None,
            display_name: None,
        };

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }

    #[test]
    fn test_auth_state_variants() {
        let anonymous = AuthState::Anonymous;
        let loading = AuthState::Loading;
        let user = User {
            id: "123".to_string(),
            email: None,
            display_name: None,
        };
        let authenticated = AuthState::Authenticated(user.clone());

        assert_eq!(anonymous, AuthState::Anonymous);
        assert_eq!(loading, AuthState::Loading);
        assert_eq!(authenticated, AuthState::Authenticated(user));
    }

    #[test]
    fn test_notification_builder_methods() {
        // Create a notification manually to test builder methods
        let notification = Notification {
            id: "test-id".to_string(),
            level: NotificationLevel::Info,
            title: "Test".to_string(),
            message: None,
            dismissible: true,
            duration_ms: Some(5000),
        };

        // Test with_message
        let with_msg = notification.clone().with_message("Hello");
        assert_eq!(with_msg.message, Some("Hello".to_string()));

        // Test persistent
        let persistent = notification.clone().persistent();
        assert_eq!(persistent.duration_ms, None);

        // Test chaining
        let chained = Notification {
            id: "test-id".to_string(),
            level: NotificationLevel::Error,
            title: "Error".to_string(),
            message: None,
            dismissible: true,
            duration_ms: Some(3000),
        }
        .with_message("Something went wrong")
        .persistent();

        assert_eq!(chained.message, Some("Something went wrong".to_string()));
        assert_eq!(chained.duration_ms, None);
    }

    #[test]
    fn test_notification_level_variants() {
        assert_eq!(NotificationLevel::Info, NotificationLevel::Info);
        assert_ne!(NotificationLevel::Info, NotificationLevel::Error);
        assert_ne!(NotificationLevel::Warning, NotificationLevel::Success);
    }

    #[test]
    fn test_route_info() {
        let route = RouteInfo {
            path: "/dashboard".to_string(),
            label: "Dashboard".to_string(),
            icon: Some("home".to_string()),
        };

        assert_eq!(route.path, "/dashboard");
        assert_eq!(route.label, "Dashboard");
        assert_eq!(route.icon, Some("home".to_string()));
    }

    #[test]
    fn test_invoice_info() {
        let invoice = InvoiceInfo {
            id: "inv_123".to_string(),
            amount: "100.00".to_string(),
            currency: "USD".to_string(),
            crypto_amount: Some("0.05".to_string()),
            crypto_currency: Some("ETH".to_string()),
            payment_address: Some("0x1234...".to_string()),
            payment_request: None,
            status: "pending".to_string(),
            expires_at: Some("2024-12-31T23:59:59Z".to_string()),
            network: Some("ethereum".to_string()),
            chain_id: Some(1),
        };

        assert_eq!(invoice.id, "inv_123");
        assert_eq!(invoice.status, "pending");
        assert_eq!(invoice.chain_id, Some(1));
    }

    #[test]
    fn test_module_config() {
        let config = ModuleConfig {
            id: "evm".to_string(),
            url: "/modules/evm.wasm".to_string(),
            api: "http://localhost:5000".to_string(),
        };

        assert_eq!(config.id, "evm");
        assert_eq!(config.url, "/modules/evm.wasm");
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.modules.is_empty());
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
            display_name: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        let parsed: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user, parsed);
    }

    #[test]
    fn test_theme_serialization() {
        let themes = [Theme::Light, Theme::Dark, Theme::System];

        for theme in themes {
            let json = serde_json::to_string(&theme).unwrap();
            let parsed: Theme = serde_json::from_str(&json).unwrap();
            assert_eq!(theme, parsed);
        }
    }
}
