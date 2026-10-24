//! Date/time arithmetic with time-valued Quantities, per
//! <https://hl7.org/fhirpath/N1/#datetime-arithmetic>.

// Durations are range-checked before they are cast.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use chrono::{Datelike, Days, Months, NaiveDate, TimeDelta};
use haste_fhir_model::r4::datetime::{Date, DateTime, Time};

const MS_PER_DAY: i64 = 86_400_000;
const MS_PER_HOUR: i64 = 3_600_000;
const MS_PER_MINUTE: i64 = 60_000;

/// A calendar duration. Years and months are calendar-relative; the rest are
/// fixed lengths in milliseconds.
#[derive(Clone, Copy)]
pub enum Duration {
    Months(i64),
    Millis(i64),
}

impl Duration {
    /// The duration `value unit` names, `None` for any other unit. Above seconds
    /// the decimal part is ignored, as calendar arithmetic is whole-unit.
    pub fn new(value: f64, unit: &str) -> Option<Self> {
        let whole = value.trunc();
        let (amount, factor) = match unit {
            "year" | "years" => return Some(Self::Months(to_i64(whole)?.checked_mul(12)?)),
            "month" | "months" => return Some(Self::Months(to_i64(whole)?)),
            "week" | "weeks" | "wk" => (whole, 7 * MS_PER_DAY),
            "day" | "days" | "d" => (whole, MS_PER_DAY),
            "hour" | "hours" | "h" => (whole, MS_PER_HOUR),
            "minute" | "minutes" | "min" => (whole, MS_PER_MINUTE),
            "second" | "seconds" | "s" => (value, 1000),
            "millisecond" | "milliseconds" | "ms" => (value, 1),
            _ => return None,
        };
        // Rounded so `1.5 seconds` keeps its milliseconds.
        to_i64((amount * factor as f64).round()).map(Self::Millis)
    }

    pub fn negate(self) -> Option<Self> {
        match self {
            Self::Months(n) => n.checked_neg().map(Self::Months),
            Self::Millis(n) => n.checked_neg().map(Self::Millis),
        }
    }
}

/// `value` as an `i64`, `None` for NaN or out of range.
fn to_i64(value: f64) -> Option<i64> {
    (value.is_finite() && value.abs() < i64::MAX as f64).then_some(value as i64)
}

fn shift_ymd(date: NaiveDate, duration: Duration) -> Option<NaiveDate> {
    match duration {
        Duration::Months(n) => {
            let months = Months::new(u32::try_from(n.unsigned_abs()).ok()?);
            if n < 0 {
                date.checked_sub_months(months)
            } else {
                date.checked_add_months(months)
            }
        }
        Duration::Millis(ms) => {
            let days = Days::new((ms / MS_PER_DAY).unsigned_abs());
            if ms < 0 {
                date.checked_sub_days(days)
            } else {
                date.checked_add_days(days)
            }
        }
    }
}

/// A year and month moved by `months`, `None` outside years 1 to 9999.
fn shift_year_month(year: u16, month: u8, months: i64) -> Option<(u16, u8)> {
    let total = i64::from(year) * 12 + i64::from(month) - 1 + months;
    let year = u16::try_from(total.div_euclid(12))
        .ok()
        .filter(|y| *y > 0 && *y <= 9999)?;
    Some((year, u8::try_from(total.rem_euclid(12)).ok()? + 1))
}

/// Moves a partial date, keeping its precision. A duration finer than the
/// precision is truncated to it, so it may not move the date at all.
fn shift_partial(
    year: u16,
    month: Option<u8>,
    day: Option<u8>,
    duration: Duration,
) -> Option<(u16, Option<u8>, Option<u8>)> {
    match (month, day, duration) {
        (None, _, Duration::Months(n)) => {
            let (year, _) = shift_year_month(year, 1, n / 12 * 12)?;
            Some((year, None, None))
        }
        (Some(month), None, Duration::Months(n)) => {
            let (year, month) = shift_year_month(year, month, n)?;
            Some((year, Some(month), None))
        }
        (Some(month), Some(day), _) => {
            let date = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())?;
            let date =
                shift_ymd(date, duration).filter(|date| (1..=9999).contains(&date.year()))?;
            Some((
                u16::try_from(date.year()).ok()?,
                Some(u8::try_from(date.month()).ok()?),
                Some(u8::try_from(date.day()).ok()?),
            ))
        }
        _ => Some((year, month, day)),
    }
}

pub fn shift_date(date: &Date, duration: Duration) -> Option<Date> {
    let (year, month, day) = match *date {
        Date::Year(y) => (y, None, None),
        Date::YearMonth(y, m) => (y, Some(m), None),
        Date::YearMonthDay(y, m, d) => (y, Some(m), Some(d)),
    };
    Some(match shift_partial(year, month, day, duration)? {
        (y, None, _) => Date::Year(y),
        (y, Some(m), None) => Date::YearMonth(y, m),
        (y, Some(m), Some(d)) => Date::YearMonthDay(y, m, d),
    })
}

pub fn shift_date_time(date_time: &DateTime, duration: Duration) -> Option<DateTime> {
    let date = match *date_time {
        DateTime::Year(y) => Date::Year(y),
        DateTime::YearMonth(y, m) => Date::YearMonth(y, m),
        DateTime::YearMonthDay(y, m, d) => Date::YearMonthDay(y, m, d),
        DateTime::Iso8601(instant) => {
            return match duration {
                Duration::Months(n) => {
                    let months = Months::new(u32::try_from(n.unsigned_abs()).ok()?);
                    if n < 0 {
                        instant.checked_sub_months(months)
                    } else {
                        instant.checked_add_months(months)
                    }
                }
                Duration::Millis(ms) => {
                    instant.checked_add_signed(TimeDelta::try_milliseconds(ms)?)
                }
            }
            .map(DateTime::Iso8601);
        }
    };
    Some(match shift_date(&date, duration)? {
        Date::Year(y) => DateTime::Year(y),
        Date::YearMonth(y, m) => DateTime::YearMonth(y, m),
        Date::YearMonthDay(y, m, d) => DateTime::YearMonthDay(y, m, d),
    })
}

/// Moves a time, wrapping past midnight. Months have no meaning for a time.
pub fn shift_time(time: &Time, duration: Duration) -> Option<Time> {
    let Duration::Millis(ms) = duration else {
        return None;
    };
    Some(match *time {
        Time::Hour(h) => {
            let hours = (i64::from(h) + ms / MS_PER_HOUR).rem_euclid(24);
            Time::Hour(u8::try_from(hours).ok()?)
        }
        Time::HourMinute(h, m) => {
            let minutes = (i64::from(h) * 60 + i64::from(m) + ms / MS_PER_MINUTE).rem_euclid(1440);
            Time::HourMinute(
                u8::try_from(minutes / 60).ok()?,
                u8::try_from(minutes % 60).ok()?,
            )
        }
        Time::HourMinuteSecond(t) => {
            Time::HourMinuteSecond(t.overflowing_add_signed(TimeDelta::try_milliseconds(ms)?).0)
        }
    })
}
