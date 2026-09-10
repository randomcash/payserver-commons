//! Invoices and their payment options.

use serde::{Deserialize, Serialize};

use crate::payment::PaymentResponse;
use types::{ChainId, InvoiceStatus, PaymentOptionData};
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Invoice response (network-agnostic).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    /// Invoice ID.
    pub id: String,
    /// Store this invoice belongs to.
    pub store_id: String,
    /// Store name, when the caller needs to tell stores apart.
    ///
    /// Only the list endpoints resolve this: a single-invoice response is
    /// always read in a context that already knows the store, and looking the
    /// name up there would be a query per request for a field nobody renders.
    pub store_name: Option<String>,
    /// Invoice currency (e.g., "USD", "EUR", "ETH").
    pub currency: String,
    /// Status.
    /// Current status.
    ///
    /// The enum rather than a `String`: `Display` and the serde form are the
    /// same snake_case words, so the wire shape is unchanged, but the client
    /// can now match on it instead of comparing strings - which is what it was
    /// already doing against its own copy of this struct.
    pub status: InvoiceStatus,
    /// Requested amount in the invoice currency.
    pub amount: String,
    /// Amount received so far (in invoice currency terms).
    #[serde(default = "zero_amount")]
    pub amount_received: String,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Metadata.
    pub metadata: Option<serde_json::Value>,
    /// Customer email for payment receipt (if set).
    pub customer_email: Option<String>,
    /// Payment options for this invoice.
    #[serde(default)]
    pub payment_options: Vec<PaymentOptionResponse>,
}

/// Payment option response for an invoice.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOptionResponse {
    /// Payment option ID.
    pub id: String,
    /// Payment method ID (e.g., "ETH-1", "USDC-137").
    pub payment_method_id: String,
    /// Chain ID (EIP-155).
    pub chain_id: ChainId,
    /// Asset symbol.
    pub asset_symbol: String,
    /// Token contract address (for ERC20, null for native).
    pub token_address: Option<String>,
    /// Asset decimals.
    pub decimals: u8,
    /// Payment address.
    pub payment_address: String,
    /// Amount in the asset's smallest unit.
    pub amount: String,
    /// Exchange rate at time of creation.
    pub rate: Option<String>,
    /// Whether this option is active.
    pub is_active: bool,
}

/// Paginated invoice list response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListResponse {
    /// Total number of matching invoices.
    pub total: i64,
    /// Invoices in this page.
    pub invoices: Vec<InvoiceResponse>,
}

/// Invoice status response with payment details.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceStatusResponse {
    /// Invoice ID.
    pub id: String,
    /// Current status.
    /// Current status.
    ///
    /// The enum rather than a `String`: `Display` and the serde form are the
    /// same snake_case words, so the wire shape is unchanged, but the client
    /// can now match on it instead of comparing strings - which is what it was
    /// already doing against its own copy of this struct.
    pub status: InvoiceStatus,
    /// Requested amount in invoice currency.
    pub amount: String,
    /// Amount received so far.
    pub amount_received: String,
    /// Invoice currency.
    pub currency: String,
    /// Expiration timestamp.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Number of payments received.
    pub payment_count: usize,
    /// Number of confirmed payments.
    pub confirmed_count: usize,
    /// Whether the invoice is fully paid.
    pub is_paid: bool,
    /// Whether the invoice is expired.
    pub is_expired: bool,
    /// Payment options for this invoice.
    pub payment_options: Vec<PaymentOptionResponse>,
    /// Payments received for this invoice.
    pub payments: Vec<PaymentResponse>,
}

/// Request to create a new invoice.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    /// Store ID this invoice belongs to.
    pub store_id: Uuid,
    /// Invoice currency (e.g., "USD", "ETH", "BTC").
    /// For asset-denominated invoices (testing), use the asset symbol directly.
    pub currency: String,
    /// Amount in the currency's standard unit (e.g., "100.00" for USD, "0.1" for ETH).
    /// For asset-denominated invoices, this is in the asset's smallest unit (wei, satoshi).
    pub amount: String,
    /// Expiration in seconds from now (default: 900 = 15 minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_seconds: Option<u64>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Optional customer email for payment receipt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    /// Optional webhook URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    /// Optional redirect URL after payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
}

impl From<PaymentOptionData> for PaymentOptionResponse {
    fn from(po: PaymentOptionData) -> Self {
        Self {
            id: po.id.0.to_string(),
            payment_method_id: po.payment_method_id.0,
            chain_id: po.chain_id,
            asset_symbol: po.asset_symbol,
            token_address: po.token_address,
            decimals: po.decimals,
            payment_address: po.payment_address,
            amount: po.amount,
            rate: po.rate,
            is_active: po.is_active,
        }
    }
}

/// Nothing received yet.
fn zero_amount() -> String {
    "0".to_string()
}
