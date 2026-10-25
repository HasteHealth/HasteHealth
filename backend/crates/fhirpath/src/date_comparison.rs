//! Comparing Date, `DateTime` and Time values for `=`, `!=`, `~`, `!~`, `<`, `>`, `<=` and
//! `>=` (<https://hl7.org/fhirpath/N1/#datetime-equality> and
//! <https://hl7.org/fhirpath/N1/#comparison>).
//!
//! Values are compared one precision at a time, from the year (the hour for a Time) down to
//! the second, where seconds and milliseconds are one precision. A Date compares as a
//! `DateTime` of the same precision, and a FHIR `instant` as a `DateTime`. Times with a
//! timezone offset are compared in UTC.

use chrono::{Datelike, NaiveDateTime, NaiveTime, Timelike};
use haste_fhir_model::r4::datetime::{Date, DateTime, Instant, Time};
use haste_reflect::MetaValue;
use std::cmp::Ordering;

/// How two Date, `DateTime` or Time values compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalOrder {
    Ordered(Ordering),
    /// They agree as far as both go but one has a precision the other lacks, or both have a
    /// time and only one has a timezone offset. `=` and the ordering operators give empty,
    /// `~` gives false.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A Date, `DateTime` or `instant`.
    DateTime,
    Time,
}

/// A value's parts from the coarsest precision down: year, month, day, hour, minute and
/// second for a `DateTime`, hour, minute and second for a Time. The second is in
/// nanoseconds, so a fraction is part of it.
struct Parts {
    kind: Kind,
    values: [i64; 6],
    len: usize,
    /// Whether the time has a timezone offset; `None` when there is no time.
    zoned: Option<bool>,
}

fn nanoseconds(time: &NaiveTime) -> i64 {
    i64::from(time.second()) * 1_000_000_000 + i64::from(time.nanosecond())
}

/// The number of parts a Time has.
fn time_len(time: &Time) -> usize {
    match time {
        Time::Hour(_) => 1,
        Time::HourMinute(..) => 2,
        Time::HourMinuteSecond(_) => 3,
    }
}

impl Parts {
    fn date(parts: &[i64]) -> Self {
        let mut values = [0; 6];
        values[..parts.len()].copy_from_slice(parts);
        Parts {
            kind: Kind::DateTime,
            values,
            len: parts.len(),
            zoned: None,
        }
    }

    fn date_time(value: &NaiveDateTime, len: usize, zoned: bool) -> Self {
        Parts {
            kind: Kind::DateTime,
            values: [
                i64::from(value.year()),
                i64::from(value.month()),
                i64::from(value.day()),
                i64::from(value.hour()),
                i64::from(value.minute()),
                nanoseconds(&value.time()),
            ],
            len,
            zoned: Some(zoned),
        }
    }

    fn of(value: &dyn MetaValue) -> Option<Self> {
        // A FHIR primitive holds its System value in `value`, read the same way through a Box.
        let value = match value.fhir_type() {
            "date" | "dateTime" | "instant" | "time" => value.get_field("value")?,
            _ => value,
        };
        let any = value.as_any();
        if let Some(date) = any.downcast_ref::<Date>() {
            Some(match date {
                Date::Year(year) => Parts::date(&[i64::from(*year)]),
                Date::YearMonth(year, month) => Parts::date(&[i64::from(*year), i64::from(*month)]),
                Date::YearMonthDay(year, month, day) => {
                    Parts::date(&[i64::from(*year), i64::from(*month), i64::from(*day)])
                }
            })
        } else if let Some(date_time) = any.downcast_ref::<DateTime>() {
            Some(match date_time {
                DateTime::Year(year) => Parts::date(&[i64::from(*year)]),
                DateTime::YearMonth(year, month) => {
                    Parts::date(&[i64::from(*year), i64::from(*month)])
                }
                DateTime::YearMonthDay(year, month, day) => {
                    Parts::date(&[i64::from(*year), i64::from(*month), i64::from(*day)])
                }
                DateTime::Iso8601(utc) => Parts::date_time(&utc.naive_utc(), 6, true),
                DateTime::Partial(date, time, offset) => {
                    let local = date.and_time(time.to_naive());
                    let utc = match offset {
                        Some(offset) => local.checked_sub_offset(*offset)?,
                        None => local,
                    };
                    Parts::date_time(&utc, 3 + time_len(time), offset.is_some())
                }
            })
        } else if let Some(Instant::Iso8601(utc)) = any.downcast_ref::<Instant>() {
            Some(Parts::date_time(&utc.naive_utc(), 6, true))
        } else if let Some(time) = any.downcast_ref::<Time>() {
            let clock = time.to_naive();
            Some(Parts {
                kind: Kind::Time,
                values: [
                    i64::from(clock.hour()),
                    i64::from(clock.minute()),
                    nanoseconds(&clock),
                    0,
                    0,
                    0,
                ],
                len: time_len(time),
                zoned: None,
            })
        } else {
            None
        }
    }
}

/// How `left` compares with `right`, or `None` when they are not both Date or `DateTime`
/// values, or both Time values.
pub fn temporal_order(left: &dyn MetaValue, right: &dyn MetaValue) -> Option<TemporalOrder> {
    let (left, right) = (Parts::of(left)?, Parts::of(right)?);
    if left.kind != right.kind {
        return None;
    }
    // Without a default offset, a time with one cannot be compared with a time without.
    if matches!((left.zoned, right.zoned), (Some(l), Some(r)) if l != r) {
        return Some(TemporalOrder::Unknown);
    }
    for (l, r) in left.values[..left.len]
        .iter()
        .zip(&right.values[..right.len])
    {
        if l != r {
            return Some(TemporalOrder::Ordered(l.cmp(r)));
        }
    }
    Some(if left.len == right.len {
        TemporalOrder::Ordered(Ordering::Equal)
    } else {
        TemporalOrder::Unknown
    })
}
