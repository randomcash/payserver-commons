//! Invoice repository traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::BoxStream;

use super::{RepositoryResult, normalize_search};
use crate::store::StoreId;
use crate::traits::InvoiceData;
use crate::types::{InvoiceId, InvoiceStatus};

/// Query parameters for listing invoices.
#[derive(Debug, Clone, Default)]
pub struct InvoiceQueryParams {
    pub store_id: Option<StoreId>,

    /// Restrict to a set of stores, for a caller listing everything they can
    /// see rather than one store or the whole server.
    ///
    /// An EMPTY vec means "no stores", and must filter every row out. It is not
    /// the same as `None`, which means "do not filter at all" - collapsing the
    /// two would turn a user who belongs to no store into a caller who sees
    /// every store on the server.
    pub store_ids: Option<Vec<StoreId>>,
    pub status: Option<InvoiceStatus>,
    pub currency: Option<String>,

    /// Free-text search over the columns the invoice list shows.
    ///
    /// Read it through [`Self::search_term`] rather than directly: blank means
    /// "no filter", and the backends have to agree on that.
    pub search: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

impl InvoiceQueryParams {
    pub fn new() -> Self {
        Self {
            store_id: None,
            store_ids: None,
            status: None,
            currency: None,
            search: None,
            created_after: None,
            created_before: None,
            limit: 50,
            offset: 0,
        }
    }

    pub fn with_store_id(mut self, store_id: StoreId) -> Self {
        self.store_id = Some(store_id);
        self
    }

    /// See [`Self::store_ids`]. An empty vec is meaningful: it matches nothing.
    pub fn with_store_ids(mut self, store_ids: Vec<StoreId>) -> Self {
        self.store_ids = Some(store_ids);
        self
    }

    pub fn with_status(mut self, status: InvoiceStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    /// Free-text search. A blank term is no filter at all - see
    /// [`Self::search_term`].
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// The search term to actually filter on, trimmed, or `None` when the
    /// caller supplied nothing usable.
    ///
    /// Every backend must filter through this and no other reading of
    /// [`Self::search`], or Postgres and the in-memory double end up
    /// disagreeing about what an empty box means.
    pub fn search_term(&self) -> Option<&str> {
        normalize_search(self.search.as_deref())
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
}

/// Read operations for invoices.
#[async_trait]
pub trait InvoiceReader: Send + Sync {
    /// Get an invoice by ID.
    async fn get(&self, id: &InvoiceId) -> RepositoryResult<Option<InvoiceData>>;

    /// Query invoices with pagination.
    /// Returns (total_count, invoices).
    async fn query(&self, params: &InvoiceQueryParams)
    -> RepositoryResult<(i64, Vec<InvoiceData>)>;

    /// Get all pending invoices that have expired.
    async fn get_expired(&self) -> RepositoryResult<Vec<InvoiceData>>;

    /// Stream expired pending invoice IDs.
    ///
    /// Only returns invoices where:
    /// - status = 'pending' (no payments detected)
    /// - expires_at < NOW()
    ///
    /// Returns a stream of invoice IDs for minimal memory usage.
    /// Callers should look up payment options for each invoice to get
    /// the addresses that need to be unwatched.
    fn stream_expired_pending(&self) -> BoxStream<'_, RepositoryResult<InvoiceId>>;
}

/// Write operations for invoices.
#[async_trait]
pub trait InvoiceWriter: Send + Sync {
    /// Insert or update an invoice.
    async fn upsert(&self, invoice: &InvoiceData) -> RepositoryResult<()>;

    /// Update an invoice's status.
    async fn update_status(&self, id: &InvoiceId, status: InvoiceStatus) -> RepositoryResult<()>;

    /// Update the amount received for an invoice.
    async fn update_amount_received(&self, id: &InvoiceId, amount: &str) -> RepositoryResult<()>;

    /// Expire a single pending invoice.
    ///
    /// Updates status to 'expired'. Returns true if the invoice was expired,
    /// false if it was already in a different status.
    async fn expire(&self, id: &InvoiceId) -> RepositoryResult<bool>;
}

/// Combined invoice repository with full read/write access.
pub trait InvoiceRepository: InvoiceReader + InvoiceWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: InvoiceReader + InvoiceWriter> InvoiceRepository for T {}
