//! Core traits for the PayServer ecosystem.
//!
//! These traits define the interface that all PayServer implementations must follow.
//! Each PayServer (ethpayserver, bitcoinpayserver, etc.) implements these traits
//! with their network-specific logic.

use std::future::Future;
use std::pin::Pin;

use crate::error::PayServerResult;
use crate::store::StoreId;
use crate::types::{AssetType, HealthStatus, InvoiceId, InvoiceStatus, Network, PaymentEvent};

/// Configuration for creating an invoice.
///
/// Invoices are network-agnostic. You specify the currency and amount,
/// and the PayServer will create payment options based on the store's
/// configured payment methods.
#[derive(Debug, Clone)]
pub struct CreateInvoiceRequest {
    /// The store this invoice belongs to.
    pub store_id: StoreId,
    /// Invoice currency (e.g., "USD", "EUR", "BTC", "ETH").
    pub currency: String,
    /// Amount in the invoice currency.
    /// For fiat, this is a decimal string (e.g., "100.00").
    /// For crypto, this can be in the smallest unit or human-readable.
    pub amount: String,
    /// Invoice expiration in seconds from now.
    pub expiration_seconds: Option<u64>,
    /// Optional metadata to attach to the invoice.
    pub metadata: Option<serde_json::Value>,
    /// Optional webhook URL for payment notifications.
    pub webhook_url: Option<String>,
    /// Optional redirect URL after successful payment.
    pub redirect_url: Option<String>,
    /// Specific payment method IDs to enable (e.g., ["ETH-1", "USDC-137"]).
    /// If None, all store payment methods are enabled.
    pub payment_methods: Option<Vec<String>>,
}

impl CreateInvoiceRequest {
    /// Create a new invoice request.
    pub fn new(store_id: StoreId, currency: impl Into<String>, amount: impl Into<String>) -> Self {
        Self {
            store_id,
            currency: currency.into(),
            amount: amount.into(),
            expiration_seconds: None,
            metadata: None,
            webhook_url: None,
            redirect_url: None,
            payment_methods: None,
        }
    }

    pub fn with_expiration(mut self, seconds: u64) -> Self {
        self.expiration_seconds = Some(seconds);
        self
    }

    pub fn with_webhook(mut self, url: String) -> Self {
        self.webhook_url = Some(url);
        self
    }

    pub fn with_redirect(mut self, url: String) -> Self {
        self.redirect_url = Some(url);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_payment_methods(mut self, methods: Vec<String>) -> Self {
        self.payment_methods = Some(methods);
        self
    }
}

/// Query parameters for listing invoices.
#[derive(Debug, Clone, Default)]
pub struct InvoiceQuery {
    /// Filter by status.
    pub status: Option<InvoiceStatus>,
    /// Filter by currency.
    pub currency: Option<String>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Generic invoice data returned by PayServers.
///
/// An invoice is network-agnostic: it represents a payment request in a
/// specific currency (which can be fiat like "USD" or crypto like "BTC").
/// The actual payment options (which chains/assets can be used to pay)
/// are stored separately in PaymentOptionData.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InvoiceData {
    pub id: InvoiceId,
    /// The store this invoice belongs to.
    pub store_id: StoreId,
    /// Invoice currency (e.g., "USD", "EUR", "BTC", "ETH").
    /// This is what the merchant prices in.
    pub currency: String,
    pub status: InvoiceStatus,
    /// Amount requested in the invoice currency.
    /// For fiat, this is a decimal string (e.g., "100.00").
    /// For crypto, this is in the smallest unit (e.g., wei for ETH).
    pub amount: String,
    /// Total amount received across all payment options, converted to invoice currency.
    /// This is calculated from payments and their exchange rates.
    pub amount_received: String,
    /// When the invoice was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the invoice expires.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
    /// Optional extra data.
    pub extra: Option<serde_json::Value>,
}

/// Generic payment data returned by PayServers.
///
/// A payment is an actual received transaction. It belongs to an invoice
/// and optionally references the payment option that was used.
///
/// Note: Confirmations are computed dynamically as `current_block - block_number + 1`.
/// The `confirmed_at` timestamp indicates when the payment reached the required
/// confirmation threshold. If `confirmed_at` is None, the payment is still awaiting
/// confirmation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaymentData {
    pub id: uuid::Uuid,
    pub invoice_id: InvoiceId,
    /// The payment option this payment was for (if known).
    pub payment_option_id: Option<uuid::Uuid>,
    /// EIP-155 chain ID where this payment was received.
    pub chain_id: u64,
    /// Asset type (native or ERC20).
    #[serde(default)]
    pub asset_type: AssetType,
    /// Amount received (smallest unit as string).
    pub amount: String,
    /// Asset symbol.
    pub asset_symbol: String,
    /// Token contract address (for ERC20 tokens).
    pub token_address: Option<String>,
    /// Transaction hash.
    pub tx_hash: String,
    /// Block number where the payment was included.
    /// Confirmations can be computed as: current_block - block_number + 1
    pub block_number: Option<u64>,
    /// When the payment was detected.
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// When the payment reached required confirmations (None = awaiting confirmation).
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Sender address (if known).
    pub from_address: Option<String>,
    /// Whether this payment was invalidated by a chain reorganization.
    #[serde(default)]
    pub reorged: bool,
    /// Network-specific extra data.
    pub extra: Option<serde_json::Value>,

    // =========================================================================
    // Payment Aggregation Fields
    //
    // These fields enable aggregating payments across different assets/chains
    // into a single invoice total. Without these, we couldn't compare a payment
    // of 0.05 ETH against a $100 USD invoice.
    // =========================================================================
    /// The payment's value credited toward the invoice total, expressed in the
    /// invoice's currency.
    ///
    /// # Purpose
    ///
    /// Invoices can be denominated in any currency (USD, EUR, BTC, ETH, etc.),
    /// but payments arrive in specific assets (ETH on Ethereum, USDC on Polygon,
    /// etc.). This field converts each payment to the invoice's currency so they
    /// can be summed and compared against the invoice amount.
    ///
    /// # Calculation
    ///
    /// **Cross-currency payments** (e.g., USD invoice paid with ETH):
    /// ```text
    /// credited_amount = (raw_amount / 10^decimals) / rate
    ///
    /// Example: 50000000000000000 wei at rate 0.0005 ETH/USD
    ///   = (50000000000000000 / 10^18) / 0.0005
    ///   = 0.05 / 0.0005
    ///   = 100 USD
    /// ```
    ///
    /// **Same-asset payments** (e.g., ETH invoice paid with ETH):
    /// ```text
    /// credited_amount = raw_amount / 10^decimals
    ///
    /// Example: 1500000000000000000 wei
    ///   = 1500000000000000000 / 10^18
    ///   = 1.5 ETH
    /// ```
    ///
    /// # Important Behavior
    ///
    /// - **None means not counted**: Payments where conversion failed or the
    ///   payment option wasn't found are recorded for audit purposes but do NOT
    ///   count toward the invoice's `amount_received`.
    ///
    /// - **Rate is locked at invoice creation**: The exchange rate used comes
    ///   from `rate_used`, which was captured when the invoice was created.
    ///   Market fluctuations after invoice creation don't affect this value.
    ///
    /// - **Aggregation happens in the database**: A trigger sums all non-null
    ///   `credited_amount` values to update `invoice.amount_received`.
    pub credited_amount: Option<String>,

    /// Exchange rate used to calculate `credited_amount`.
    ///
    /// Format: `1 invoice_currency = rate asset_units`
    ///
    /// For example, if the invoice is in USD and payment is in ETH:
    /// - rate = "0.0005" means 1 USD = 0.0005 ETH
    ///
    /// This rate is captured at invoice creation time and locked in. It's stored
    /// here for auditability - you can verify how `credited_amount` was calculated.
    ///
    /// **None** for same-asset payments (no conversion needed).
    pub rate_used: Option<String>,

    /// When the rate was applied to calculate `credited_amount`.
    ///
    /// This is the timestamp when the payment was detected and converted,
    /// not when the rate was originally fetched (that's stored in the payment option).
    pub rate_applied_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Core trait that all payment servers must implement.
pub trait PayServer: Send + Sync {
    /// Returns the networks this PayServer supports.
    fn supported_networks(&self) -> Vec<Network>;

    /// Create a new invoice.
    fn create_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<InvoiceData>> + Send + '_>>;

    /// Get an invoice by ID.
    fn get_invoice(
        &self,
        id: &InvoiceId,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<InvoiceData>> + Send + '_>>;

    /// Cancel an invoice.
    fn cancel_invoice(
        &self,
        id: &InvoiceId,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<()>> + Send + '_>>;

    /// List invoices matching the query.
    fn list_invoices(
        &self,
        query: InvoiceQuery,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<Vec<InvoiceData>>> + Send + '_>>;

    /// Get all payments for an invoice.
    fn get_payments(
        &self,
        invoice_id: &InvoiceId,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<Vec<PaymentData>>> + Send + '_>>;

    /// Get health status of the payment server.
    fn health(&self) -> Pin<Box<dyn Future<Output = PayServerResult<HealthStatus>> + Send + '_>>;
}

/// Trait for monitoring blocknetwork for payments.
pub trait PaymentMonitor: Send + Sync {
    /// Start monitoring for payments.
    fn start(&self) -> Pin<Box<dyn Future<Output = PayServerResult<()>> + Send + '_>>;

    /// Stop monitoring.
    fn stop(&self) -> Pin<Box<dyn Future<Output = PayServerResult<()>> + Send + '_>>;

    /// Check if the monitor is running.
    fn is_running(&self) -> bool;

    /// Get the current block height being monitored for a network.
    fn current_block_height(
        &self,
        network: Network,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<u64>> + Send + '_>>;
}

/// Trait for subscribing to payment events.
pub trait PaymentEventSubscriber: Send + Sync {
    /// Subscribe to payment events.
    fn subscribe(
        &self,
    ) -> Pin<
        Box<
            dyn Future<Output = PayServerResult<tokio::sync::broadcast::Receiver<PaymentEvent>>>
                + Send
                + '_,
        >,
    >;
}

/// Trait for publishing payment events.
pub trait PaymentEventPublisher: Send + Sync {
    /// Publish a payment event.
    fn publish(
        &self,
        event: PaymentEvent,
    ) -> Pin<Box<dyn Future<Output = PayServerResult<()>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_invoice_request_builder() {
        let store_id = StoreId::new();
        let request = CreateInvoiceRequest::new(store_id, "USD", "100.00")
            .with_expiration(3600)
            .with_webhook("https://example.com/webhook".to_string());

        assert_eq!(request.store_id, store_id);
        assert_eq!(request.currency, "USD");
        assert_eq!(request.amount, "100.00");
        assert_eq!(request.expiration_seconds, Some(3600));
        assert!(request.webhook_url.is_some());
    }

    #[test]
    fn test_create_invoice_request_with_payment_methods() {
        let store_id = StoreId::new();
        let request = CreateInvoiceRequest::new(store_id, "USD", "50.00")
            .with_payment_methods(vec!["ETH-1".to_string(), "USDC-137".to_string()]);

        assert_eq!(
            request.payment_methods,
            Some(vec!["ETH-1".to_string(), "USDC-137".to_string()])
        );
    }

    #[test]
    fn test_invoice_query_default() {
        let query = InvoiceQuery::default();
        assert!(query.status.is_none());
        assert!(query.currency.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }
}
