//! Store-related types.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Store wallet configuration for payment address derivation.
#[derive(Debug, Clone)]
pub struct StoreWallet {
    pub id: Uuid,
    pub store_id: Uuid,
    pub xpub: String,
    pub derivation_index: i32,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

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
