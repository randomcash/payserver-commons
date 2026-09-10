//! Repository traits for data persistence.
//!
//! This module defines the core repository traits that abstract database operations.
//! Each domain (invoices, payments, watched addresses) has:
//!
//! - **Reader trait** - Read-only operations (queries, lookups)
//! - **Writer trait** - Write operations (insert, update, delete)
//! - **Repository trait** - Combined read/write access (supertrait)
//!
//! # Example Usage
//!
//! ```ignore
//! // Read-only access for API queries
//! fn list_invoices(reader: &impl InvoiceReader) { ... }
//!
//! // Write access for processing
//! fn create_invoice(writer: &impl InvoiceWriter) { ... }
//!
//! // Full access
//! fn process_payment(repo: &impl InvoiceRepository) { ... }
//! ```
//!
//! # DataService
//!
//! For convenience, [`DataService`] combines all repository traits.

mod error;
mod invoice;
mod live_watched_address;
mod payment;
mod payment_event;
mod payment_option;
mod payout;
mod refund;
mod store_payment_method;
mod store_settings;
mod store_token_policy;
mod store_webhook;
mod token;
mod wallet;
mod watched_address;
mod webhook_delivery;

pub use error::{RepositoryError, RepositoryResult};
pub use invoice::{InvoiceQueryParams, InvoiceReader, InvoiceRepository, InvoiceWriter};
pub use live_watched_address::{
    LiveWatchedAddressReader, LiveWatchedAddressRepository, LiveWatchedAddressWriter,
};
pub use payment::{PaymentQueryParams, PaymentReader, PaymentRepository, PaymentWriter};
pub use payment_event::PaymentEventWriter;
pub use payment_option::{PaymentOptionReader, PaymentOptionRepository, PaymentOptionWriter};
pub use payout::{PayoutReader, PayoutRepository, PayoutWriter};
pub use refund::{RefundReader, RefundRepository, RefundWriter};
pub use store_payment_method::{
    StorePaymentMethodReader, StorePaymentMethodRepository, StorePaymentMethodWriter,
};
pub use store_settings::{StoreSettingsReader, StoreSettingsRepository, StoreSettingsWriter};
pub use store_token_policy::{
    StoreTokenPolicyReader, StoreTokenPolicyRepository, StoreTokenPolicyWriter,
    TokenPolicyEntryInput,
};
pub use store_webhook::{StoreWebhookReader, StoreWebhookRepository, StoreWebhookWriter};
pub use token::{TokenQueryParams, TokenReader, TokenRepository, TokenWriter};
pub use wallet::{WalletReader, WalletRepository, WalletWriter};
pub use watched_address::{WatchedAddressReader, WatchedAddressRepository, WatchedAddressWriter};
pub use webhook_delivery::{
    CreateDeliveryParams, WebhookDeliveryReader, WebhookDeliveryRepository, WebhookDeliveryWriter,
};

/// Normalize a free-text list search term.
///
/// Trims, and treats a blank term as absent. The distinction matters: an empty
/// search box must mean "no filter", never "match nothing" - a user who clears
/// the box, or whose client sends `?search=`, is asking to see everything
/// again, not to be shown an empty list.
///
/// Both `InvoiceQueryParams::search_term` and `PaymentQueryParams::search_term`
/// go through here, and so does every backend that reads the field, so the rule
/// cannot drift between the SQL store and the in-memory double, which have
/// silently disagreed before.
pub fn normalize_search(search: Option<&str>) -> Option<&str> {
    search.map(str::trim).filter(|term| !term.is_empty())
}

/// Combined data service trait with full read/write access to all repositories.
///
/// This supertrait combines all repository traits for convenience when you need
/// access to all data operations. For more focused dependencies, use the
/// individual Reader/Writer traits.
pub trait DataService:
    InvoiceRepository
    + PaymentRepository
    + PaymentOptionRepository
    + WatchedAddressRepository
    + TokenRepository
{
}

/// Blanket implementation: any type implementing all repository traits is a DataService.
impl<T> DataService for T where
    T: InvoiceRepository
        + PaymentRepository
        + PaymentOptionRepository
        + WatchedAddressRepository
        + TokenRepository
{
}

/// Read-only data service trait.
///
/// Use this when you only need read access to all repositories.
pub trait DataServiceReader:
    InvoiceReader + PaymentReader + PaymentOptionReader + WatchedAddressReader + TokenReader
{
}

/// Blanket implementation for read-only access.
impl<T> DataServiceReader for T where
    T: InvoiceReader + PaymentReader + PaymentOptionReader + WatchedAddressReader + TokenReader
{
}

/// Write-only data service trait.
///
/// Use this when you only need write access to all repositories.
pub trait DataServiceWriter:
    InvoiceWriter + PaymentWriter + PaymentOptionWriter + WatchedAddressWriter + TokenWriter
{
}

/// Blanket implementation for write-only access.
impl<T> DataServiceWriter for T where
    T: InvoiceWriter + PaymentWriter + PaymentOptionWriter + WatchedAddressWriter + TokenWriter
{
}

#[cfg(test)]
mod tests {
    use super::{InvoiceQueryParams, PaymentQueryParams, normalize_search};

    #[test]
    fn blank_search_is_no_filter_not_an_impossible_one() {
        assert_eq!(normalize_search(None), None);
        assert_eq!(normalize_search(Some("")), None);
        assert_eq!(normalize_search(Some("   ")), None);
        assert_eq!(normalize_search(Some("\t\n")), None);
    }

    #[test]
    fn search_is_trimmed_so_a_pasted_hash_still_matches() {
        assert_eq!(normalize_search(Some("  0xdead ")), Some("0xdead"));
        assert_eq!(normalize_search(Some("usd")), Some("usd"));
    }

    #[test]
    fn both_query_params_read_the_field_through_the_same_rule() {
        assert_eq!(
            InvoiceQueryParams::new()
                .with_search("  USD ")
                .search_term(),
            Some("USD")
        );
        assert_eq!(
            InvoiceQueryParams::new().with_search("  ").search_term(),
            None
        );
        assert_eq!(
            PaymentQueryParams::new()
                .with_search(" 0xabc ")
                .search_term(),
            Some("0xabc")
        );
        assert_eq!(
            PaymentQueryParams::new().with_search("").search_term(),
            None
        );
    }
}
