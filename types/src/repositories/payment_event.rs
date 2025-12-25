//! Payment event repository traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::InvoiceId;

/// Write operations for payment events (audit logging).
#[async_trait]
pub trait PaymentEventWriter: Send + Sync {
    /// Create a payment event for audit logging.
    async fn create_event(
        &self,
        invoice_id: &InvoiceId,
        payment_id: Option<Uuid>,
        event_type: &str,
        event_data: Option<serde_json::Value>,
    ) -> RepositoryResult<Uuid>;
}
