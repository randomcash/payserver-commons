//! Event types.

use serde::{Deserialize, Serialize};

use super::{InvoiceId, InvoiceStatus, Network};

/// Health status of a PayServer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the service is healthy.
    pub healthy: bool,
    /// Service version.
    pub version: String,
    /// Networks this server supports.
    pub supported_networks: Vec<Network>,
    /// Current block heights per network (if applicable).
    pub block_heights: Option<std::collections::HashMap<Network, u64>>,
    /// Number of pending invoices.
    pub pending_invoices: Option<u64>,
    /// Additional details.
    pub details: Option<serde_json::Value>,
}

/// Events emitted by the payment system.
///
/// These are network-agnostic events. PayServers may emit additional
/// network-specific events internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PaymentEvent {
    /// Invoice was created.
    InvoiceCreated {
        invoice_id: InvoiceId,
        network: Network,
    },
    /// Payment was detected (unconfirmed).
    PaymentDetected {
        invoice_id: InvoiceId,
        tx_hash: String,
        network: Network,
    },
    /// Payment was confirmed.
    PaymentConfirmed {
        invoice_id: InvoiceId,
        tx_hash: String,
        confirmations: u32,
    },
    /// Invoice status changed.
    InvoiceStatusChanged {
        invoice_id: InvoiceId,
        old_status: InvoiceStatus,
        new_status: InvoiceStatus,
    },
    /// Invoice was fully paid.
    InvoicePaid { invoice_id: InvoiceId },
    /// Invoice expired.
    InvoiceExpired { invoice_id: InvoiceId },
}
