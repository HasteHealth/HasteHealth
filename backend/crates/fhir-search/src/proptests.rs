//! Property tests for the value parsing and range arithmetic every backend
//! shares. Nothing here touches a database.
//!
//! The decimal properties hold the code to the half-open precision range the
//! module documents: `45` is `[44.5, 45.5)`, and `1e2` is `[50, 150)`
//! (<https://hl7.org/fhir/R4/search.html#number>). The date properties hold
//! it to the ranges a date implies at each precision and to the tiling of the
//! timeline by consecutive dates.

use chrono::{Datelike, NaiveDate, TimeDelta, TimeZone, Utc};
use haste_fhir_client::url::Prefix;
use haste_fhir_model::r4::{
    datetime::{DateTime, parse_datetime},
    strategies::{
        calendar_date, date_string, date_time_value, decimal_string_with_exponent, id_string,
        time_of_day,
    },
};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use crate::{
    indexing_conversion::{DateRange, DecimalRange, date_time_range, get_decimal_range},
    query::value::{
        DateValue, IdentifierTypeValue, NumberValue, QuantityValue, ReferenceValue, TokenSystem,
        TokenValue,
    },
    search_ranges::{above, approximate_date, approximate_decimal, below},
};

/// A decimal as an integer mantissa over `10^scale`, so its neighbours at
/// the same precision can be written exactly. A mantissa of up to fourteen
/// digits: fourteen significant digits, within what an `f64` tells apart.
#[derive(Debug, Clone, Copy)]
struct Scaled {
    mantissa: i64,
    scale: u32,
}

impl Scaled {
    fn render(self) -> String {
        let sign = if self.mantissa < 0 { "-" } else { "" };
        let magnitude = self.mantissa.unsigned_abs();
        if self.scale == 0 {
            return format!("{sign}{magnitude}");
        }
        let unit = 10u64.pow(self.scale);
        format!(
            "{sign}{}.{:0width$}",
            magnitude / unit,
            magnitude % unit,
            width = self.scale as usize
        )
    }

    fn next(self) -> Self {
        Self {
            mantissa: self.mantissa + 1,
            ..self
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn value(self) -> f64 {
        self.mantissa as f64 / 10f64.powi(self.scale.cast_signed())
    }

    fn step(self) -> f64 {
        10f64.powi(-self.scale.cast_signed())
    }
}

fn scaled() -> impl Strategy<Value = Scaled> {
    (-99_999_999_999_999i64..=99_999_999_999_999, 0u32..=6)
        .prop_map(|(mantissa, scale)| Scaled { mantissa, scale })
}

/// Epoch milliseconds between the years 1 and 9999, the span a date index
/// can hold.
fn epoch_ms() -> impl Strategy<Value = i64> {
    -62_135_596_800_000i64..=253_402_300_799_999
}

fn prefixes() -> impl Strategy<Value = (&'static str, Prefix)> {
    prop::sample::select(vec![
        ("eq", Prefix::Eq),
        ("ne", Prefix::Ne),
        ("gt", Prefix::Gt),
        ("lt", Prefix::Lt),
        ("ge", Prefix::Ge),
        ("le", Prefix::Le),
        ("sa", Prefix::Sa),
        ("eb", Prefix::Eb),
        ("ap", Prefix::Ap),
    ])
}

/// A token or quantity segment: a system, code or unit, possibly empty.
fn segment() -> impl Strategy<Value = String> {
    "[A-Za-z0-9:/.#_-]{0,10}"
}

fn resource_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "Patient",
        "Observation",
        "Encounter",
        "Organization",
        "Practitioner",
        "User",
        "Membership",
    ])
}

fn failed<T, E: std::fmt::Debug>(what: &str, result: Result<T, E>) -> Result<T, TestCaseError> {
    result.map_err(|error| TestCaseError::fail(format!("{what}: {error:?}")))
}

fn decimal_range(text: &str) -> Result<DecimalRange, TestCaseError> {
    failed(
        &format!("{text:?} has no decimal range"),
        get_decimal_range(text),
    )
}

fn date_range(value: &DateTime) -> Result<DateRange, TestCaseError> {
    failed(
        &format!("{value:?} has no date range"),
        date_time_range(value),
    )
}

/// How far a range's width may differ from `step` for a value of this
/// magnitude: a millionth of the step, plus the `f64` rounding of each bound.
fn width_slack(step: f64, value: f64) -> f64 {
    step * 1e-6 + value.abs() * 8.0 * f64::EPSILON
}

fn contains(outer: &DateRange, inner: &DateRange) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

/// `value` as a search parameter value, zero-padded as the specification
/// writes dates.
fn render(value: &DateTime) -> String {
    match value {
        DateTime::Year(year) => format!("{year:04}"),
        DateTime::YearMonth(year, month) => format!("{year:04}-{month:02}"),
        DateTime::YearMonthDay(year, month, day) => format!("{year:04}-{month:02}-{day:02}"),
        DateTime::Iso8601(instant) => instant.to_rfc3339(),
    }
}

fn calendar_date_time(
    (year, month, day): (u16, u8, u8),
    (hour, minute, second, milli): (u32, u32, u32, u32),
) -> DateTime {
    DateTime::Iso8601(
        Utc.with_ymd_and_hms(
            i32::from(year),
            u32::from(month),
            u32::from(day),
            hour,
            minute,
            second,
        )
        .single()
        .expect("a calendar date and a time of day")
            + TimeDelta::milliseconds(i64::from(milli)),
    )
}

proptest! {
    /// A decimal's range contains the value and is one unit of the value's
    /// precision wide.
    #[test]
    fn decimal_range_is_one_precision_step_around_the_value(decimal in scaled()) {
        let text = decimal.render();
        let range = decimal_range(&text)?;
        let value = decimal.value();
        prop_assert!(
            range.start <= value && value < range.end,
            "{} is outside its range [{}, {})", text, range.start, range.end
        );
        let width = range.end - range.start;
        prop_assert!(
            (width - decimal.step()).abs() <= width_slack(decimal.step(), value),
            "{} has a range {} wide, expected {}", text, width, decimal.step()
        );
    }

    /// The same in exponent notation, where the precision is the mantissa's:
    /// `1e2` is 100 to one significant figure, the range [50, 150).
    #[test]
    fn exponent_decimal_range_is_one_precision_step_around_the_value(
        text in decimal_string_with_exponent()
    ) {
        let value: f64 = failed(&format!("{text:?} is not an f64"), text.parse())?;
        let range = decimal_range(&text)?;
        prop_assert!(
            range.start <= value && value < range.end,
            "{} is outside its range [{}, {})", text, range.start, range.end
        );
        let (mantissa, exponent) = text.split_once(['e', 'E']).expect("an exponent");
        let fraction_digits = mantissa.split_once('.').map_or(0, |(_, fraction)| fraction.len());
        let exponent: i32 = exponent.parse().expect("a signed digit");
        let step = 10f64.powi(exponent - i32::try_from(fraction_digits).expect("a few digits"));
        let width = range.end - range.start;
        prop_assert!(
            (width - step).abs() <= width_slack(step, value),
            "{} has a range {} wide, expected {}", text, width, step
        );
    }

    /// Values one precision step apart are `above` and `below` each other,
    /// and a value is neither above nor below itself.
    #[test]
    fn adjacent_decimals_are_above_and_below_each_other(decimal in scaled()) {
        let lower_text = decimal.render();
        let upper_text = decimal.next().render();
        let lower = decimal_range(&lower_text)?;
        let upper = decimal_range(&upper_text)?;
        prop_assert!(
            upper.start >= above(&lower),
            "{} is not above {}: {} < {}", upper_text, lower_text, upper.start, above(&lower)
        );
        prop_assert!(
            lower.end <= below(&upper),
            "{} is not below {}: {} > {}", lower_text, upper_text, lower.end, below(&upper)
        );
        prop_assert!(lower.start < above(&lower), "{} is above itself", lower_text);
        prop_assert!(lower.end > below(&lower), "{} is below itself", lower_text);
    }

    /// `ap` widens a decimal's range by a tenth of the value each way and
    /// never narrows it.
    #[test]
    fn approximate_decimal_widens_by_a_tenth(decimal in scaled()) {
        let text = decimal.render();
        let range = decimal_range(&text)?;
        let (low, high) = approximate_decimal(&range);
        prop_assert!(low <= range.start && high >= range.end, "ap narrowed {}", text);
        let value = decimal.value();
        let margin = (value * 0.1).abs();
        let slack = (margin + decimal.step()) * 1e-6;
        prop_assert!(
            low <= value - margin + slack && high >= value + margin - slack,
            "ap {} is [{}, {}], expected at least [{}, {}]",
            text, low, high, value - margin, value + margin
        );
    }

    /// `ap` widens a date range by a tenth of each bound's distance from now
    /// and never narrows it.
    #[test]
    fn approximate_date_widens_by_a_tenth_of_the_distance_from_now(
        start in epoch_ms(), length in 0i64..=400_000_000_000, now in epoch_ms()
    ) {
        let end = start + length;
        let (low, high) = approximate_date(start, end, now);
        prop_assert!(low <= start && high >= end);
        prop_assert!(((start - low) - (now - start).abs() / 10).abs() <= 1);
        prop_assert!(((high - end) - (now - end).abs() / 10).abs() <= 1);
    }

    /// A coarser date's range contains every finer date that starts with it,
    /// and an instant's range is a point.
    #[test]
    fn coarser_dates_contain_finer_ones(date in calendar_date(), time in time_of_day()) {
        let (year, month, day) = date;
        let year_range = date_range(&DateTime::Year(year))?;
        let month_range = date_range(&DateTime::YearMonth(year, month))?;
        let day_range = date_range(&DateTime::YearMonthDay(year, month, day))?;
        let instant_range = date_range(&calendar_date_time(date, time))?;
        prop_assert!(contains(&year_range, &month_range));
        prop_assert!(contains(&month_range, &day_range));
        prop_assert!(contains(&day_range, &instant_range));
        prop_assert_eq!(instant_range.start, instant_range.end);
    }

    /// Consecutive days, months and years tile the timeline: each range ends
    /// the millisecond before the next begins.
    #[test]
    fn consecutive_dates_tile_the_timeline((year, month, day) in calendar_date()) {
        prop_assume!(year < 9999);
        let date = NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
            .expect("a calendar date");
        let next = date.succ_opt().expect("a next day");
        let next = DateTime::YearMonthDay(
            u16::try_from(next.year()).expect("a year"),
            u8::try_from(next.month()).expect("a month"),
            u8::try_from(next.day()).expect("a day"),
        );
        let today = date_range(&DateTime::YearMonthDay(year, month, day))?;
        prop_assert_eq!(today.end + 1, date_range(&next)?.start);

        let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
        let this_month = date_range(&DateTime::YearMonth(year, month))?;
        prop_assert_eq!(
            this_month.end + 1,
            date_range(&DateTime::YearMonth(next_year, next_month))?.start
        );

        let this_year = date_range(&DateTime::Year(year))?;
        prop_assert_eq!(this_year.end + 1, date_range(&DateTime::Year(year + 1))?.start);
    }

    /// Every date the model accepts can be indexed.
    #[test]
    fn every_accepted_date_indexes(input in date_string()) {
        let value = failed(&format!("{input:?} did not parse"), parse_datetime(&input))?;
        prop_assert!(
            date_time_range(&value).is_ok(),
            "{} parsed as {:?} but has no index range", input, value
        );
    }

    /// A prefixed date value keeps its prefix and, except for `ap`, exactly
    /// the unprefixed date's range; `ap` only widens it.
    #[test]
    fn date_value_keeps_prefix_and_range((code, prefix) in prefixes(), value in date_time_value()) {
        let text = render(&value);
        let plain = failed(&format!("{text:?} did not parse"), DateValue::parse(&text))?;
        let prefixed_text = format!("{code}{text}");
        let prefixed = failed(
            &format!("{prefixed_text:?} did not parse"),
            DateValue::parse(&prefixed_text),
        )?;
        prop_assert_eq!(plain.prefix, Prefix::Eq);
        prop_assert_eq!(prefixed.prefix, prefix);
        if prefix == Prefix::Ap {
            prop_assert!(
                prefixed.range.start <= plain.range.start && prefixed.range.end >= plain.range.end
            );
        } else {
            prop_assert_eq!(
                (prefixed.range.start, prefixed.range.end),
                (plain.range.start, plain.range.end)
            );
        }
    }

    /// A prefixed number value keeps its prefix, value and, except for `ap`,
    /// exactly the unprefixed number's range; `ap` only widens it.
    #[test]
    fn number_value_keeps_prefix_and_range((code, prefix) in prefixes(), decimal in scaled()) {
        let text = decimal.render();
        let plain = failed(&format!("{text:?} did not parse"), NumberValue::parse(&text))?;
        let prefixed_text = format!("{code}{text}");
        let prefixed = failed(
            &format!("{prefixed_text:?} did not parse"),
            NumberValue::parse(&prefixed_text),
        )?;
        prop_assert_eq!(plain.prefix, Prefix::Eq);
        prop_assert_eq!(prefixed.prefix, prefix);
        prop_assert_eq!(prefixed.value, plain.value);
        if prefix == Prefix::Ap {
            prop_assert!(
                prefixed.range.start <= plain.range.start && prefixed.range.end >= plain.range.end
            );
        } else {
            prop_assert_eq!(
                (prefixed.range.start, prefixed.range.end),
                (plain.range.start, plain.range.end)
            );
        }
    }

    /// A quantity's segments parse to the amount, system and code given, with
    /// an empty segment unconstrained.
    #[test]
    fn quantity_value_keeps_its_segments(
        (code, prefix) in prefixes(), decimal in scaled(), system in segment(), unit in segment()
    ) {
        let text = format!("{code}{}|{system}|{unit}", decimal.render());
        let value = failed(&format!("{text:?} did not parse"), QuantityValue::parse(&text))?;
        let amount = value.amount.as_ref().expect("an amount");
        prop_assert_eq!(amount.prefix, prefix);
        prop_assert_eq!(amount.value, decimal.value());
        prop_assert_eq!(value.system, (!system.is_empty()).then_some(system.as_str()));
        prop_assert_eq!(value.code, (!unit.is_empty()).then_some(unit.as_str()));

        let bare = decimal.render();
        let value = failed(&format!("{bare:?} did not parse"), QuantityValue::parse(&bare))?;
        prop_assert!(value.system.is_none() && value.code.is_none());
    }

    /// A token's `system|code` parses to the system and code given: an absent
    /// system means any, an empty one means none.
    #[test]
    fn token_value_keeps_system_and_code(system in segment(), code in segment()) {
        let value = failed(&format!("{code:?} did not parse"), TokenValue::parse(&code))?;
        prop_assert!(matches!(value.system, TokenSystem::Any));
        prop_assert_eq!(value.code, Some(code.as_str()));

        let text = format!("{system}|{code}");
        let value = failed(&format!("{text:?} did not parse"), TokenValue::parse(&text))?;
        match (system.is_empty(), code.is_empty()) {
            (true, true) => {
                prop_assert!(matches!(value.system, TokenSystem::Any));
                prop_assert_eq!(value.code, None);
            }
            (true, false) => {
                prop_assert!(matches!(value.system, TokenSystem::Absent));
                prop_assert_eq!(value.code, Some(code.as_str()));
            }
            (false, _) => {
                prop_assert!(matches!(value.system, TokenSystem::Is(s) if s == system));
                prop_assert_eq!(value.code, (!code.is_empty()).then_some(code.as_str()));
            }
        }
    }

    /// A reference parses to its type and id, takes the type the modifier
    /// supplies when it has none, and rejects a type that contradicts it.
    #[test]
    fn reference_value_keeps_type_and_id(
        resource_type in resource_type(), other_type in resource_type(), id in id_string()
    ) {
        let qualified = format!("{resource_type}/{id}");
        let value = failed(&format!("{qualified:?} did not parse"), ReferenceValue::parse(&qualified, None))?;
        prop_assert_eq!((value.resource_type, value.id), (Some(resource_type), id.as_str()));

        let value = failed(&format!("{id:?} did not parse"), ReferenceValue::parse(&id, Some(resource_type)))?;
        prop_assert_eq!((value.resource_type, value.id), (Some(resource_type), id.as_str()));

        let value = failed(&format!("{id:?} did not parse"), ReferenceValue::parse(&id, None))?;
        prop_assert_eq!((value.resource_type, value.id), (None, id.as_str()));

        prop_assert_eq!(
            ReferenceValue::parse(&qualified, Some(other_type)).is_ok(),
            other_type == resource_type
        );
    }

    /// Value parsing never panics, whatever the text.
    #[test]
    fn value_parsers_never_panic(text in ".*", modifier_type in prop::option::of("[A-Za-z]{1,16}")) {
        let _ = TokenValue::parse(&text);
        let _ = IdentifierTypeValue::parse(&text);
        let _ = ReferenceValue::parse(&text, modifier_type.as_deref());
        let _ = QuantityValue::parse(&text);
        let _ = NumberValue::parse(&text);
        let _ = DateValue::parse(&text);
    }
}
