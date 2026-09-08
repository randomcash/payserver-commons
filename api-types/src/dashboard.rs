//! Dashboard statistics and analytics.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Dashboard statistics response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    /// Total number of invoices across all user stores.
    pub total_invoices: i64,
    /// Number of pending invoices.
    pub pending_invoices: i64,
    /// Number of paid invoices.
    pub paid_invoices: i64,
    /// Number of expired invoices.
    pub expired_invoices: i64,
    /// Total number of payments received.
    pub total_payments: i64,
    /// Number of stores the user has access to.
    pub total_stores: u32,
}
/// One day of volume for a single asset.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DailyVolume {
    /// UTC calendar day.
    pub date: NaiveDate,
    /// Volume for the day in whole units of the asset (e.g. `"0.75"` ETH).
    pub amount: String,
    /// Payments received that day.
    pub payment_count: i64,
}

/// Volume for one asset over the whole window.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AssetVolume {
    /// Asset symbol as recorded on the payments (e.g. `ETH`).
    pub asset_symbol: String,
    /// Window total in whole units of this asset. Only comparable to other
    /// values of the same `asset_symbol`.
    pub total_amount: String,
    /// Payments received in this asset over the window.
    pub payment_count: i64,
    /// This asset's share of the window's payment *count*, 0.0..=100.0.
    pub share_percent: f64,
    /// One entry per day of the window, ascending, zero-filled.
    pub daily: Vec<DailyVolume>,
}

/// Payment analytics for the authenticated user's stores.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DashboardAnalytics {
    /// Window size actually used.
    pub days: u32,
    /// First day of the window (UTC, inclusive).
    pub start_date: NaiveDate,
    /// Last day of the window (UTC, inclusive).
    pub end_date: NaiveDate,
    /// Payments received across all assets in the window.
    pub total_payments: i64,
    /// Per-asset series, busiest asset first. Empty when nothing was received
    /// — an account with no payments gets `[]`, never a fabricated series.
    pub assets: Vec<AssetVolume>,
}
