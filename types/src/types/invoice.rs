//! Invoice-related types.

use serde::{Deserialize, Serialize};

/// Status of an invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    /// Invoice created, awaiting payment.
    Pending,
    /// Payment detected but not confirmed.
    Processing,
    /// Payment partially received.
    PartiallyPaid,
    /// Payment fully received and confirmed.
    Paid,
    /// Invoice expired without payment.
    Expired,
    /// Invoice cancelled.
    Cancelled,
    /// Payment refunded.
    Refunded,
    /// Payment received after invoice expired.
    LatePaid,
}

impl InvoiceStatus {
    /// Returns true if this is a final status (no more changes expected).
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            InvoiceStatus::Paid
                | InvoiceStatus::Expired
                | InvoiceStatus::Cancelled
                | InvoiceStatus::Refunded
                | InvoiceStatus::LatePaid
        )
    }

    /// Returns true if this invoice can still receive payments.
    pub fn is_payable(&self) -> bool {
        matches!(
            self,
            InvoiceStatus::Pending | InvoiceStatus::Processing | InvoiceStatus::PartiallyPaid
        )
    }
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Processing => "processing",
            InvoiceStatus::PartiallyPaid => "partially_paid",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Expired => "expired",
            InvoiceStatus::Cancelled => "cancelled",
            InvoiceStatus::Refunded => "refunded",
            InvoiceStatus::LatePaid => "late_paid",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for InvoiceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(InvoiceStatus::Pending),
            "processing" => Ok(InvoiceStatus::Processing),
            "partially_paid" => Ok(InvoiceStatus::PartiallyPaid),
            "paid" => Ok(InvoiceStatus::Paid),
            "expired" => Ok(InvoiceStatus::Expired),
            "cancelled" | "canceled" => Ok(InvoiceStatus::Cancelled),
            "refunded" => Ok(InvoiceStatus::Refunded),
            "late_paid" => Ok(InvoiceStatus::LatePaid),
            _ => Err(format!("unknown invoice status: {}", s)),
        }
    }
}

/// Asset type for payments.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// Native network currency (ETH, BTC, POL, etc.)
    #[default]
    Native,
    /// ERC20 token (for EVM networks)
    ERC20,
}

impl AssetType {
    /// Returns the database representation of this asset type.
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Native => "native",
            AssetType::ERC20 => "erc20",
        }
    }
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AssetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "native" => Ok(AssetType::Native),
            "erc20" => Ok(AssetType::ERC20),
            _ => Err(format!("unknown asset type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_status() {
        assert!(InvoiceStatus::Paid.is_final());
        assert!(InvoiceStatus::Expired.is_final());
        assert!(!InvoiceStatus::Pending.is_final());
        assert!(!InvoiceStatus::Processing.is_final());

        assert!(InvoiceStatus::Pending.is_payable());
        assert!(InvoiceStatus::PartiallyPaid.is_payable());
        assert!(!InvoiceStatus::Paid.is_payable());
        assert!(!InvoiceStatus::Expired.is_payable());
    }
}
