//! Webhook delivery repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::types::WebhookDelivery;

/// Parameters for creating a webhook delivery record.
pub struct CreateDeliveryParams {
    pub store_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub http_status: Option<i16>,
    pub response_body: Option<String>,
    pub latency_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub attempt_number: i32,
}

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
        params: CreateDeliveryParams,
    ) -> RepositoryResult<WebhookDelivery>;
}

/// Combined webhook delivery repository.
pub trait WebhookDeliveryRepository: WebhookDeliveryReader + WebhookDeliveryWriter {}

impl<T: WebhookDeliveryReader + WebhookDeliveryWriter> WebhookDeliveryRepository for T {}
