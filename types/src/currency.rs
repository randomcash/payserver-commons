//! Currency constants and invoice defaults.
//!
//! Canonical source of truth for supported invoice currencies and
//! expiration presets. Used by both server and client (WASM-compatible).

/// Common fiat currency codes (ISO 4217) supported for invoice denomination.
pub const FIAT_CURRENCIES: &[&str] = &[
    // Major currencies
    "USD", "EUR", "GBP", "JPY", "CHF", "CNY",
    // Americas
    "CAD", "MXN", "BRL", "ARS", "CLP", "COP", "PEN",
    // Asia-Pacific
    "AUD", "NZD", "SGD", "HKD", "KRW", "INR", "IDR", "THB", "VND", "PHP", "MYR", "TWD",
    // Europe
    "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RON", "BGN", "HRK", "RUB", "UAH", "TRY",
    // Middle East & Africa
    "ILS", "AED", "SAR", "ZAR", "EGP", "NGN", "KES",
];

/// Crypto currencies supported for invoice denomination.
pub const CRYPTO_CURRENCIES: &[&str] = &["ETH", "BTC", "USDC", "USDT", "DAI"];

/// Invoice currency options for UI display: `(code, label)`.
///
/// Subset of commonly used currencies for the create-invoice form.
pub const INVOICE_CURRENCY_OPTIONS: &[(&str, &str)] = &[
    ("USD", "USD - US Dollar"),
    ("EUR", "EUR - Euro"),
    ("GBP", "GBP - British Pound"),
    ("ETH", "ETH - Ether"),
    ("BTC", "BTC - Bitcoin"),
    ("USDC", "USDC"),
    ("USDT", "USDT"),
    ("DAI", "DAI"),
];

/// Default invoice expiration in seconds (15 minutes).
pub const DEFAULT_INVOICE_EXPIRATION_SECS: u64 = 900;

/// Invoice expiration presets for UI display: `(minutes_str, label)`.
pub const EXPIRATION_PRESETS: &[(&str, &str)] = &[
    ("15", "15 minutes"),
    ("30", "30 minutes"),
    ("60", "1 hour"),
    ("1440", "24 hours"),
];

/// Check if a currency code is a known fiat currency (case-insensitive).
pub fn is_fiat_currency(currency: &str) -> bool {
    let upper = currency.to_uppercase();
    FIAT_CURRENCIES.contains(&upper.as_str())
}

/// Check if a currency code is a cryptocurrency (not fiat).
pub fn is_crypto_currency(currency: &str) -> bool {
    !is_fiat_currency(currency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_currencies() {
        assert!(is_fiat_currency("USD"));
        assert!(is_fiat_currency("EUR"));
        assert!(is_fiat_currency("usd")); // case-insensitive
        assert!(!is_fiat_currency("ETH"));
        assert!(!is_fiat_currency("BTC"));
    }

    #[test]
    fn test_crypto_currencies() {
        assert!(is_crypto_currency("ETH"));
        assert!(is_crypto_currency("BTC"));
        assert!(is_crypto_currency("USDC"));
        assert!(!is_crypto_currency("USD"));
    }

    #[test]
    fn test_invoice_currency_options_are_valid() {
        for (code, _label) in INVOICE_CURRENCY_OPTIONS {
            assert!(
                FIAT_CURRENCIES.contains(code) || CRYPTO_CURRENCIES.contains(code),
                "{code} is not in FIAT_CURRENCIES or CRYPTO_CURRENCIES"
            );
        }
    }

    #[test]
    fn test_default_expiration_is_15_min() {
        assert_eq!(DEFAULT_INVOICE_EXPIRATION_SECS, 900);
    }

    #[test]
    fn test_expiration_presets_parseable() {
        for (mins_str, _label) in EXPIRATION_PRESETS {
            let mins: u64 = mins_str.parse().unwrap();
            assert!(mins > 0);
        }
    }
}
