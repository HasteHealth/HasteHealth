//! <https://hl7.org/fhirpath/N1/#datetime-arithmetic>: a time-valued Quantity
//! added to or subtracted from a Date, `DateTime` or Time, at the value's
//! precision.

use crate::{error::OperationError, partial_datetime::PartialDateTime};
use chrono::{Months, NaiveDate, NaiveTime, TimeDelta, Timelike};
use haste_fhir_model::r4::{
    datetime::{Date, DateTime, Time},
    generated::types::{FHIRDate, FHIRDateTime, FHIRTime},
};
use haste_reflect::MetaValue;

const MILLIS_PER_MINUTE: i64 = 60_000;
const MILLIS_PER_HOUR: i64 = 60 * MILLIS_PER_MINUTE;

/// A signed duration in the coarsest family its unit belongs to.
#[derive(Clone, Copy)]
enum Duration {
    Months(i64),
    Days(i64),
    Millis(i64),
}

/// The duration of `amount` `unit`s, or `None` when it overflows. Above
/// seconds the decimal part is ignored, as such arithmetic is in whole units.
/// A unit with no fixed length (`'mo'`, `'a'`) or no time unit at all is an
/// error.
#[allow(clippy::cast_possible_truncation)]
fn duration(amount: f64, unit: &str) -> Result<Option<Duration>, OperationError> {
    let whole = amount.trunc() as i64;
    let fixed = |factor: i64| whole.checked_mul(factor);
    let duration = match unit {
        "year" | "years" => fixed(12).map(Duration::Months),
        "month" | "months" => Some(Duration::Months(whole)),
        "week" | "weeks" | "wk" => fixed(7).map(Duration::Days),
        "day" | "days" | "d" => Some(Duration::Days(whole)),
        "hour" | "hours" | "h" => fixed(MILLIS_PER_HOUR).map(Duration::Millis),
        "minute" | "minutes" | "min" => fixed(MILLIS_PER_MINUTE).map(Duration::Millis),
        "second" | "seconds" | "s" => Some(Duration::Millis((amount * 1000.0).round() as i64)),
        "millisecond" | "milliseconds" | "ms" => Some(Duration::Millis(amount.round() as i64)),
        _ => return Err(OperationError::InvalidTimeUnit(unit.to_string())),
    };
    Ok(duration.filter(|_| amount.is_finite()))
}

/// The shifted value, of the same type and precision as the input.
pub enum Shifted {
    Date(Date),
    DateTime(DateTime),
    PartialDateTime(PartialDateTime),
    Time(Time),
}

/// A Date, `DateTime` or Time item.
pub enum Temporal<'v> {
    Date(&'v Date),
    DateTime(&'v DateTime),
    PartialDateTime(&'v PartialDateTime),
    Time(&'v Time),
}

impl<'v> Temporal<'v> {
    pub fn of(value: &'v dyn MetaValue) -> Option<Self> {
        let any = value.as_any();
        if let Some(date) = any.downcast_ref::<FHIRDate>() {
            date.value.as_ref().map(Temporal::Date)
        } else if let Some(datetime) = any.downcast_ref::<FHIRDateTime>() {
            datetime.value.as_ref().map(Temporal::DateTime)
        } else if let Some(datetime) = any.downcast_ref::<PartialDateTime>() {
            Some(Temporal::PartialDateTime(datetime))
        } else {
            any.downcast_ref::<FHIRTime>()?
                .value
                .as_ref()
                .map(Temporal::Time)
        }
    }

    /// Adds `amount` `unit`s, or subtracts them. An error when the unit is not
    /// a time unit of fixed length; `None` (empty) when the value's precision
    /// cannot take it or the result is out of range.
    pub fn shift(
        &self,
        amount: f64,
        unit: &str,
        subtract: bool,
    ) -> Result<Option<Shifted>, OperationError> {
        let Some(duration) = duration(if subtract { -amount } else { amount }, unit)? else {
            return Ok(None);
        };
        Ok(match self {
            Temporal::Date(date) => shift_date(date, duration).map(Shifted::Date),
            Temporal::DateTime(datetime) => {
                shift_datetime(datetime, duration).map(Shifted::DateTime)
            }
            Temporal::PartialDateTime(datetime) => {
                shift_partial_datetime(datetime, duration).map(Shifted::PartialDateTime)
            }
            Temporal::Time(time) => shift_time(time, duration).map(Shifted::Time),
        })
    }
}

/// `date` moved by whole `months`, the day clamped to the month's end.
fn add_months(date: NaiveDate, months: i64) -> Option<NaiveDate> {
    let count = Months::new(u32::try_from(months.unsigned_abs()).ok()?);
    if months >= 0 {
        date.checked_add_months(count)
    } else {
        date.checked_sub_months(count)
    }
}

fn year(year: i64) -> Option<u16> {
    u16::try_from(year)
        .ok()
        .filter(|year| (1..=9999).contains(year))
}

fn shift_date(date: &Date, duration: Duration) -> Option<Date> {
    match (date, duration) {
        (Date::Year(y), Duration::Months(months)) => {
            year(i64::from(*y) + months / 12).map(Date::Year)
        }
        (Date::YearMonth(y, m), Duration::Months(months)) => {
            let total = i64::from(*y) * 12 + i64::from(*m) - 1 + months;
            let month = u8::try_from(total.rem_euclid(12) + 1).ok()?;
            Some(Date::YearMonth(year(total.div_euclid(12))?, month))
        }
        (Date::YearMonthDay(y, m, d), duration) => {
            let date = NaiveDate::from_ymd_opt(i32::from(*y), u32::from(*m), u32::from(*d))?;
            let shifted = match duration {
                Duration::Months(months) => add_months(date, months)?,
                Duration::Days(days) => date.checked_add_signed(TimeDelta::try_days(days)?)?,
                Duration::Millis(_) => return None,
            };
            Some(Date::YearMonthDay(
                year(i64::from(chrono::Datelike::year(&shifted)))?,
                u8::try_from(chrono::Datelike::month(&shifted)).ok()?,
                u8::try_from(chrono::Datelike::day(&shifted)).ok()?,
            ))
        }
        _ => None,
    }
}

fn shift_datetime(datetime: &DateTime, duration: Duration) -> Option<DateTime> {
    let partial = match datetime {
        DateTime::Year(y) => Date::Year(*y),
        DateTime::YearMonth(y, m) => Date::YearMonth(*y, *m),
        DateTime::YearMonthDay(y, m, d) => Date::YearMonthDay(*y, *m, *d),
        DateTime::Iso8601(instant) => {
            let shifted = match duration {
                Duration::Months(months) => {
                    let count = Months::new(u32::try_from(months.unsigned_abs()).ok()?);
                    if months >= 0 {
                        instant.checked_add_months(count)
                    } else {
                        instant.checked_sub_months(count)
                    }
                }
                Duration::Days(days) => instant.checked_add_signed(TimeDelta::try_days(days)?),
                Duration::Millis(millis) => {
                    instant.checked_add_signed(TimeDelta::try_milliseconds(millis)?)
                }
            };
            return shifted.map(DateTime::Iso8601);
        }
    };
    Some(match shift_date(&partial, duration)? {
        Date::Year(y) => DateTime::Year(y),
        Date::YearMonth(y, m) => DateTime::YearMonth(y, m),
        Date::YearMonthDay(y, m, d) => DateTime::YearMonthDay(y, m, d),
    })
}

/// The time moves by `duration` and is cut back to the value's precision.
fn shift_partial_datetime(
    datetime: &PartialDateTime,
    duration: Duration,
) -> Option<PartialDateTime> {
    let local = datetime.local();
    let shifted = match duration {
        Duration::Months(months) => {
            let count = Months::new(u32::try_from(months.unsigned_abs()).ok()?);
            if months >= 0 {
                local.checked_add_months(count)
            } else {
                local.checked_sub_months(count)
            }
        }
        Duration::Days(days) => local.checked_add_signed(TimeDelta::try_days(days)?),
        Duration::Millis(millis) => local.checked_add_signed(TimeDelta::try_milliseconds(millis)?),
    }?;
    // A year outside 1 to 9999 is empty, as for the other precisions.
    year(i64::from(chrono::Datelike::year(&shifted)))?;
    let time = match datetime.time {
        Time::Hour(_) => Time::Hour(u8::try_from(shifted.hour()).ok()?),
        Time::HourMinute(..) => Time::HourMinute(
            u8::try_from(shifted.hour()).ok()?,
            u8::try_from(shifted.minute()).ok()?,
        ),
        Time::HourMinuteSecond(_) => Time::HourMinuteSecond(shifted.time()),
    };
    Some(PartialDateTime {
        date: shifted.date(),
        time,
        offset: datetime.offset,
    })
}

/// A clock time wraps past midnight and is cut back to the value's precision.
fn shift_time(time: &Time, duration: Duration) -> Option<Time> {
    let Duration::Millis(millis) = duration else {
        return None;
    };
    let delta = TimeDelta::try_milliseconds(millis)?;
    let moved = |base: Option<NaiveTime>| Some(base?.overflowing_add_signed(delta).0);
    match time {
        Time::Hour(h) => {
            let moved = moved(NaiveTime::from_hms_opt(u32::from(*h), 0, 0))?;
            Some(Time::Hour(u8::try_from(moved.hour()).ok()?))
        }
        Time::HourMinute(h, m) => {
            let moved = moved(NaiveTime::from_hms_opt(u32::from(*h), u32::from(*m), 0))?;
            Some(Time::HourMinute(
                u8::try_from(moved.hour()).ok()?,
                u8::try_from(moved.minute()).ok()?,
            ))
        }
        Time::HourMinuteSecond(t) => moved(Some(*t)).map(Time::HourMinuteSecond),
    }
}
