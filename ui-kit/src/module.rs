//! Frontend module interface.
//!
//! Each payment server frontend implements these traits to integrate
//! with the dashboard aggregator and checkout page.

use crate::types::RouteInfo;
use wasm_bindgen::prelude::*;

/// Events sent from modules to the aggregator.
#[derive(Debug, Clone)]
pub enum ModuleEvent {
    /// Request to show a notification.
    ShowNotification(crate::types::Notification),
    /// Request to update the sidebar badge count.
    UpdateBadge { count: u32 },
    /// Request to navigate to a route.
    NavigateTo(String),
}

/// Events sent from the aggregator to modules.
#[derive(Debug, Clone)]
pub enum AggregatorEvent {
    /// User authentication state changed.
    UserChanged(Option<crate::types::User>),
    /// Theme changed.
    ThemeChanged(crate::types::Theme),
    /// API token for authenticated requests.
    TokenUpdated(Option<String>),
}

/// Module metadata for display in the dashboard.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Unique identifier (e.g., "evm", "btc", "sol").
    pub id: &'static str,
    /// Display name (e.g., "Ethereum", "Bitcoin").
    pub name: &'static str,
    /// Icon identifier or SVG.
    pub icon: &'static str,
    /// API base URL.
    pub api_url: String,
    /// Available routes.
    pub routes: Vec<RouteInfo>,
}

/// Registry for loaded frontend modules.
#[derive(Default)]
pub struct ModuleRegistry {
    modules: Vec<ModuleInfo>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, module: ModuleInfo) {
        self.modules.push(module);
    }

    pub fn get(&self, id: &str) -> Option<&ModuleInfo> {
        self.modules.iter().find(|m| m.id == id)
    }

    pub fn all(&self) -> impl Iterator<Item = &ModuleInfo> {
        self.modules.iter()
    }
}

/// Checkout slot definitions.
///
/// Each server implements these slots for the checkout page.
pub mod checkout_slots {
    use leptos::prelude::*;

    /// Render a network badge.
    pub type NetworkBadgeFn = fn(chain_id: u64, network_name: &str) -> AnyView;

    /// Render amount details (gas, fees, etc.).
    pub type AmountDetailsFn = fn(chain_id: u64, amount: &str) -> Option<AnyView>;

    /// Render a QR code for payment.
    pub type QrCodeFn = fn(payment_request: &str) -> AnyView;

    /// Render wallet action buttons.
    pub type WalletActionsFn = fn(payment_address: &str, chain_id: u64) -> Option<AnyView>;
}

/// Configuration for a checkout plugin.
#[derive(Clone)]
pub struct CheckoutPluginConfig {
    /// Module identifier this plugin is for.
    pub module_id: &'static str,
    /// Render network badge.
    pub network_badge: checkout_slots::NetworkBadgeFn,
    /// Render amount details.
    pub amount_details: Option<checkout_slots::AmountDetailsFn>,
    /// Render QR code.
    pub qr_code: checkout_slots::QrCodeFn,
    /// Render wallet actions.
    pub wallet_actions: Option<checkout_slots::WalletActionsFn>,
}

/// Registry for checkout plugins.
#[derive(Default)]
pub struct CheckoutPluginRegistry {
    plugins: Vec<CheckoutPluginConfig>,
}

impl CheckoutPluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: CheckoutPluginConfig) {
        self.plugins.push(plugin);
    }

    pub fn get(&self, module_id: &str) -> Option<&CheckoutPluginConfig> {
        self.plugins.iter().find(|p| p.module_id == module_id)
    }
}

/// JavaScript interface for module loading.
#[wasm_bindgen]
extern "C" {
    /// Log to console.
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Initialize the module system.
/// Only enabled when `module-init` feature is active (for standalone builds).
#[cfg(feature = "module-init")]
#[wasm_bindgen(start)]
pub fn init_module_system() {
    console_error_panic_hook::set_once();
    log("PayServer module system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_module(id: &'static str, name: &'static str) -> ModuleInfo {
        ModuleInfo {
            id,
            name,
            icon: "icon",
            api_url: format!("http://localhost/{}", id),
            routes: vec![
                RouteInfo {
                    path: format!("/{}", id),
                    label: name.to_string(),
                    icon: None,
                },
            ],
        }
    }

    #[test]
    fn test_module_registry_new() {
        let registry = ModuleRegistry::new();
        assert_eq!(registry.all().count(), 0);
    }

    #[test]
    fn test_module_registry_default() {
        let registry = ModuleRegistry::default();
        assert_eq!(registry.all().count(), 0);
    }

    #[test]
    fn test_module_registry_register() {
        let mut registry = ModuleRegistry::new();

        registry.register(create_test_module("evm", "Ethereum"));
        assert_eq!(registry.all().count(), 1);

        registry.register(create_test_module("btc", "Bitcoin"));
        assert_eq!(registry.all().count(), 2);
    }

    #[test]
    fn test_module_registry_get() {
        let mut registry = ModuleRegistry::new();
        registry.register(create_test_module("evm", "Ethereum"));
        registry.register(create_test_module("btc", "Bitcoin"));

        let evm = registry.get("evm");
        assert!(evm.is_some());
        assert_eq!(evm.unwrap().name, "Ethereum");

        let btc = registry.get("btc");
        assert!(btc.is_some());
        assert_eq!(btc.unwrap().name, "Bitcoin");

        let sol = registry.get("sol");
        assert!(sol.is_none());
    }

    #[test]
    fn test_module_registry_all() {
        let mut registry = ModuleRegistry::new();
        registry.register(create_test_module("evm", "Ethereum"));
        registry.register(create_test_module("btc", "Bitcoin"));
        registry.register(create_test_module("sol", "Solana"));

        let all: Vec<_> = registry.all().collect();
        assert_eq!(all.len(), 3);

        let ids: Vec<_> = all.iter().map(|m| m.id).collect();
        assert!(ids.contains(&"evm"));
        assert!(ids.contains(&"btc"));
        assert!(ids.contains(&"sol"));
    }

    #[test]
    fn test_module_info_clone() {
        let module = create_test_module("evm", "Ethereum");
        let cloned = module.clone();

        assert_eq!(cloned.id, module.id);
        assert_eq!(cloned.name, module.name);
        assert_eq!(cloned.api_url, module.api_url);
        assert_eq!(cloned.routes.len(), module.routes.len());
    }

    #[test]
    fn test_checkout_plugin_registry_new() {
        let registry = CheckoutPluginRegistry::new();
        assert!(registry.get("evm").is_none());
    }

    #[test]
    fn test_module_event_variants() {
        use crate::types::{Notification, NotificationLevel};

        let notification = Notification {
            id: "test".to_string(),
            level: NotificationLevel::Info,
            title: "Test".to_string(),
            message: None,
            dismissible: true,
            duration_ms: Some(5000),
        };

        let event1 = ModuleEvent::ShowNotification(notification);
        let event2 = ModuleEvent::UpdateBadge { count: 5 };
        let event3 = ModuleEvent::NavigateTo("/dashboard".to_string());

        // Just ensure they can be created and cloned
        let _ = event1.clone();
        let _ = event2.clone();
        let _ = event3.clone();
    }

    #[test]
    fn test_aggregator_event_variants() {
        use crate::types::{Theme, User};

        let user = User {
            id: "123".to_string(),
            email: None,
            display_name: None,
        };

        let event1 = AggregatorEvent::UserChanged(Some(user));
        let event2 = AggregatorEvent::UserChanged(None);
        let event3 = AggregatorEvent::ThemeChanged(Theme::Dark);
        let event4 = AggregatorEvent::TokenUpdated(Some("token".to_string()));

        // Just ensure they can be created and cloned
        let _ = event1.clone();
        let _ = event2.clone();
        let _ = event3.clone();
        let _ = event4.clone();
    }
}
