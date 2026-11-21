//! Property tests for the date and time primitives: every string the
//! specification's regexes accept parses, and the JSON a parsed value writes
//! parses back to the same value.

use std::fmt::Debug;

use chrono::{Datelike, Days, NaiveDate};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use serde::{Serialize, de::DeserializeOwned};

use super::{Date, DateTime, parse_date, parse_datetime, parse_instant, parse_time};
use crate::r4::strategies::{
    date_string, date_time_string, days_in_month, instant_string, time_string,
};

/// A year, a month shorter than 31 days, and a day past its end.
fn overlong_day() -> impl Strategy<Value = (u16, u8, u8)> {
    (1u16..=9999, prop::sample::select(vec![2u8, 4, 6, 9, 11])).prop_flat_map(|(year, month)| {
        (
            Just(year),
            Just(month),
            (days_in_month(year, month) + 1)..=31,
        )
    })
}

fn parsed<T, E: Debug>(input: &str, result: Result<T, E>) -> Result<T, TestCaseError> {
    result.map_err(|error| TestCaseError::fail(format!("{input:?} did not parse: {error:?}")))
}

/// A value for the round-trip properties, which concern what parses; the
/// regex-and-parser properties cover what does not.
fn parsed_or_skip<T, E: Debug>(result: Result<T, E>) -> Result<T, TestCaseError> {
    result.map_err(|_| TestCaseError::reject("did not parse"))
}

/// The JSON `value` writes parses back to `value`.
fn round_trips<T>(input: &str, value: &T) -> Result<(), TestCaseError>
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string(value)
        .map_err(|error| TestCaseError::fail(format!("{input:?} did not serialize: {error}")))?;
    let again: T = serde_json::from_str(&json).map_err(|error| {
        TestCaseError::fail(format!(
            "{input:?} serialized as {json}, which did not parse: {error}"
        ))
    })?;
    prop_assert_eq!(&again, value, "{:?} serialized as {}", input, json);
    Ok(())
}

proptest! {
    /// Every string the `date` regex accepts parses as a date.
    #[test]
    fn date_regex_and_parser_agree(input in date_string()) {
        parsed(&input, parse_date(&input))?;
    }

    /// Every string the `dateTime` regex accepts parses as a dateTime.
    #[test]
    fn date_time_regex_and_parser_agree(input in date_time_string()) {
        parsed(&input, parse_datetime(&input))?;
    }

    /// Every string the `instant` regex accepts parses as an instant.
    #[test]
    fn instant_regex_and_parser_agree(input in instant_string()) {
        parsed(&input, parse_instant(&input))?;
    }

    /// Every string the `time` regex accepts parses as a time.
    #[test]
    fn time_regex_and_parser_agree(input in time_string()) {
        parsed(&input, parse_time(&input))?;
    }

    /// A date survives JSON: what it writes parses to the same value.
    #[test]
    fn date_round_trips_through_json(input in date_string()) {
        let value = parsed_or_skip(parse_date(&input))?;
        round_trips(&input, &value)?;
    }

    /// A dateTime survives JSON. The zone is normalized to UTC on the way,
    /// so the text may change; the instant and precision may not.
    #[test]
    fn date_time_round_trips_through_json(input in date_time_string()) {
        let value = parsed_or_skip(parse_datetime(&input))?;
        round_trips(&input, &value)?;
    }

    /// An instant survives JSON.
    #[test]
    fn instant_round_trips_through_json(input in instant_string()) {
        let value = parsed_or_skip(parse_instant(&input))?;
        round_trips(&input, &value)?;
    }

    /// A time survives JSON.
    #[test]
    fn time_round_trips_through_json(input in time_string()) {
        let value = parsed_or_skip(parse_time(&input))?;
        round_trips(&input, &value)?;
    }

    /// A day the month lacks carries into the next month by the days it
    /// overshoots, as chrono counts: 2023-02-30 is 2023-03-02.
    #[test]
    fn overlong_days_carry_into_the_next_month((year, month, day) in overlong_day()) {
        let last = days_in_month(year, month);
        let input = format!("{year:04}-{month:02}-{day:02}");
        let expected = NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(last))
            .and_then(|end| end.checked_add_days(Days::new(u64::from(day - last))))
            .expect("a date within the calendar");
        let expected = Date::YearMonthDay(
            u16::try_from(expected.year()).expect("a year"),
            u8::try_from(expected.month()).expect("a month"),
            u8::try_from(expected.day()).expect("a day"),
        );
        prop_assert_eq!(parsed(&input, parse_date(&input))?, expected);
    }

    /// A date string is also a dateTime, at the same precision.
    #[test]
    fn date_parses_as_date_time(input in date_string()) {
        let date = parsed(&input, parse_date(&input))?;
        let date_time = parsed(&input, parse_datetime(&input))?;
        let expected = match date {
            Date::Year(year) => DateTime::Year(year),
            Date::YearMonth(year, month) => DateTime::YearMonth(year, month),
            Date::YearMonthDay(year, month, day) => DateTime::YearMonthDay(year, month, day),
        };
        prop_assert_eq!(date_time, expected);
    }
}
