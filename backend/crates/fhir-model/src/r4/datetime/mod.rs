use regex::Regex;
use std::fmt;
use std::sync::LazyLock;

mod reflect;
mod serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum DateTime {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
    Iso8601(chrono::DateTime<chrono::Utc>),
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateTime::Year(year) => write!(f, "{year}"),
            DateTime::YearMonth(year, month) => {
                write!(f, "{year:04}-{month:02}")
            }
            DateTime::YearMonthDay(year, month, day) => {
                write!(f, "{year:04}-{month:02}-{day:02}")
            }
            DateTime::Iso8601(dt) => {
                write!(f, "{}", dt.to_rfc3339())
            }
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
                    format!("{year}-01-01T00:00:00Z").as_str(),
                )
                .map_err(|_| ParseError::InvalidFormat)?;

                Ok(datetime.with_timezone(&chrono::Utc))
            }
            DateTime::YearMonth(year, month) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(
                    format!("{year}-{month:02}-01T00:00:00Z").as_str(),
                )
                .map_err(|_| ParseError::InvalidFormat)?;

                Ok(datetime.with_timezone(&chrono::Utc))
            }
            DateTime::YearMonthDay(year, month, day) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(
                    format!("{year}-{month:02}-{day:02}T00:00:00Z").as_str(),
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

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Date::Year(year) => write!(f, "{year}"),
            Date::YearMonth(year, month) => {
                write!(f, "{year:04}-{month:02}")
            }
            Date::YearMonthDay(year, month, day) => {
                write!(f, "{year:04}-{month:02}-{day:02}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instant {
    Iso8601(chrono::DateTime<chrono::Utc>),
}

impl Instant {
    #[must_use]
    pub fn format(&self, fmt: &str) -> String {
        match self {
            Instant::Iso8601(dt) => dt.to_utc().format(fmt).to_string(),
        }
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instant::Iso8601(dt) => write!(f, "{}", dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Time(chrono::NaiveTime);

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%H:%M:%S%.f"))
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

/// Parses an FHIR instant string into an [`Instant`].
///
/// The input must conform to the FHIR instant format and contain a valid
/// RFC 3339 timestamp.
///
/// # Errors
///
/// Returns [`ParseError::InvalidFormat`] if the input does not match the
/// expected instant format or cannot be parsed as a valid RFC 3339 datetime.
pub fn parse_instant(instant_string: &str) -> Result<Instant, ParseError> {
    if INSTANT_REGEX.is_match(instant_string) {
        let datetime = chrono::DateTime::parse_from_rfc3339(instant_string)
            .map_err(|_| ParseError::InvalidFormat)?;
        Ok(Instant::Iso8601(datetime.with_timezone(&chrono::Utc)))
    } else {
        Err(ParseError::InvalidFormat)
    }
}

/// Parses an FHIR time string into a [`Time`].
///
/// The input must conform to the FHIR time format and is parsed using the
/// `HH:MM:SS` format with optional fractional seconds.
///
/// # Errors
///
/// Returns [`ParseError::InvalidFormat`] if the input does not match the
/// expected time format or cannot be parsed as a valid time value.
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

fn parse_u16(value: &str) -> Result<u16, ParseError> {
    value.parse::<u16>().map_err(|_| ParseError::InvalidFormat)
}

fn parse_u8(value: &str) -> Result<u8, ParseError> {
    value.parse::<u8>().map_err(|_| ParseError::InvalidFormat)
}

/// Parses an FHIR date/time string into a [`DateTime`].
///
/// Supports FHIR date precision levels including year, year-month, and
/// year-month-day values, as well as full RFC 3339 date-time values.
///
/// # Errors
///
/// Returns [`ParseError::InvalidFormat`] if the input does not match a
/// supported FHIR date/time format or cannot be parsed as a valid RFC 3339
/// datetime.
pub fn parse_datetime(datetime_string: &str) -> Result<DateTime, ParseError> {
    if let Some(captures) = DATETIME_REGEX.captures(datetime_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
            captures.name("time"),
        ) {
            (Some(year), None, None, None) => Ok(DateTime::Year(parse_u16(year.as_str())?)),

            (Some(year), Some(month), None, None) => Ok(DateTime::YearMonth(
                parse_u16(year.as_str())?,
                parse_u8(month.as_str())?,
            )),

            (Some(year), Some(month), Some(day), None) => Ok(DateTime::YearMonthDay(
                parse_u16(year.as_str())?,
                parse_u8(month.as_str())?,
                parse_u8(day.as_str())?,
            )),

            _ => {
                let datetime = chrono::DateTime::parse_from_rfc3339(datetime_string)
                    .map_err(|_| ParseError::InvalidFormat)?;

                Ok(DateTime::Iso8601(datetime.with_timezone(&chrono::Utc)))
            }
        }
    } else {
        Err(ParseError::InvalidFormat)
    }
}

fn parse_year(value: &str) -> Result<u16, ParseError> {
    value.parse::<u16>().map_err(|_| ParseError::InvalidFormat)
}

fn parse_month_or_day(value: &str) -> Result<u8, ParseError> {
    value.parse::<u8>().map_err(|_| ParseError::InvalidFormat)
}

/// Parses an FHIR date string into a [`Date`].
///
/// Supports FHIR date precision levels including year, year-month, and
/// year-month-day values.
///
/// # Errors
///
/// Returns [`ParseError::InvalidFormat`] if the input does not match a
/// supported FHIR date format or if any date component cannot be parsed.
pub fn parse_date(date_string: &str) -> Result<Date, ParseError> {
    if let Some(captures) = DATE_REGEX.captures(date_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
        ) {
            (Some(year), None, None) => Ok(Date::Year(parse_year(year.as_str())?)),

            (Some(year), Some(month), None) => Ok(Date::YearMonth(
                parse_year(year.as_str())?,
                parse_month_or_day(month.as_str())?,
            )),

            (Some(year), Some(month), Some(day)) => Ok(Date::YearMonthDay(
                parse_year(year.as_str())?,
                parse_month_or_day(month.as_str())?,
                parse_month_or_day(day.as_str())?,
            )),

            _ => Err(ParseError::InvalidFormat),
        }
    } else {
        Err(ParseError::InvalidFormat)
    }
}

#[derive(Clone, Copy)]
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

/// Parses an input string into a date-related value based on the provided
/// [`DateKind`].
///
/// The parser delegates to the appropriate function depending on the requested
/// kind:
/// - [`DateKind::DateTime`] uses [`parse_datetime`].
/// - [`DateKind::Date`] uses [`parse_date`].
/// - [`DateKind::Time`] uses [`parse_time`].
/// - [`DateKind::Instant`] uses [`parse_instant`].
///
/// # Errors
///
/// Returns [`ParseError`] if the input does not match the expected format for
/// the requested [`DateKind`].
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
