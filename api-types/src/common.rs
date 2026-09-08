//! Shapes shared across several endpoints.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// A page of results.
///
/// The list endpoints each return their own named wrapper (`InvoiceListResponse`
/// and friends) because that is what they have always returned and renaming
/// them would break every integrator. This generic exists for the client, which
/// wants one shape to render a table from.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
