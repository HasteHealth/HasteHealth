/// Reference of conversions found here https://www.hl7.org/fhir/R4/search.html#table
use oxidized_fhir_model::r4::{
    datetime::{Date, DateTime, Instant},
    generated::{
        resources::{ResourceType, ResourceTypeError, SearchParameter},
        terminology::SearchParamType,
        types::{
            Address, Age, CodeableConcept, Coding, ContactPoint, Duration, FHIRBoolean,
            FHIRCanonical, FHIRDate, FHIRDateTime, FHIRDecimal, FHIRId, FHIRInstant, FHIRInteger,
            FHIRMarkdown, FHIRPositiveInt, FHIRString, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid,
            HumanName, Identifier, Money, Period, Quantity, Range, Reference, Timing,
        },
    },
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_reflect::MetaValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenIndex {
    system: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RangeValue {
    Number(f64),
    Infinity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuantityRange {
    start_value: RangeValue,
    start_code: Option<String>,
    start_system: Option<String>,
    end_value: RangeValue,
    end_code: Option<String>,
    end_system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DateRange {
    /// Milliseconds since epoch.
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceIndex {
    id: Option<String>,
    resource_type: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InsertableIndex {
    Meta(String),
    String(Vec<String>),
    Number(Vec<f64>),
    URI(Vec<String>),
    Token(Vec<TokenIndex>),
    Date(Vec<DateRange>),
    Reference(Vec<ReferenceIndex>),
    Quantity(Vec<QuantityRange>),
    Composite(Vec<String>),
    Special(Vec<String>),
}

#[derive(OperationOutcomeError, Debug)]
pub enum InsertableIndexError {
    #[fatal(
        code = "exception",
        diagnostic = "Invalid type for insertable index: '{arg0}'"
    )]
    InvalidType(String),
    #[fatal(
        code = "exception",
        diagnostic = "Failed to downcast value to number: {arg0}"
    )]
    FailedDowncast(String),
    #[fatal(
        code = "exception",
        diagnostic = "Reference contains invalid resource type."
    )]
    ResourceTypeError(#[from] ResourceTypeError),
}

// "http://hl7.org/fhirpath/System.String" => value
//     .as_any()
//     .downcast_ref::<String>()
//     .map(|v| vec![v.clone()])
//     .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),

fn convert_fp_string(value: &FHIRString) -> Vec<String> {
    value
        .value
        .as_ref()
        .map(|v| vec![v.to_string()])
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string(value: &Option<Box<FHIRString>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| convert_fp_string(v))
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string_vec(value: &Option<Vec<Box<FHIRString>>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| v.iter().flat_map(|s| convert_fp_string(s)).collect())
        .unwrap_or_else(|| vec![])
}

fn index_string(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRString" => {
            let fp_string = value.as_any().downcast_ref::<FHIRString>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_string
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        // Even though spec states won't encounter this it does. [ImplementationGuide.description]
        "FHIRMarkdown" => {
            let fp_markdown = value
                .as_any()
                .downcast_ref::<FHIRMarkdown>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_markdown
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "HumanName" => {
            let human_name = value.as_any().downcast_ref::<HumanName>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            let mut string_index = vec![];
            string_index.extend(convert_optional_fp_string(&human_name.text));
            string_index.extend(convert_optional_fp_string(&human_name.family));
            string_index.extend(convert_optional_fp_string_vec(&human_name.given));
            string_index.extend(convert_optional_fp_string_vec(&human_name.prefix));
            string_index.extend(convert_optional_fp_string_vec(&human_name.suffix));
            Ok(string_index)
        }
        "Address" => {
            let mut string_index = vec![];
            let address = value.as_any().downcast_ref::<Address>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            string_index.extend(convert_optional_fp_string(&address.text));
            string_index.extend(convert_optional_fp_string_vec(&address.line));
            string_index.extend(convert_optional_fp_string(&address.city));
            string_index.extend(convert_optional_fp_string(&address.district));
            string_index.extend(convert_optional_fp_string(&address.state));
            string_index.extend(convert_optional_fp_string(&address.postalCode));
            string_index.extend(convert_optional_fp_string(&address.country));

            Ok(string_index)
        }

        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_number(value: &dyn MetaValue) -> Result<Vec<f64>, InsertableIndexError> {
    match value.typename() {
        "FHIRInteger" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_integer
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRDecimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_decimal
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRPositiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_positive_int
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUnsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_unsigned_int
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_uri(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRUrl" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUrl>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUuid" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUuid>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRCanonical" => {
            let fp_uri = value
                .as_any()
                .downcast_ref::<FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUri" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUri>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_token(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    match value.typename() {
        "Coding" => {
            let fp_coding = value.as_any().downcast_ref::<Coding>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: fp_coding.system.as_ref().and_then(|s| s.value.clone()),
                code: fp_coding.code.as_ref().and_then(|v| v.value.clone()),
            }])
        }
        "CodeableConcept" => {
            let fp_codeable_concept = value
                .as_any()
                .downcast_ref::<CodeableConcept>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_codeable_concept
                .coding
                .as_ref()
                .and_then(|coding| {
                    Some(
                        coding
                            .iter()
                            .map(|c| TokenIndex {
                                system: c.system.as_ref().and_then(|s| s.value.clone()),
                                code: c.code.as_ref().and_then(|v| v.value.clone()),
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .unwrap_or_else(|| vec![]))
        }
        "Identifier" => {
            let fp_identifier = value.as_any().downcast_ref::<Identifier>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: fp_identifier.system.as_ref().and_then(|s| s.value.clone()),
                code: fp_identifier.value.as_ref().and_then(|v| v.value.clone()),
            }])
        }
        "ContactPoint" => {
            let fp_contact_point =
                value
                    .as_any()
                    .downcast_ref::<ContactPoint>()
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.typename().to_string())
                    })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_contact_point
                    .value
                    .as_ref()
                    .and_then(|v| v.value.clone()),
            }])
        }
        "FHIRCode" => {
            let fp_code = value
                .get_field("value")
                .and_then(|v| v.as_any().downcast_ref::<String>());

            Ok(vec![TokenIndex {
                system: None,
                code: fp_code.map(|v| v.to_string()),
            }])
        }
        "FHIRBoolean" => {
            let fp_boolean = value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(vec![TokenIndex {
                system: Some("http://hl7.org/fhir/special-values".to_string()),
                code: fp_boolean.value.as_ref().map(|v| v.to_string()),
            }])
        }
        "http://hl7.org/fhirpath/System.String" => {
            let string = value.as_any().downcast_ref::<String>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: Some(string.clone()),
            }])
        }
        "FHIRString" => {
            let fp_string = value.as_any().downcast_ref::<FHIRString>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_string.value.as_ref().map(|v| v.to_string()),
            }])
        }
        "FHIRId" => {
            let fp_id = value.as_any().downcast_ref::<FHIRId>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_id.value.as_ref().map(|v| v.to_string()),
            }])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.typename().to_string(),
        )),
    }
}

fn get_decimal_precision(value: f64) -> u32 {
    let value = value.to_string();
    let decimal_characters = value.split('.').nth(1);
    let mut digits = 0;
    if let Some(decimal_part) = decimal_characters {
        decimal_part.chars().for_each(|_| digits += 1);
    }

    digits
}

pub struct DecimalRange {
    pub start: f64,
    pub end: f64,
}

// Number and quantity dependent on the precision for indexing.
pub fn get_decimal_range(value: f64) -> DecimalRange {
    let decimal_precision = get_decimal_precision(value);
    return DecimalRange {
        start: value - 0.5 * 10f64.powi(-(decimal_precision as i32)),
        end: value + 0.5 * 10f64.powi(-(decimal_precision as i32)),
    };
}

fn fhirdecimal_to_quantity_range(value: &Option<Box<FHIRDecimal>>) -> Option<DecimalRange> {
    let decimal_range = value
        .as_ref()
        .and_then(|v| v.value.as_ref().map(|v| get_decimal_range(*v)));

    decimal_range
}

fn index_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    match value.typename() {
        "Range" => {
            let fp_range = value.as_any().downcast_ref::<Range>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            if fp_range.low.is_some() || fp_range.high.is_some() {
                let start_value = fp_range
                    .low
                    .as_ref()
                    .and_then(|v| v.value.as_ref().and_then(|v| v.value));
                let start_system = fp_range
                    .low
                    .as_ref()
                    .and_then(|v| v.system.as_ref().and_then(|s| s.value.clone()));
                let start_code = fp_range
                    .low
                    .as_ref()
                    .and_then(|v| v.code.as_ref().and_then(|c| c.value.clone()));

                let end_value = fp_range
                    .high
                    .as_ref()
                    .and_then(|v| v.value.as_ref().and_then(|v| v.value));
                let end_system = fp_range
                    .high
                    .as_ref()
                    .and_then(|v| v.system.as_ref().and_then(|s| s.value.clone()));
                let end_code = fp_range
                    .high
                    .as_ref()
                    .and_then(|v| v.code.as_ref().and_then(|c| c.value.clone()));

                return Ok(vec![QuantityRange {
                    start_system: start_system,
                    start_code: start_code,
                    start_value: start_value
                        .map_or(RangeValue::Infinity, |v| RangeValue::Number(v)),
                    end_system: end_system,
                    end_code: end_code,
                    end_value: end_value.map_or(RangeValue::Infinity, |v| RangeValue::Number(v)),
                }]);
            }
            return Ok(vec![]);
        }
        "Age" => {
            let fp_age = value.as_any().downcast_ref::<Age>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(decimal_range) = fhirdecimal_to_quantity_range(&fp_age.value) {
                return Ok(vec![QuantityRange {
                    start_system: fp_age.system.as_ref().and_then(|s| s.value.clone()),
                    start_code: fp_age.code.as_ref().and_then(|c| c.value.clone()),
                    start_value: RangeValue::Number(decimal_range.start),
                    end_system: fp_age.system.as_ref().and_then(|s| s.value.clone()),
                    end_code: fp_age.code.as_ref().and_then(|c| c.value.clone()),
                    end_value: RangeValue::Number(decimal_range.end),
                }]);
            } else {
                return Ok(vec![]);
            }
        }
        "Money" => {
            let fp_money = value.as_any().downcast_ref::<Money>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(decimal_range) = fhirdecimal_to_quantity_range(&fp_money.value) {
                return Ok(vec![QuantityRange {
                    start_system: Some("urn:iso:std:iso:4217".to_string()),
                    start_code: fp_money.currency.as_ref().and_then(|c| c.value.clone()),
                    start_value: RangeValue::Number(decimal_range.start),
                    end_system: Some("urn:iso:std:iso:4217".to_string()),
                    end_code: fp_money.currency.as_ref().and_then(|c| c.value.clone()),
                    end_value: RangeValue::Number(decimal_range.end),
                }]);
            } else {
                return Ok(vec![]);
            }
        }
        "Duration" => {
            let fp_duration = value.as_any().downcast_ref::<Duration>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(decimal_range) = fhirdecimal_to_quantity_range(&fp_duration.value) {
                return Ok(vec![QuantityRange {
                    start_system: fp_duration.system.as_ref().and_then(|s| s.value.clone()),
                    start_code: fp_duration.code.as_ref().and_then(|c| c.value.clone()),
                    start_value: RangeValue::Number(decimal_range.start),
                    end_system: fp_duration.system.as_ref().and_then(|s| s.value.clone()),
                    end_code: fp_duration.code.as_ref().and_then(|c| c.value.clone()),
                    end_value: RangeValue::Number(decimal_range.end),
                }]);
            } else {
                return Ok(vec![]);
            }
        }
        "Quantity" => {
            let fp_quantity = value.as_any().downcast_ref::<Quantity>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(decimal_range) = fhirdecimal_to_quantity_range(&fp_quantity.value) {
                return Ok(vec![QuantityRange {
                    start_system: fp_quantity.system.as_ref().and_then(|s| s.value.clone()),
                    start_code: fp_quantity.code.as_ref().and_then(|c| c.value.clone()),
                    start_value: RangeValue::Number(decimal_range.start),
                    end_system: fp_quantity.system.as_ref().and_then(|s| s.value.clone()),
                    end_code: fp_quantity.code.as_ref().and_then(|c| c.value.clone()),
                    end_value: RangeValue::Number(decimal_range.end),
                }]);
            } else {
                return Ok(vec![]);
            }
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.typename().to_string(),
        )),
    }
}

fn year_to_daterange(year: u16) -> Result<DateRange, InsertableIndexError> {
    let start_date = chrono::NaiveDate::from_ymd_opt(year as i32, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date = chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1)
        .and_then(|d| d.pred_opt())
        .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

fn year_month_to_daterange(year: u16, month: u8) -> Result<DateRange, InsertableIndexError> {
    let start_date = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date = if month < 12 {
        chrono::NaiveDate::from_ymd_opt(year as i32, (month + 1).into(), 1)
            .and_then(|d| d.pred_opt())
            .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
    } else {
        chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1)
            .and_then(|d| d.pred_opt())
            .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
    }
    .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

fn year_month_day_to_daterange(
    year: u16,
    month: u8,
    day: u8,
) -> Result<DateRange, InsertableIndexError> {
    let start_date = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

pub fn date_time_range(value: &DateTime) -> Result<DateRange, InsertableIndexError> {
    match value {
        DateTime::Year(year) => Ok(year_to_daterange(*year)?),
        DateTime::YearMonth(year, month) => Ok(year_month_to_daterange(*year, *month)?),
        DateTime::YearMonthDay(year, month, day) => {
            Ok(year_month_day_to_daterange(*year, *month, *day)?)
        }
        DateTime::Iso8601(date_time) => {
            return Ok(DateRange {
                start: date_time.timestamp_millis(),
                end: date_time.timestamp_millis(),
            });
        }
    }
}

fn index_date(value: &dyn MetaValue) -> Result<Vec<DateRange>, InsertableIndexError> {
    match value.typename() {
        "Timing" => {
            let fp_timing = value.as_any().downcast_ref::<Timing>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(events) = fp_timing.event.as_ref() {
                let date_ranges = events
                    .iter()
                    .map(|event| index_date(event.as_ref()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(date_ranges.into_iter().flatten().collect())
            } else {
                Ok(vec![])
            }
        }
        "FHIRDate" => {
            let fp_date = value
                .as_any()
                .downcast_ref::<FHIRDate>()
                .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string()))?
                .value
                .as_ref();

            match &fp_date {
                Some(Date::Year(year)) => Ok(vec![year_to_daterange(*year)?]),
                Some(Date::YearMonth(year, month)) => {
                    Ok(vec![year_month_to_daterange(*year, *month)?])
                }
                Some(Date::YearMonthDay(year, month, day)) => {
                    Ok(vec![year_month_day_to_daterange(*year, *month, *day)?])
                }
                None => Ok(vec![]),
            }
        }
        "FHIRDateTime" => {
            let fp_datetime = value
                .as_any()
                .downcast_ref::<FHIRDateTime>()
                .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string()))?
                .value
                .as_ref();

            match &fp_datetime {
                Some(date_time) => date_time_range(date_time).map(|date_range| vec![date_range]),
                None => {
                    return Ok(vec![]);
                }
            }
        }
        "FHIRInstant" => {
            let fp_instant = value
                .as_any()
                .downcast_ref::<FHIRInstant>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            match &fp_instant.value {
                Some(Instant::Iso8601(instant)) => {
                    let timestamp = instant.timestamp_millis();
                    return Ok(vec![DateRange {
                        start: timestamp,
                        end: timestamp,
                    }]);
                }
                None => {
                    return Ok(vec![]);
                }
            }
        }
        "Period" => {
            let fp_period = value.as_any().downcast_ref::<Period>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            let fp_start = if let Some(date) = fp_period.start.as_ref() {
                let date = date.as_ref();
                let date_range = index_date(date)?;
                date_range
                    .get(0)
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.typename().to_string())
                    })?
                    .start
            } else {
                0
            };

            let fp_end = if let Some(date) = fp_period.end.as_ref() {
                let date = date.as_ref();
                let date_range = index_date(date)?;
                date_range
                    .get(0)
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.typename().to_string())
                    })?
                    .end
            } else {
                i64::MAX
            };

            Ok(vec![DateRange {
                start: fp_start,
                end: fp_end,
            }])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.typename().to_string(),
        )),
    }
}

fn index_reference(value: &dyn MetaValue) -> Result<Vec<ReferenceIndex>, InsertableIndexError> {
    match value.typename() {
        "Reference" => {
            let fp_reference = value.as_any().downcast_ref::<Reference>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            if let Some(reference) = &fp_reference
                .reference
                .as_ref()
                .and_then(|r| r.value.as_ref())
            {
                let parts: Vec<&str> = reference.split('/').collect();
                if parts.len() == 2 {
                    let resource_type = ResourceType::try_from(parts[0])?;
                    let id = parts[1].to_string();
                    return Ok(vec![ReferenceIndex {
                        resource_type: Some(resource_type.as_ref().to_string()),
                        id: Some(id),
                        uri: None,
                    }]);
                }
            }

            Ok(vec![])
        }
        "FHIRCanonical" => {
            let fp_canonical = value
                .as_any()
                .downcast_ref::<FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            if let Some(canonical) = &fp_canonical.value {
                return Ok(vec![ReferenceIndex {
                    id: None,
                    resource_type: None,
                    uri: Some(canonical.to_string()),
                }]);
            }
            Ok(vec![])
        }
        "FHIRUri" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUri>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            if let Some(uri) = &fp_uri.value {
                return Ok(vec![ReferenceIndex {
                    id: None,
                    resource_type: None,
                    uri: Some(uri.to_string()),
                }]);
            }
            Ok(vec![])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.typename().to_string(),
        )),
    }
}

pub fn to_insertable_index(
    parameter: &SearchParameter,
    result: Vec<&dyn MetaValue>,
) -> Result<InsertableIndex, OperationOutcomeError> {
    match parameter.type_.as_ref() {
        SearchParamType::Number(_) => {
            let numbers = result
                .iter()
                .filter_map(|v| index_number(*v).ok())
                .flatten()
                .collect::<Vec<_>>();
            Ok(InsertableIndex::Number(numbers))
        }
        SearchParamType::String(_) => {
            let strings = result
                .iter()
                .filter_map(|v| index_string(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::String(strings))
        }
        SearchParamType::Uri(_) => {
            let uris = result
                .iter()
                .filter_map(|v| index_uri(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::URI(uris))
        }
        SearchParamType::Token(_) => {
            let tokens = result
                .iter()
                .filter_map(|v| index_token(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Token(tokens))
        }
        SearchParamType::Date(_) => {
            let dates = result
                .iter()
                .filter_map(|v| index_date(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Date(dates))
        }
        SearchParamType::Reference(_) => {
            let references = result
                .iter()
                .filter_map(|v: &&dyn MetaValue| index_reference(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Reference(references))
        }
        SearchParamType::Quantity(_) => {
            let quantities = result
                .iter()
                .filter_map(|v| index_quantity(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Quantity(quantities))
        }
        // Not Supported yet
        SearchParamType::Composite(_) => Ok(InsertableIndex::Composite(vec![])),
        SearchParamType::Special(_) => Ok(InsertableIndex::Special(vec![])),
        _ => {
            let type_name: Option<String> = parameter.type_.as_ref().into();
            Err(
                InsertableIndexError::InvalidType(type_name.unwrap_or("unknown".to_string()))
                    .into(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidized_fhir_model::r4::generated::types::{
        FHIRDate, FHIRDateTime, Period, Reference, Timing,
    };

    #[test]
    fn test_year_month_to_daterange() {
        let year = 2023;
        let month: u8 = 5;
        let date_range = year_month_to_daterange(year, month).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_year_month_day_to_daterange() {
        let year = 2023;
        let month: u8 = 5;
        let day = 15;
        let date_range = year_month_day_to_daterange(year, month, day).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_year_to_daterange() {
        let year = 2023;
        let date_range = year_to_daterange(year).unwrap();
        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_index_date() {
        let date_value = FHIRDate {
            id: None,
            extension: None,
            value: Some(Date::Year(2023)),
        };
        let result = index_date(&date_value).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            result[0].end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_instant() {
        let fhir_date = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Iso8601(
                chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            )),
        };

        let range = index_date(&fhir_date).unwrap();
        let date_range = range.get(0).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                .unwrap()
                .with_timezone(&chrono::Utc)
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                .unwrap()
                .with_timezone(&chrono::Utc)
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_period() {
        let start = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Year(2023)),
        };

        let end = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::YearMonthDay(2023, 5, 15)),
        };

        let period = Period {
            id: None,
            extension: None,
            start: Some(Box::new(start)),
            end: Some(Box::new(end)),
        };

        let range = index_date(&period).unwrap();
        let date_range = range.get(0).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_missing() {
        let start = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Year(2023)),
        };

        let end = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::YearMonthDay(2023, 5, 15)),
        };

        let period = Period {
            id: None,
            extension: None,
            start: None,
            end: Some(Box::new(end)),
        };

        let range = index_date(&period).unwrap();
        let date_range = range.get(0).unwrap();

        assert_eq!(date_range.start, 0);
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );

        let period = Period {
            id: None,
            extension: None,
            start: Some(Box::new(start)),
            end: None,
        };

        let range = index_date(&period).unwrap();
        let date_range = range.get(0).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(date_range.end, i64::MAX);
    }

    #[test]
    fn test_date_range_end() {
        let year = 2023;
        let month: u8 = 12;
        let day = 31;
        let date_range = year_month_day_to_daterange(year, month, day).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );

        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );

        let date_range = year_month_to_daterange(year, month).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-12-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );

        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_timing() {
        let mut timing = Timing::default();
        timing.event = Some(vec![
            Box::new(FHIRDateTime {
                id: None,
                extension: None,
                value: Some(DateTime::YearMonthDay(2023, 12, 31)),
            }),
            Box::new(FHIRDateTime {
                id: None,
                extension: None,
                value: Some(DateTime::YearMonthDay(2024, 1, 1)),
            }),
        ]);

        let date_ranges = index_date(&timing).unwrap();
        assert_eq!(date_ranges.len(), 2);

        assert_eq!(
            date_ranges[0],
            DateRange {
                start: chrono::DateTime::parse_from_rfc3339("2023-12-31T00:00:00Z")
                    .unwrap()
                    .timestamp_millis(),
                end: chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                    .unwrap()
                    .timestamp_millis(),
            }
        );

        assert_eq!(
            date_ranges[1],
            DateRange {
                start: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                    .unwrap()
                    .timestamp_millis(),
                end: chrono::DateTime::parse_from_rfc3339("2024-01-01T23:59:59.999Z")
                    .unwrap()
                    .timestamp_millis(),
            }
        );
    }

    #[test]
    fn test_indexing_reference() {
        let reference = Reference {
            type_: None,
            identifier_: None,
            display: None,
            id: None,
            extension: None,
            reference: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("Patient/123".to_string()),
            })),
        };

        let result = index_reference(&reference).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resource_type, Some("Patient".to_string()));
        assert_eq!(result[0].id, Some("123".to_string()));
        assert!(result[0].uri.is_none());
    }

    #[test]
    fn test_indexing_invalid_reference() {
        let reference = Reference {
            type_: None,
            identifier_: None,
            display: None,
            id: None,
            extension: None,
            reference: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("BadType/123".to_string()),
            })),
        };

        let result = index_reference(&reference);

        assert!(result.is_err())
    }

    #[test]
    fn test_canonical_index() {
        let canonical = FHIRCanonical {
            id: None,
            extension: None,
            value: Some("http://example.com/CanonicalResource".to_string()),
        };
        let result = index_reference(&canonical).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].id.is_none());
        assert!(result[0].resource_type.is_none());
        assert_eq!(
            result[0].uri,
            Some("http://example.com/CanonicalResource".to_string())
        );
    }

    #[test]
    fn test_uri_indexing() {
        let uri = FHIRUri {
            id: None,
            extension: None,
            value: Some("http://example.com/URIResource".to_string()),
        };
        let result = index_reference(&uri).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].id.is_none());
        assert!(result[0].resource_type.is_none());
        assert_eq!(
            result[0].uri,
            Some("http://example.com/URIResource".to_string())
        );
    }
}
