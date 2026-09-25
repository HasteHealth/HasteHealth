//! Range arithmetic for search prefixes, shared by both backends.
//!
//! A decimal's implicit precision range is half-open: `45` covers
//! `[44.5, 45.5)`, so `44` and `45` share the boundary `44.5` without
//! overlapping.

use crate::indexing_conversion::DecimalRange;

/// `ap` margin. FHIR leaves it to the server; like other servers this uses 10%.
const MARGIN: f64 = 0.1;

/// Fraction of a precision step treated as float rounding error. Far below
/// any real difference between decimals, far above `f64` rounding.
const TOLERANCE: f64 = 1e-9;

/// The lowest indexed range start that lies wholly above `range` (`gt`/`sa`).
/// Touching counts: `45` (`[44.5, 45.5)`) is above `44` (`[43.5, 44.5)`).
#[must_use]
pub fn above(range: &DecimalRange) -> f64 {
    range.end - tolerance(range)
}

/// The highest indexed range end that lies wholly below `range` (`lt`/`eb`).
#[must_use]
pub fn below(range: &DecimalRange) -> f64 {
    range.start + tolerance(range)
}

fn tolerance(range: &DecimalRange) -> f64 {
    (range.end - range.start) * TOLERANCE
}

/// A decimal's precision range widened by 10% of its value.
#[must_use]
pub fn approximate_decimal(range: &DecimalRange) -> (f64, f64) {
    let value = f64::midpoint(range.start, range.end);
    let margin = (value * MARGIN).abs();
    (
        range.start.min(value - margin),
        range.end.max(value + margin),
    )
}

/// A date range (epoch ms) widened by 10% of each bound's distance from `now_ms`.
#[must_use]
pub fn approximate_date(start: i64, end: i64, now_ms: i64) -> (i64, i64) {
    let margin = |bound: i64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let margin = ((now_ms - bound).unsigned_abs() as f64 * MARGIN) as i64;
        margin
    };
    (start - margin(start), end + margin(end))
}

/// Current time in epoch milliseconds, for [`approximate_date`].
#[must_use]
pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::indexing_conversion::get_decimal_range;

    fn range(value: &str) -> DecimalRange {
        get_decimal_range(value).unwrap()
    }

    /// Adjacent values share a boundary; float rounding must not decide it.
    #[test]
    fn adjacent_values_are_above_and_below_each_other() {
        assert!(range("45").start >= above(&range("44")));
        assert!(range("45").start < above(&range("45")));
        assert!(range("45").end <= below(&range("46")));
        assert!(range("45").end > below(&range("45")));

        assert!(range("7.2").start >= above(&range("7.1")));
        assert!(range("7.2").end <= below(&range("7.3")));
        assert!(range("7.2").start < above(&range("7.2")));
        // Inside 44's implicit range, so not above it.
        assert!(range("44.4").start < above(&range("44")));
    }

    #[test]
    fn decimals_widen_by_ten_percent() {
        let (low, high) = approximate_decimal(&DecimalRange {
            start: 69.5,
            end: 70.5,
        });
        assert!((low - 63.0).abs() < 1e-9 && (high - 77.0).abs() < 1e-9);

        // Negative values widen in both directions too.
        let (low, high) = approximate_decimal(&DecimalRange {
            start: -10.5,
            end: -9.5,
        });
        assert!((low + 11.0).abs() < 1e-9 && (high + 9.0).abs() < 1e-9);
    }

    #[test]
    fn dates_widen_by_their_distance_from_now() {
        assert_eq!(approximate_date(0, 100, 1_000), (-100, 190));
    }
}
