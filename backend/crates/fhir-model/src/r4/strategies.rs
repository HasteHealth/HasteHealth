//! `proptest` strategies for FHIR R4 primitives, shared by the crates that
//! parse, index or serialize them.
//!
//! Compiled for this crate's own tests and, behind the `testing` feature, for
//! the dev-dependencies of other crates:
//!
//! ```toml
//! [dev-dependencies]
//! haste-fhir-model = { path = "../fhir-model", version = "0.*", features = ["testing"] }
//! ```
//!
//! The string strategies are driven by the regexes the parsers themselves
//! use, so they generate exactly what the specification's grammar admits,
//! calendar-impossible dates included. The structured strategies generate
//! only values that exist.

use chrono::{Datelike, NaiveDate, TimeDelta, TimeZone, Utc};
use proptest::prelude::*;
use proptest::string::string_regex;

use super::datetime::{DATE_REGEX, DATETIME_REGEX, DateTime, INSTANT_REGEX, TIME_REGEX};

/// The pattern of `regex` without its `^` and `$` anchors, which
/// `string_regex` does not generate from.
fn unanchored(regex: &regex::Regex) -> &str {
    regex.as_str().trim_start_matches('^').trim_end_matches('$')
}

fn matching(regex: &regex::Regex) -> BoxedStrategy<String> {
    string_regex(unanchored(regex))
        .expect("the FHIR primitive regexes generate")
        .boxed()
}

/// A string the FHIR `date` regex accepts: a year, a year and month, or a
/// full date. The regex accepts day 31 in every month and February 30, so
/// not every value is a calendar date; [`calendar_date`] gives only those.
pub fn date_string() -> BoxedStrategy<String> {
    matching(&DATE_REGEX)
}

/// A string the FHIR `dateTime` regex accepts: a [`date_string`], or a full
/// date with a time and zone.
pub fn date_time_string() -> BoxedStrategy<String> {
    matching(&DATETIME_REGEX)
}

/// A string the FHIR `instant` regex accepts: a full date, time and zone.
pub fn instant_string() -> BoxedStrategy<String> {
    matching(&INSTANT_REGEX)
}

/// A string the FHIR `time` regex accepts.
pub fn time_string() -> BoxedStrategy<String> {
    matching(&TIME_REGEX)
}

/// A calendar date as `(year, month, day)`: years 1 to 9999, and only the
/// days the month has.
pub fn calendar_date() -> BoxedStrategy<(u16, u8, u8)> {
    (1u16..=9999, 1u8..=12)
        .prop_flat_map(|(year, month)| (Just(year), Just(month), 1u8..=days_in_month(year, month)))
        .boxed()
}

/// The number of days in `month` of `year`.
#[must_use]
pub fn days_in_month(year: u16, month: u8) -> u8 {
    let first_of_next = if month == 12 {
        NaiveDate::from_ymd_opt(i32::from(year) + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(i32::from(year), u32::from(month) + 1, 1)
    };
    let last = first_of_next
        .and_then(|date| date.pred_opt())
        .expect("every month has a last day");
    u8::try_from(last.day()).expect("a day of the month fits in a u8")
}

/// A time of day as `(hour, minute, second, millisecond)`, without leap
/// seconds.
pub fn time_of_day() -> BoxedStrategy<(u32, u32, u32, u32)> {
    (0u32..24, 0u32..60, 0u32..60, 0u32..1000).boxed()
}

/// A [`DateTime`] at a random precision: a year, a year and month, a
/// calendar date, or an instant on one.
pub fn date_time_value() -> BoxedStrategy<DateTime> {
    (calendar_date(), time_of_day(), 0u8..4)
        .prop_map(
            |((year, month, day), (hour, minute, second, milli), precision)| match precision {
                0 => DateTime::Year(year),
                1 => DateTime::YearMonth(year, month),
                2 => DateTime::YearMonthDay(year, month, day),
                _ => DateTime::Iso8601(
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
                ),
            },
        )
        .boxed()
}

/// A FHIR `decimal` in plain notation, `-?(0|[1-9][0-9]*)(\.[0-9]+)?`, with
/// the digit counts bounded so an `f64` holds the value closely.
pub fn decimal_string() -> BoxedStrategy<String> {
    string_regex(r"-?(0|[1-9][0-9]{0,8})(\.[0-9]{1,6})?")
        .expect("a decimal generates")
        .boxed()
}

/// A FHIR `decimal` in the exponent notation the specification also allows,
/// such as `1e2` or `-1.5E-3`. The exponent is a single digit so the value
/// and its precision stay within what an `f64` represents.
pub fn decimal_string_with_exponent() -> BoxedStrategy<String> {
    string_regex(r"-?(0|[1-9][0-9]{0,8})(\.[0-9]{1,6})?[eE][+-]?[0-9]")
        .expect("a decimal with an exponent generates")
        .boxed()
}

/// A FHIR `id`: up to 64 letters, digits, hyphens and dots.
pub fn id_string() -> BoxedStrategy<String> {
    string_regex(r"[A-Za-z0-9\-.]{1,64}")
        .expect("an id generates")
        .boxed()
}
