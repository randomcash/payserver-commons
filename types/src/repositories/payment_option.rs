//! Repository traits for payment options.

use async_trait::async_trait;

use super::RepositoryResult;
use crate::InvoiceId;
use crate::types::{PaymentMethodId, PaymentOptionData, PaymentOptionId};

/// Read operations for payment options.
#[async_trait]
pub trait PaymentOptionReader: Send + Sync {
    /// Get a payment option by ID.
    async fn get(&self, id: &PaymentOptionId) -> RepositoryResult<Option<PaymentOptionData>>;

    /// Get all payment options for an invoice.
    async fn get_for_invoice(
        &self,
        invoice_id: &InvoiceId,
    ) -> RepositoryResult<Vec<PaymentOptionData>>;

    /// Get a payment option by invoice and payment method.
    async fn get_by_payment_method(
        &self,
        invoice_id: &InvoiceId,
        payment_method_id: &PaymentMethodId,
    ) -> RepositoryResult<Option<PaymentOptionData>>;

    /// Get active payment options for an invoice.
    async fn get_active_for_invoice(
        &self,
        invoice_id: &InvoiceId,
    ) -> RepositoryResult<Vec<PaymentOptionData>>;

    /// Find a payment option by payment address and chain.
    async fn get_by_address(
        &self,
        address: &str,
        chain_id: u64,
        token_address: Option<&str>,
    ) -> RepositoryResult<Option<PaymentOptionData>>;
}

/// Write operations for payment options.
#[async_trait]
pub trait PaymentOptionWriter: Send + Sync {
    /// Create a new payment option.
    async fn create(&self, option: &PaymentOptionData) -> RepositoryResult<()>;

    /// Update a payment option.
    async fn update(&self, option: &PaymentOptionData) -> RepositoryResult<()>;

    /// Deactivate a payment option.
    async fn deactivate(&self, id: &PaymentOptionId) -> RepositoryResult<bool>;

    /// Deactivate all payment options for an invoice.
    async fn deactivate_for_invoice(&self, invoice_id: &InvoiceId) -> RepositoryResult<u64>;
}

/// Combined read/write access for payment options.
pub trait PaymentOptionRepository: PaymentOptionReader + PaymentOptionWriter {}

/// Blanket implementation.
impl<T> PaymentOptionRepository for T where T: PaymentOptionReader + PaymentOptionWriter {}
