//! Store settings repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::StoreSettings;

/// Read operations for store settings.
#[async_trait]
pub trait StoreSettingsReader: Send + Sync {
    /// Get settings for a store. Returns None if no custom settings exist.
    async fn get_store_settings(&self, store_id: Uuid) -> RepositoryResult<Option<StoreSettings>>;
}

/// Write operations for store settings.
#[async_trait]
pub trait StoreSettingsWriter: Send + Sync {
    /// Create or update settings for a store (upsert).
    async fn upsert_store_settings(
        &self,
        store_id: Uuid,
        default_chain_id: Option<i64>,
        default_display_currency: Option<&str>,
        logo_url: Option<&str>,
        accent_color: Option<&str>,
        notification_prefs: &serde_json::Value,
    ) -> RepositoryResult<StoreSettings>;
}

/// Combined store settings repository.
pub trait StoreSettingsRepository: StoreSettingsReader + StoreSettingsWriter {}

impl<T: StoreSettingsReader + StoreSettingsWriter> StoreSettingsRepository for T {}
