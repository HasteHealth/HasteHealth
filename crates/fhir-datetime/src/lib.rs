use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum DateTime {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
    Iso8601(chrono::DateTime<chrono::Utc>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Date {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
}

pub enum Instant {
    Iso8601(chrono::DateTime<chrono::Utc>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
}

pub static DATE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?<year>[0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(?<month>0[1-9]|1[0-2])(-(?<day>0[1-9]|[1-2][0-9]|3[0-1]))?)?$",
    ).unwrap()
});

pub static DATETIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?<year>[0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(?<month>0[1-9]|1[0-2])(-(?<day>0[1-9]|[1-2][0-9]|3[0-1])(?<time>T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?$",
    ).unwrap()
});

pub fn parse_datetime(datetime_string: &str) -> Result<DateTime, ParseError> {
    if let Some(captures) = DATETIME_REGEX.captures(datetime_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
            captures.name("time"),
        ) {
            (Some(year), None, None, None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                Ok(DateTime::Year(year))
            }
            (Some(year), Some(month), None, None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                Ok(DateTime::YearMonth(year, month))
            }
            (Some(year), Some(month), Some(day), None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                let day = day.as_str().parse::<u8>().unwrap();
                Ok(DateTime::YearMonthDay(year, month, day))
            }
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

pub fn parse_date(date_string: &str) -> Result<Date, ParseError> {
    if let Some(captures) = DATE_REGEX.captures(date_string) {
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
        ) {
            (Some(year), None, None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                Ok(Date::Year(year))
            }
            (Some(year), Some(month), None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                Ok(Date::YearMonth(year, month))
            }
            (Some(year), Some(month), Some(day)) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                let day = day.as_str().parse::<u8>().unwrap();
                Ok(Date::YearMonthDay(year, month, day))
            }
            _ => Err(ParseError::InvalidFormat),
        }
    } else {
        Err(ParseError::InvalidFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
