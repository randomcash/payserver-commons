//! One amount formatter, written once.
//!
//! This existed four times across the two repositories — a currency
//! formatter that never trimmed, two crypto formatters that duplicated the
//! same base-units division, and a fourth in `payserver-billing` — and only
//! one of the four actually trimmed trailing zeros. The rest glued a symbol
//! onto whatever `NUMERIC(38,18)` sent over the wire, zeros and all.
//!
//! Everything here works on strings, never `f64`: a `NUMERIC(38,18)` amount
//! can carry more precision than a float represents exactly, so parsing one
//! into a float and formatting it back is itself a source of the bug this
//! module exists to fix.
//!
//! Two distinct problems live here, and only one of them wants subscript
//! notation:
//!
//! - **Trailing zeros** (`30.000000000000000000`) are noise. [`trim_amount`]
//!   and [`format_units`] just trim them; a fiat amount instead rounds to its
//!   minor unit with [`round_amount`], since digits past the cent boundary
//!   must be rounded away, not merely trimmed.
//! - **Leading zeros** (`0.000000000000000001`) are a wei-scale or dust
//!   amount worth compressing for a summary display — `0.0₁₇1` — but the
//!   compression is a *summary* device. [`CompactAmount`] renders it, and it
//!   is never the right choice for an amount the reader might copy, type, or
//!   send: use the plain literal ([`format_units`] or [`trim_amount`]) there
//!   instead, since a wallet has no idea what `0.0₁₇1` means.

use leptos::prelude::*;

/// Convert a smallest-unit integer string (e.g. wei) to a decimal string, with
/// no trimming. String arithmetic throughout, so a value with more digits
/// than `u128` holds still converts correctly.
pub fn units_to_decimal(smallest_units: &str, decimals: u8) -> String {
    if let Some(magnitude) = smallest_units.strip_prefix('-') {
        return format!("-{}", units_to_decimal(magnitude, decimals));
    }
    let d = decimals as usize;
    if d == 0 {
        return smallest_units.to_string();
    }
    let len = smallest_units.len();
    if len <= d {
        format!("0.{}{}", "0".repeat(d - len), smallest_units)
    } else {
        let (int_part, frac_part) = smallest_units.split_at(len - d);
        format!("{}.{}", int_part, frac_part)
    }
}

/// Trim trailing zeros from a decimal string's fractional part, keeping at
/// least `min_decimals` digits (padding with zeros if it has fewer).
///
/// This is the one place both a currency display (`min_decimals: 2`, so
/// `"30.000000000000000000"` reads `"30.00"`) and a crypto display
/// (`min_decimals: 0`, so `"0.500000000"` reads `"0.5"`) trim through.
pub fn trim_amount(decimal: &str, min_decimals: usize) -> String {
    let Some((int_part, frac_part)) = decimal.split_once('.') else {
        return decimal.to_string();
    };
    // Every real caller comes through `units_to_decimal`, which always emits
    // a `"0."`-prefixed string, but an int-part-elided input like `".5"`
    // would otherwise silently drop its leading `0` below.
    let int_part = if int_part.is_empty() { "0" } else { int_part };
    let trimmed = frac_part.trim_end_matches('0');
    if trimmed.len() >= min_decimals {
        if trimmed.is_empty() {
            int_part.to_string()
        } else {
            format!("{int_part}.{trimmed}")
        }
    } else {
        format!(
            "{int_part}.{trimmed}{}",
            "0".repeat(min_decimals - trimmed.len())
        )
    }
}

/// Round a decimal string to exactly `scale` fractional digits, half rounding
/// away from zero. String arithmetic throughout, so a value with more digits
/// than `u128`/`f64` can represent exactly still rounds correctly.
///
/// This is the fiat-currency counterpart to [`trim_amount`]: a currency
/// amount always shows exactly its minor-unit scale (`"$30.13"`, never
/// `"$30.126"`), so digits past that scale must be rounded away, not merely
/// trimmed - trimming only removes zeros and would leave `"30.126"` alone.
pub fn round_amount(decimal: &str, scale: usize) -> String {
    let (negative, unsigned) = match decimal.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, decimal),
    };
    let (int_part, frac_part) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    // Same int-part-elided guard as `trim_amount`: `".5"` must round like
    // `"0.5"`, not lose its leading digit.
    let int_part = if int_part.is_empty() { "0" } else { int_part };

    let rounded = if frac_part.len() <= scale {
        format!(
            "{int_part}{frac_part}{}",
            "0".repeat(scale - frac_part.len())
        )
    } else {
        let kept = &frac_part[..scale];
        let round_up = frac_part.as_bytes()[scale] >= b'5';
        let combined = format!("{int_part}{kept}");
        if round_up {
            increment_digits(&combined)
        } else {
            combined
        }
    };

    let split_at = rounded.len() - scale;
    let (int_out, frac_out) = rounded.split_at(split_at);
    let magnitude = if scale == 0 {
        int_out.to_string()
    } else {
        format!("{int_out}.{frac_out}")
    };
    if negative {
        format!("-{magnitude}")
    } else {
        magnitude
    }
}

/// Add one to a string of decimal digits, growing it by a digit on overflow
/// (`"999"` -> `"1000"`).
fn increment_digits(digits: &str) -> String {
    let mut bytes = digits.as_bytes().to_vec();
    for b in bytes.iter_mut().rev() {
        if *b == b'9' {
            *b = b'0';
        } else {
            *b += 1;
            return String::from_utf8(bytes).expect("ASCII digits stay ASCII");
        }
    }
    let mut result = String::from("1");
    result.push_str(&String::from_utf8(bytes).expect("ASCII digits stay ASCII"));
    result
}

/// Format a smallest-unit amount as a trimmed decimal string.
///
/// This is the literal, exact form. Use it — never [`CompactAmount`] — for an
/// amount the reader must act on, like the payable amount on a checkout page:
/// subscript notation is a summary device, and it means nothing to a wallet.
pub fn format_units(smallest_units: &str, decimals: u8) -> String {
    trim_amount(&units_to_decimal(smallest_units, decimals), 0)
}

/// Below this many leading zeros in the fraction, showing the digits plainly
/// reads no harder than compressing them - `"0.0001"` needn't become
/// `"0.0(3)1"`. Above it, the zeros are noise a reader has to count rather
/// than a magnitude they can see at a glance.
const SUBSCRIPT_THRESHOLD: usize = 4;

/// How a [`compact_amount`] value should be rendered.
pub enum AmountDisplay {
    /// No compression was worth doing; show the literal string.
    Literal(String),
    /// `zero_count` leading zeros in the fraction, then `tail`. Rendered as
    /// `"0.0"` followed by `zero_count` as a subscript, then `tail`.
    Compact { zero_count: usize, tail: String },
}

/// Summary form of a decimal string: trims trailing zeros like
/// [`trim_amount`], and additionally compresses a long run of leading zeros
/// into a form meant for subscript rendering.
///
/// Returns the untrimmed-of-compression literal alongside the display form,
/// so a caller can always recover the exact value — the compressed form
/// carries no information a screen reader or a copy-paste should rely on.
pub fn compact_amount(decimal: &str, min_decimals: usize) -> (String, AmountDisplay) {
    let literal = trim_amount(decimal, min_decimals);
    if let Some((int_part, frac_part)) = literal.split_once('.')
        && int_part == "0"
    {
        let zero_count = frac_part.chars().take_while(|&c| c == '0').count();
        if zero_count >= SUBSCRIPT_THRESHOLD && zero_count < frac_part.len() {
            let tail = frac_part[zero_count..].to_string();
            return (literal, AmountDisplay::Compact { zero_count, tail });
        }
    }
    (literal.clone(), AmountDisplay::Literal(literal))
}

/// Render a decimal amount in summary form, with subscript-compressed
/// leading zeros for dust-scale amounts (`0.000000000000000001` -> `0.0₁₇1`).
///
/// The full value always carries through to `title` and `aria-label`: a
/// subscript digit is decoration to a screen reader and breaks text
/// selection, so the exact number stays reachable either way.
///
/// Never use this for a payable or copyable amount - see the module docs.
#[component]
pub fn CompactAmount(
    /// The amount as a plain decimal string, already in display units (not
    /// smallest units - convert with [`units_to_decimal`] first if needed).
    #[prop(into)]
    value: String,
    /// Minimum fractional digits to keep after trimming trailing zeros.
    #[prop(default = 0)]
    min_decimals: usize,
    /// Unit or currency symbol appended after the amount, e.g. `"ETH"`.
    #[prop(optional, into)]
    symbol: Option<String>,
) -> impl IntoView {
    let (literal, display) = compact_amount(&value, min_decimals);
    let symbol = symbol.unwrap_or_default();
    let full = if symbol.is_empty() {
        literal
    } else {
        format!("{literal} {symbol}")
    };
    let suffix = (!symbol.is_empty()).then(|| format!(" {symbol}"));

    match display {
        AmountDisplay::Literal(s) => view! {
            <span title=full>{s}{suffix}</span>
        }
        .into_any(),
        AmountDisplay::Compact { zero_count, tail } => view! {
            <span title=full.clone() aria-label=full>
                "0.0"<sub>{zero_count.to_string()}</sub>{tail}{suffix}
            </span>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_to_decimal_pads_when_shorter_than_decimals() {
        assert_eq!(units_to_decimal("5", 18), "0.000000000000000005");
        assert_eq!(units_to_decimal("1000000", 6), "1.000000");
    }

    #[test]
    fn units_to_decimal_splits_when_longer_than_decimals() {
        assert_eq!(units_to_decimal("123456789", 6), "123.456789");
    }

    #[test]
    fn units_to_decimal_is_a_no_op_at_zero_decimals() {
        assert_eq!(units_to_decimal("42", 0), "42");
    }

    #[test]
    fn trim_amount_removes_trailing_zeros_down_to_the_minimum() {
        // The bug from the screenshot: an all-zero NUMERIC(38,18) fraction.
        assert_eq!(trim_amount("30.000000000000000000", 2), "30.00");
        assert_eq!(trim_amount("0.000000000000000000", 2), "0.00");
    }

    #[test]
    fn trim_amount_keeps_significant_digits_past_the_minimum() {
        assert_eq!(trim_amount("0.500000000", 0), "0.5");
        assert_eq!(trim_amount("0.001", 0), "0.001");
    }

    #[test]
    fn trim_amount_pads_when_shorter_than_the_minimum() {
        assert_eq!(trim_amount("1.5", 2), "1.50");
    }

    #[test]
    fn trim_amount_passes_through_a_whole_number() {
        assert_eq!(trim_amount("100", 0), "100");
    }

    #[test]
    fn trim_amount_keeps_the_leading_zero_on_an_int_part_elided_input() {
        assert_eq!(trim_amount(".500000", 0), "0.5");
    }

    #[test]
    fn format_units_matches_the_wei_scale_example() {
        assert_eq!(format_units("1000000000000000000", 18), "1");
        assert_eq!(format_units("1500000000000000000", 18), "1.5");
    }

    #[test]
    fn units_to_decimal_preserves_a_negative_sign() {
        // -0.5 at 18 decimals: the sign must not end up stuck mid-string.
        assert_eq!(
            units_to_decimal("-500000000000000000", 18),
            "-0.500000000000000000"
        );
        assert_eq!(units_to_decimal("-42", 0), "-42");
    }

    #[test]
    fn round_amount_matches_the_screenshot_examples() {
        assert_eq!(round_amount("30.000000000000000000", 2), "30.00");
        assert_eq!(round_amount("0.000000000000000000", 2), "0.00");
    }

    #[test]
    fn round_amount_rounds_rather_than_truncates_past_the_scale() {
        // Trimming alone would leave "30.126" - a dollar amount with three
        // decimal places - since there are no trailing zeros to strip.
        assert_eq!(round_amount("30.126000000000000000", 2), "30.13");
        assert_eq!(round_amount("30.124000000000000000", 2), "30.12");
    }

    #[test]
    fn round_amount_carries_through_a_run_of_nines() {
        assert_eq!(round_amount("9.996", 2), "10.00");
    }

    #[test]
    fn round_amount_pads_a_short_or_missing_fraction() {
        assert_eq!(round_amount("30", 2), "30.00");
        assert_eq!(round_amount("1.5", 2), "1.50");
    }

    #[test]
    fn round_amount_rounds_a_negative_amount_away_from_zero() {
        assert_eq!(round_amount("-0.005", 2), "-0.01");
    }

    #[test]
    fn round_amount_keeps_the_leading_zero_on_an_int_part_elided_input() {
        assert_eq!(round_amount(".5", 2), "0.50");
    }

    #[test]
    fn compact_amount_compresses_long_runs_of_leading_zeros() {
        // "0.0000001234" -> six leading zeros, then "1234".
        let (literal, display) = compact_amount("0.0000001234", 0);
        assert_eq!(literal, "0.0000001234");
        match display {
            AmountDisplay::Compact { zero_count, tail } => {
                assert_eq!(zero_count, 6);
                assert_eq!(tail, "1234");
            }
            AmountDisplay::Literal(_) => panic!("expected compression"),
        }
    }

    #[test]
    fn compact_amount_matches_the_wei_dust_example() {
        let (literal, display) = compact_amount("0.000000000000000001", 0);
        assert_eq!(literal, "0.000000000000000001");
        match display {
            AmountDisplay::Compact { zero_count, tail } => {
                assert_eq!(zero_count, 17);
                assert_eq!(tail, "1");
            }
            AmountDisplay::Literal(_) => panic!("expected compression"),
        }
    }

    #[test]
    fn compact_amount_leaves_ordinary_values_literal() {
        let (_, display) = compact_amount("0.5", 0);
        assert!(matches!(display, AmountDisplay::Literal(s) if s == "0.5"));
    }

    #[test]
    fn compact_amount_does_not_compress_below_the_threshold() {
        // Three leading zeros: not dust enough to be worth compressing.
        let (_, display) = compact_amount("0.0001", 0);
        assert!(matches!(display, AmountDisplay::Literal(s) if s == "0.0001"));
    }

    #[test]
    fn compact_amount_leaves_zero_itself_literal() {
        // All zeros: there is no nonzero tail to compress toward.
        let (_, display) = compact_amount("0.0000000000000000", 2);
        assert!(matches!(display, AmountDisplay::Literal(s) if s == "0.00"));
    }

    #[test]
    fn the_crate_root_exports_the_formatter() {
        // See copy.rs's identical test: a `pub` item can be unreachable from
        // outside the crate while every in-crate test still passes.
        let _ = crate::format_units;
        let _ = crate::trim_amount;
        let _ = crate::round_amount;
        let _ = crate::units_to_decimal;
        let _ = crate::compact_amount;
        let _ = crate::CompactAmount;
    }
}
