use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum DateTime {
    Year(u16),
    YearMonth(u16, u8),
    YearMonthDay(u16, u8, u8),
    Iso8601(chrono::DateTime<chrono::Utc>),
}

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

pub static DATETIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    let re = Regex::new(
        r"(?<year>[0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(?<month>0[1-9]|1[0-2])(-(?<day>0[1-9]|[1-2][0-9]|3[0-1])(T(?<time>[01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?",
    )
    .unwrap();

    re
});

pub fn parse_datetime(date_string: &str) -> Result<DateTime, ParseError> {
    if let Some(captures) = DATETIME_REGEX.captures(date_string) {
        println!("{:?}", captures);
        match (
            captures.name("year"),
            captures.name("month"),
            captures.name("day"),
            captures.name("time"),
        ) {
            (Some(year), None, None, None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                return Ok(DateTime::Year(year));
            }
            (Some(year), Some(month), None, None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                return Ok(DateTime::YearMonth(year, month));
            }
            (Some(year), Some(month), Some(day), None) => {
                let year = year.as_str().parse::<u16>().unwrap();
                let month = month.as_str().parse::<u8>().unwrap();
                let day = day.as_str().parse::<u8>().unwrap();
                return Ok(DateTime::YearMonthDay(year, month, day));
            }
            _ => {
                let datetime = chrono::DateTime::parse_from_rfc3339(date_string)
                    .map_err(|_| ParseError::InvalidFormat)?;
                return Ok(DateTime::Iso8601(datetime.with_timezone(&chrono::Utc)));
            }
        }
    } else {
        println!("No match found for date string: {}", date_string);
        return Err(ParseError::InvalidFormat);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(parse_datetime("2023-01-32").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00Z").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+00:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+01:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00-01:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+02:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00-02:00").is_ok());
        assert!(parse_datetime("2023-01-01T12:00:00+14:00").is_ok());
    }
}
