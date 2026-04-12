//! Cryptocurrency amount display components.

use leptos::prelude::*;

/// Display a crypto amount with symbol.
#[component]
pub fn CryptoAmount(
    amount: String,
    symbol: String,
    #[prop(default = "md")] size: &'static str,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let size_class = format!("ps-amount-{}", size);

    view! {
        <span class=format!("ps-crypto-amount {} {}", size_class, class)>
            <span class="ps-amount-value">{amount}</span>
            <span class="ps-amount-symbol">{symbol}</span>
        </span>
    }
}

/// Display amount with fiat equivalent.
#[component]
pub fn AmountWithFiat(
    crypto_amount: String,
    crypto_symbol: String,
    #[prop(optional)] fiat_amount: Option<String>,
    #[prop(default = "USD")] fiat_symbol: &'static str,
) -> impl IntoView {
    view! {
        <div class="ps-amount-with-fiat">
            <CryptoAmount amount=crypto_amount symbol=crypto_symbol size="lg" />
            {fiat_amount.map(|fiat| view! {
                <span class="ps-amount-fiat">
                    "≈ " {fiat} " " {fiat_symbol}
                </span>
            })}
        </div>
    }
}

/// Price display component.
#[component]
pub fn Price(
    amount: String,
    #[prop(default = "USD")] currency: &'static str,
    #[prop(default = "md")] size: &'static str,
) -> impl IntoView {
    let size_class = format!("ps-price-{}", size);
    let symbol = match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "",
    };

    view! {
        <span class=format!("ps-price {}", size_class)>
            <span class="ps-price-symbol">{symbol}</span>
            <span class="ps-price-value">{amount}</span>
            {(symbol.is_empty()).then(|| view! {
                <span class="ps-price-currency">{currency}</span>
            })}
        </span>
    }
}

/// Format a large number with appropriate suffixes (K, M, B).
pub fn format_large_number(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("{:.2}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}K", value / 1_000.0)
    } else {
        format!("{:.2}", value)
    }
}

/// Format a crypto amount with appropriate decimal places.
pub fn format_crypto_amount(amount: &str, decimals: u8) -> String {
    let value: f64 = amount.parse().unwrap_or(0.0);

    if value == 0.0 {
        return "0".to_string();
    }

    if value < 0.0001 && value > 0.0 {
        return format!("{:.2e}", value);
    }

    let display_decimals = if value >= 1.0 {
        2.min(decimals)
    } else if value >= 0.01 {
        4.min(decimals)
    } else {
        6.min(decimals)
    };

    format!("{:.1$}", value, display_decimals as usize)
}

/// Amount styles CSS.
pub const AMOUNT_STYLES: &str = r#"
.ps-crypto-amount {
    display: inline-flex;
    align-items: baseline;
    gap: var(--ps-spacing-xs);
    font-weight: 500;
}

.ps-amount-sm { font-size: var(--ps-font-sm); }
.ps-amount-md { font-size: var(--ps-font-md); }
.ps-amount-lg { font-size: var(--ps-font-xl); }

.ps-amount-value {
    font-family: monospace;
    color: var(--ps-text);
}

.ps-amount-symbol {
    font-size: 0.85em;
    color: var(--ps-text-muted);
    text-transform: uppercase;
}

.ps-amount-with-fiat {
    display: flex;
    flex-direction: column;
    gap: var(--ps-spacing-xs);
}

.ps-amount-fiat {
    font-size: var(--ps-font-sm);
    color: var(--ps-text-muted);
}

.ps-price {
    display: inline-flex;
    align-items: baseline;
    font-weight: 500;
    color: var(--ps-text);
}

.ps-price-sm { font-size: var(--ps-font-sm); }
.ps-price-md { font-size: var(--ps-font-md); }
.ps-price-lg { font-size: var(--ps-font-xl); }

.ps-price-symbol { margin-right: 0.1em; }
.ps-price-value { font-family: monospace; }
.ps-price-currency {
    margin-left: var(--ps-spacing-xs);
    font-size: 0.85em;
    color: var(--ps-text-muted);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_large_number_billions() {
        assert_eq!(format_large_number(1_500_000_000.0), "1.50B");
        assert_eq!(format_large_number(10_000_000_000.0), "10.00B");
    }

    #[test]
    fn test_format_large_number_millions() {
        assert_eq!(format_large_number(1_500_000.0), "1.50M");
        assert_eq!(format_large_number(999_999_999.0), "1000.00M");
    }

    #[test]
    fn test_format_large_number_thousands() {
        assert_eq!(format_large_number(1_500.0), "1.50K");
        assert_eq!(format_large_number(999_999.0), "1000.00K");
    }

    #[test]
    fn test_format_large_number_small() {
        assert_eq!(format_large_number(999.0), "999.00");
        assert_eq!(format_large_number(1.5), "1.50");
        assert_eq!(format_large_number(0.0), "0.00");
    }

    #[test]
    fn test_format_crypto_amount_zero() {
        assert_eq!(format_crypto_amount("0", 18), "0");
        assert_eq!(format_crypto_amount("0.0", 18), "0");
        assert_eq!(format_crypto_amount("invalid", 18), "0");
    }

    #[test]
    fn test_format_crypto_amount_large_values() {
        // Large values get 2 decimal places (or less if decimals param is lower)
        assert_eq!(format_crypto_amount("100.123456", 18), "100.12");
        assert_eq!(format_crypto_amount("1.999", 18), "2.00");
        assert_eq!(format_crypto_amount("50", 18), "50.00");
    }

    #[test]
    fn test_format_crypto_amount_medium_values() {
        // Values >= 0.01 and < 1.0 get 4 decimal places
        assert_eq!(format_crypto_amount("0.123456", 18), "0.1235");
        assert_eq!(format_crypto_amount("0.01", 18), "0.0100");
        assert_eq!(format_crypto_amount("0.5", 18), "0.5000");
    }

    #[test]
    fn test_format_crypto_amount_small_values() {
        // Values >= 0.0001 and < 0.01 get 6 decimal places
        assert_eq!(format_crypto_amount("0.001234", 18), "0.001234");
        assert_eq!(format_crypto_amount("0.009999", 18), "0.009999");
    }

    #[test]
    fn test_format_crypto_amount_very_small_values() {
        // Values < 0.0001 get scientific notation
        let result = format_crypto_amount("0.00001", 18);
        assert!(
            result.contains("e"),
            "Expected scientific notation, got: {}",
            result
        );
    }

    #[test]
    fn test_format_crypto_amount_respects_decimals_param() {
        // When decimals param is lower than calculated, use decimals param
        assert_eq!(format_crypto_amount("100.123456", 1), "100.1");
        assert_eq!(format_crypto_amount("0.123456", 2), "0.12");
    }
}
