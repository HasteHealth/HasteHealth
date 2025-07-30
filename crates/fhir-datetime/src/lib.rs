// ([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)
// (-(0[1-9]|1[0-2])(-(0[1-9]|[1-2][0-9]|3[0-1])
// (T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?

use regex::Regex;

// YYYY, YYYY-MM, YYYY-MM-DD or YYYY-MM-DDThh:mm:ss+zz:zz
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

pub enum ParseError {
    InvalidFormat,
}

pub fn parse_datetime(date_string: &str) -> Result<DateTime, ParseError> {
    let re = Regex::new(
        r"([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(0[1-9]|1[0-2])(-(0[1-9]|[1-2][0-9]|3[0-1])(T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?
",
    )
    .unwrap();

    panic!();
}
