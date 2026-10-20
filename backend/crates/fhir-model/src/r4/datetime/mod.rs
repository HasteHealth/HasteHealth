use chrono::{Datelike, Days, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

mod reflect;
mod serialize;

#[cfg(test)]
mod proptests;

/// `year` as four digits. The integer fast path, padded only when short.
#[inline]
fn padded_year(year: u16) -> String {
    let mut text = year.to_string();
    if text.len() < 4 {
        text.insert_str(0, &"000"[..4 - text.len()]);
    }
    text
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateTime {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
    Iso8601(chrono::DateTime<chrono::Utc>),
}

impl ToString for DateTime {
    fn to_string(&self) -> String {
        match self {
            DateTime::Year(year) => padded_year(*year),
            DateTime::YearMonth(year, month) => format!("{:04}-{:02}", year, month),
            DateTime::YearMonthDay(year, month, day) => {
                format!("{:04}-{:02}-{:02}", year, month, day)
            }
            DateTime::Iso8601(dt) => dt.to_rfc3339(),
        }
    }
}

impl TryFrom<DateTime> for chrono::DateTime<chrono::Utc> {
    type Error = ParseError;

    fn try_from(value: DateTime) -> Result<Self, Self::Error> {
        match value {
            // "1996-12-19T16:39:57-08:00"
            DateTime::Year(year) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(
                    format!("{}-01-01T00:00:00Z", year).as_str(),
                )
                .map_err(|_| ParseError::InvalidFormat)?;

                Ok(datetime.with_timezone(&chrono::Utc))
            }
            DateTime::YearMonth(year, month) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(
                    format!("{}-{:02}-01T00:00:00Z", year, month).as_str(),
                )
                .map_err(|_| ParseError::InvalidFormat)?;

                Ok(datetime.with_timezone(&chrono::Utc))
            }
            DateTime::YearMonthDay(year, month, day) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(
                    format!("{}-{:02}-{:02}T00:00:00Z", year, month, day).as_str(),
                )
                .map_err(|_| ParseError::InvalidFormat)?;

                Ok(datetime.with_timezone(&chrono::Utc))
            }
            DateTime::Iso8601(dt) => Ok(dt),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Date {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
}

impl ToString for Date {
    fn to_string(&self) -> String {
        match self {
            Date::Year(year) => padded_year(*year),
            Date::YearMonth(year, month) => format!("{:04}-{:02}", year, month),
            Date::YearMonthDay(year, month, day) => {
                format!("{:04}-{:02}-{:02}", year, month, day)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instant {
    Iso8601(chrono::DateTime<chrono::Utc>),
}

impl Instant {
    pub fn format(&self, fmt: &str) -> String {
        match self {
            Instant::Iso8601(dt) => dt.to_utc().format(fmt).to_string(),
        }
    }
}

impl ToString for Instant {
    fn to_string(&self) -> String {
        match self {
            Instant::Iso8601(dt) => dt.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Time(chrono::NaiveTime);

impl ToString for Time {
    fn to_string(&self) -> String {
        self.0.format("%H:%M:%S%.f").to_string()
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
}

pub static DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<year>[0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(?<month>0[1-9]|1[0-2])(-(?<day>0[1-9]|[1-2][0-9]|3[0-1]))?)?$",
    ).unwrap()
});

pub static DATETIME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<year>[0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(?<month>0[1-9]|1[0-2])(-(?<day>0[1-9]|[1-2][0-9]|3[0-1])(?<time>T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?$",
    ).unwrap()
});

pub static INSTANT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)-(0[1-9]|1[0-2])-(0[1-9]|[1-2][0-9]|3[0-1])T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00))$",
    ).unwrap()
});

pub static TIME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?$").unwrap()
});

/// A capture the regex matched as digits.
#[inline]
fn captured<T: std::str::FromStr>(capture: regex::Match<'_>) -> T
where
    T::Err: std::fmt::Debug,
{
    capture.as_str().parse().expect("the regex matched digits")
}

/// The calendar date `day` of `month` names. The regexes admit any day up
/// to 31; a day the month lacks carries into the next month as chrono
/// counts from the first: `2023-02-30` is `2023-03-02`, `2024-06-31` is
/// `2024-07-01`. Every month has 28 days, so only a later day is checked.
#[inline]
fn calendar_date(year: u16, month: u8, day: u8) -> Result<(u16, u8, u8), ParseError> {
    let (y, m) = (i32::from(year), u32::from(month));
    if day <= 28 || NaiveDate::from_ymd_opt(y, m, u32::from(day)).is_some() {
        return Ok((year, month, day));
    }
    let carried = NaiveDate::from_ymd_opt(y, m, 1)
        .and_then(|first| first.checked_add_days(Days::new(u64::from(day).saturating_sub(1))))
        .ok_or(ParseError::InvalidFormat)?;
    Ok((
        u16::try_from(carried.year()).map_err(|_| ParseError::InvalidFormat)?,
        u8::try_from(carried.month()).map_err(|_| ParseError::InvalidFormat)?,
        u8::try_from(carried.day()).map_err(|_| ParseError::InvalidFormat)?,
    ))
}

/// RFC 3339 text a regex matched, parsed. Chrono rejects a day the month
/// lacks, so that text is parsed again with the day carried over
/// ([`calendar_date`]); the regex fixes the date at ten characters.
#[inline]
fn parse_rfc3339(text: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ParseError> {
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(text) {
        return Ok(datetime);
    }
    let (date, time) = text.split_at(10);
    let (year, rest) = date.split_once('-').ok_or(ParseError::InvalidFormat)?;
    let (month, day) = rest.split_once('-').ok_or(ParseError::InvalidFormat)?;
    let (year, month, day) = calendar_date(
        year.parse().map_err(|_| ParseError::InvalidFormat)?,
        month.parse().map_err(|_| ParseError::InvalidFormat)?,
        day.parse().map_err(|_| ParseError::InvalidFormat)?,
    )?;
    chrono::DateTime::parse_from_rfc3339(&format!("{year:04}-{month:02}-{day:02}{time}"))
        .map_err(|_| ParseError::InvalidFormat)
}

pub fn parse_instant(instant_string: &str) -> Result<Instant, ParseError> {
    if INSTANT_REGEX.is_match(instant_string) {
        let datetime = parse_rfc3339(instant_string)?;
        Ok(Instant::Iso8601(datetime.with_timezone(&chrono::Utc)))
    } else {
        Err(ParseError::InvalidFormat)
    }
}

pub fn parse_time(time_string: &str) -> Result<Time, ParseError> {
    if TIME_REGEX.is_match(time_string) {
        let time = Time(
            chrono::NaiveTime::parse_from_str(time_string, "%H:%M:%S%.f")
                .map_err(|_| ParseError::InvalidFormat)?,
        );
        Ok(time)
    } else {
        Err(ParseError::InvalidFormat)
    }
}

pub fn parse_datetime(datetime_string: &str) -> Result<DateTime, ParseError> {
    if let Some(captures) = DATETIME_REGEX.captures(datetime_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
            captures.name("time"),
        ) {
            (Some(year), None, None, None) => Ok(DateTime::Year(captured(year))),
            (Some(year), Some(month), None, None) => {
                Ok(DateTime::YearMonth(captured(year), captured(month)))
            }
            (Some(year), Some(month), Some(day), None) => {
                let (year, month, day) =
                    calendar_date(captured(year), captured(month), captured(day))?;
                Ok(DateTime::YearMonthDay(year, month, day))
            }
            (Some(_), Some(_), Some(_), Some(_)) => {
                let datetime = parse_rfc3339(datetime_string)?;
                Ok(DateTime::Iso8601(datetime.with_timezone(&chrono::Utc)))
            }
            _ => Err(ParseError::InvalidFormat),
        }
    } else {
        Err(ParseError::InvalidFormat)
    }
}

pub fn parse_date(date_string: &str) -> Result<Date, ParseError> {
    if let Some(captures) = DATE_REGEX.captures(date_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
        ) {
            (Some(year), None, None) => Ok(Date::Year(captured(year))),
            (Some(year), Some(month), None) => Ok(Date::YearMonth(captured(year), captured(month))),
            (Some(year), Some(month), Some(day)) => {
                let (year, month, day) =
                    calendar_date(captured(year), captured(month), captured(day))?;
                Ok(Date::YearMonthDay(year, month, day))
            }
            _ => Err(ParseError::InvalidFormat),
        }
    } else {
        Err(ParseError::InvalidFormat)
    }
}

pub enum DateKind {
    DateTime,
    Date,
    Time,
    Instant,
}

pub enum DateResult {
    DateTime(DateTime),
    Date(Date),
    Time(Time),
    Instant(Instant),
}

pub fn parse(kind: DateKind, input: &str) -> Result<DateResult, ParseError> {
    match kind {
        DateKind::DateTime => Ok(DateResult::DateTime(parse_datetime(input)?)),
        DateKind::Date => Ok(DateResult::Date(parse_date(input)?)),
        DateKind::Time => Ok(DateResult::Time(parse_time(input)?)),
        DateKind::Instant => Ok(DateResult::Instant(parse_instant(input)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert!(parse_time("12:34:56").is_ok());
        assert!(parse_time("23:59:59").is_ok());
        assert!(parse_time("23:59:59.232").is_ok());
        assert_eq!(
            parse_time("23:59:59.232").unwrap(),
            Time(chrono::NaiveTime::from_hms_milli_opt(23, 59, 59, 232).unwrap())
        );
    }

    #[test]
    fn test_parse_instant() {
        assert!(parse_instant("2015-02-07T13:28:17.239+02:00").is_ok());
        assert!(parse_instant("2017-01-01T00:00:00Z").is_ok());
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2023").unwrap(), Date::Year(2023));
        assert_eq!(parse_date("2023-01").unwrap(), Date::YearMonth(2023, 1));
        assert_eq!(
            parse_date("2023-01-01").unwrap(),
            Date::YearMonthDay(2023, 1, 1)
        );

        assert_eq!(
            Date::YearMonthDay(2023, 1, 19),
            parse_date("2023-01-19").unwrap()
        );

        assert!(parse_date("2023-01-33").is_err());
        assert!(parse_date("2023-13-30").is_err());
        assert!(parse_date("2023-01-01T12:00:00Z").is_err());
    }
    #[test]
    fn test_parse_datetime() {
        assert_eq!(parse_datetime("2023").unwrap(), DateTime::Year(2023));
        assert_eq!(
            parse_datetime("2023-01").unwrap(),
            DateTime::YearMonth(2023, 1)
        );
        assert_eq!(
            parse_datetime("2023-01-01").unwrap(),
            DateTime::YearMonthDay(2023, 1, 1)
        );

        assert_eq!(
            DateTime::YearMonthDay(2023, 1, 19),
            parse_datetime("2023-01-19").unwrap()
        );

        // Invalid day won't parse.
        assert!(parse_datetime("2023-01-42").is_err());

        assert_eq!(
            parse_datetime("2023-01-01T12:00:00Z").unwrap(),
            DateTime::Iso8601(
                chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc)
            )
        );
        assert!(parse_datetime("2023-01-01T12:00:00+00:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+01:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00-01:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+02:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00-02:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+14:00").is_ok());
    }
}
