//! Store webhook repository traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::RepositoryResult;

/// Store webhook configuration for invoice notifications.
#[derive(Debug, Clone)]
pub struct StoreWebhook {
    pub id: Uuid,
    pub store_id: Uuid,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Read operations for store webhooks.
#[async_trait]
pub trait StoreWebhookReader: Send + Sync {
    /// Get enabled webhook configuration for a store.
    /// Returns None if webhook is not configured or is disabled.
    async fn get_enabled_webhook(&self, store_id: Uuid) -> RepositoryResult<Option<StoreWebhook>>;
}

/// Write operations for store webhooks.
#[async_trait]
pub trait StoreWebhookWriter: Send + Sync {
    /// Create or update webhook configuration for a store.
    async fn upsert_webhook(
        &self,
        store_id: Uuid,
        webhook_url: &str,
        webhook_secret: &str,
        enabled: bool,
    ) -> RepositoryResult<StoreWebhook>;

    /// Delete webhook configuration for a store.
    async fn delete_webhook(&self, store_id: Uuid) -> RepositoryResult<bool>;
}

/// Combined store webhook repository.
pub trait StoreWebhookRepository: StoreWebhookReader + StoreWebhookWriter {}

impl<T: StoreWebhookReader + StoreWebhookWriter> StoreWebhookRepository for T {}
