//! `FHIRPath` `DateTime` values that FHIR's `dateTime` cannot hold. FHIR requires seconds and a
//! timezone offset whenever a `dateTime` has a time, while a `FHIRPath` `DateTime` may stop at
//! the hour or minute and may have no offset (`@2015-02-04T14:34`, `@2015-02-04T14:34:28`).
//! Only `FHIRPath` makes these, so they live here and not in the FHIR model; every other
//! `DateTime` the engine makes is a FHIR `dateTime`.

use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use haste_fhir_model::r4::datetime::{
    Date, DateTime, Time, parse_date, parse_datetime, parse_partial_time,
};
use haste_reflect::MetaValue;
use regex::Regex;
use std::{any::Any, fmt, sync::LazyLock};

/// What follows a `DateTime`'s `T`: a time, then `Z` or an offset in FHIR's range.
static TIME_AND_OFFSET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<time>[0-9][0-9:.]*?)(?<offset>Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00))?$")
        .unwrap()
});

/// A `FHIRPath` `DateTime` whose time FHIR's `dateTime` cannot hold.
#[derive(Debug, Clone, PartialEq)]
pub struct PartialDateTime {
    pub date: NaiveDate,
    pub time: Time,
    /// `None` when the value has no timezone offset.
    pub offset: Option<FixedOffset>,
}

/// A `FHIRPath` `DateTime`: a FHIR `dateTime` when FHIR's type can hold it.
pub enum AnyDateTime {
    Fhir(DateTime),
    Partial(PartialDateTime),
}

/// `text` as a `FHIRPath` `DateTime`, or `None` when it is not one.
pub fn parse_any_datetime(text: &str) -> Option<AnyDateTime> {
    match parse_datetime(text) {
        Ok(value) => Some(AnyDateTime::Fhir(value)),
        Err(_) => PartialDateTime::parse(text).map(AnyDateTime::Partial),
    }
}

/// The clock time a Time names, the parts below its precision at zero.
pub fn naive_time(time: &Time) -> NaiveTime {
    match time {
        Time::Hour(hour) => NaiveTime::from_hms_opt(u32::from(*hour), 0, 0),
        Time::HourMinute(hour, minute) => {
            NaiveTime::from_hms_opt(u32::from(*hour), u32::from(*minute), 0)
        }
        Time::HourMinuteSecond(time) => Some(*time),
    }
    .unwrap_or_default()
}

impl PartialDateTime {
    /// `text` as a `DateTime` with a time, a full date before it, or `None`.
    fn parse(text: &str) -> Option<Self> {
        let (date, time) = text.split_once('T')?;
        let captures = TIME_AND_OFFSET_REGEX.captures(time)?;
        let Date::YearMonthDay(year, month, day) = parse_date(date).ok()? else {
            return None;
        };
        // chrono's offset parser does not take `Z`; the regex keeps the rest in FHIR's range.
        let offset = match captures.name("offset").map(|offset| offset.as_str()) {
            None => None,
            Some("Z") => Some(FixedOffset::east_opt(0)?),
            Some(offset) => Some(offset.parse::<FixedOffset>().ok()?),
        };
        Some(PartialDateTime {
            date: NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))?,
            time: parse_partial_time(&captures["time"]).ok()?,
            offset,
        })
    }

    /// The date and clock time as written, the parts below the precision at zero.
    #[must_use]
    pub fn local(&self) -> NaiveDateTime {
        self.date.and_time(naive_time(&self.time))
    }

    /// The date and clock time in UTC, or as written when there is no offset.
    #[must_use]
    pub fn utc(&self) -> Option<NaiveDateTime> {
        match self.offset {
            Some(offset) => self.local().checked_sub_offset(offset),
            None => Some(self.local()),
        }
    }
}

impl fmt::Display for PartialDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}T{}",
            self.date.format("%Y-%m-%d"),
            self.time.to_string()
        )?;
        match self.offset {
            Some(offset) => write!(f, "{offset}"),
            None => Ok(()),
        }
    }
}

/// A `FHIRPath` `System.DateTime`: there is no FHIR type for it.
impl MetaValue for PartialDateTime {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn fhir_type(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.DateTime"
    }

    fn is_many(&self) -> bool {
        false
    }
}
