//! Webhook delivery repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::WebhookDelivery;

/// Read operations for webhook deliveries.
#[async_trait]
pub trait WebhookDeliveryReader: Send + Sync {
    /// List deliveries for a store, ordered by created_at desc.
    async fn list_deliveries(
        &self,
        store_id: Uuid,
        event_type: Option<&str>,
        success: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<(i64, Vec<WebhookDelivery>)>;

    /// Get a single delivery by ID.
    async fn get_delivery(&self, delivery_id: Uuid) -> RepositoryResult<Option<WebhookDelivery>>;
}

/// Write operations for webhook deliveries.
#[async_trait]
pub trait WebhookDeliveryWriter: Send + Sync {
    /// Record a webhook delivery attempt.
    async fn create_delivery(
        &self,
        store_id: Uuid,
        event_type: &str,
        payload: serde_json::Value,
        http_status: Option<i16>,
        response_body: Option<&str>,
        latency_ms: i32,
        success: bool,
        error_message: Option<&str>,
        attempt_number: i32,
    ) -> RepositoryResult<WebhookDelivery>;
}

/// Combined webhook delivery repository.
pub trait WebhookDeliveryRepository: WebhookDeliveryReader + WebhookDeliveryWriter {}

impl<T: WebhookDeliveryReader + WebhookDeliveryWriter> WebhookDeliveryRepository for T {}
