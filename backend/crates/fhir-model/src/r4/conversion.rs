use crate::r4::{
    datetime::{Date, DateTime, Instant, Time},
    generated::types::{FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRUnsignedInt},
};
use haste_reflect::MetaValue;
use std::{collections::HashSet, sync::LazyLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DowncastError {
    #[error("Failed to downcast value to type '{0}'")]
    FailedDowncast(String),
}

/// Number types to use in FHIR evaluation
pub static NUMBER_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("integer");
    m.insert("decimal");
    m.insert("positiveInt");
    m.insert("unsignedInt");
    m.insert("http://hl7.org/fhirpath/System.Decimal");
    m.insert("http://hl7.org/fhirpath/System.Integer");
    m
});

pub static BOOLEAN_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("boolean");
    m.insert("http://hl7.org/fhirpath/System.Boolean");
    m
});

pub static DATE_TIME_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("date");
    m.insert("dateTime");
    m.insert("instant");
    m.insert("time");
    m.insert("http://hl7.org/fhirpath/System.DateTime");
    m.insert("http://hl7.org/fhirpath/System.Instant");
    m.insert("http://hl7.org/fhirpath/System.Date");
    m.insert("http://hl7.org/fhirpath/System.Time");
    m
});

pub static STRING_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("base64Binary");
    m.insert("canonical");

    m.insert("id");
    m.insert("code");
    m.insert("string");
    m.insert("oid");
    m.insert("uri");
    m.insert("url");
    m.insert("uuid");
    m.insert("xhtml");

    m.insert("http://hl7.org/fhirpath/System.String");
    m
});

pub static PRIMITIVE_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut res = BOOLEAN_TYPES.clone();
    res.extend(NUMBER_TYPES.iter().map(|s| *s));
    res.extend(DATE_TIME_TYPES.iter().map(|s| *s));
    res.extend(STRING_TYPES.iter().map(|s| *s));

    res
});

pub fn downcast_bool(value: &dyn MetaValue) -> Result<bool, DowncastError> {
    match value.fhir_type() {
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .map(|v| *v)
            .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string())),
        "boolean" => {
            let fp_bool = value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;
            downcast_bool(fp_bool.value.as_ref().unwrap_or(&false))
        }
        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}

pub fn downcast_string(value: &dyn MetaValue) -> Result<String, DowncastError> {
    match value.fhir_type() {
        "canonical" | "base64Binary" | "code" | "string" | "oid" | "uri" | "url" | "uuid"
        | "id" | "xhtml" => downcast_string(value.get_field("value").unwrap_or(&"".to_string())),

        "http://hl7.org/fhirpath/System.String" => value
            .as_any()
            .downcast_ref::<String>()
            .map(|v| v.clone())
            .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string())),

        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}

pub fn downcast_number(value: &dyn MetaValue) -> Result<f64, DowncastError> {
    match value.fhir_type() {
        "integer" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;
            downcast_number(fp_integer.value.as_ref().unwrap_or(&0))
        }
        "decimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;
            downcast_number(fp_decimal.value.as_ref().unwrap_or(&0.0))
        }
        "positiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;

            downcast_number(fp_positive_int.value.as_ref().unwrap_or(&0))
        }
        "unsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;

            downcast_number(fp_unsigned_int.value.as_ref().unwrap_or(&0))
        }
        "http://hl7.org/fhirpath/System.Integer" => value
            .as_any()
            .downcast_ref::<i64>()
            .map(|v| *v as f64)
            .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string())),

        "http://hl7.org/fhirpath/System.Decimal" => value
            .as_any()
            .downcast_ref::<f64>()
            .map(|v| *v)
            .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string())),
        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}

pub fn downcast_datetime(value: &dyn MetaValue) -> Result<String, DowncastError> {
    // For simplicity, we will just downcast to string for date and datetime types, as FHIRPath evaluation only requires string representation of dates.
    match value.fhir_type() {
        "date" | "dateTime" | "instant" | "time" => {
            downcast_datetime(value.get_field("value").unwrap_or(&"".to_string()))
        }
        "http://hl7.org/fhirpath/System.Date" => {
            let fp_date = value
                .as_any()
                .downcast_ref::<Date>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;

            Ok(fp_date.to_string())
        }
        "http://hl7.org/fhirpath/System.DateTime" => {
            let fp_datetime = value
                .as_any()
                .downcast_ref::<DateTime>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;

            Ok(fp_datetime.to_string())
        }
        "http://hl7.org/fhirpath/System.Instant" => {
            let fp_instant = value
                .as_any()
                .downcast_ref::<Instant>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;

            Ok(fp_instant.to_string())
        }
        "http://hl7.org/fhirpath/System.Time" => {
            let fp_time = value
                .as_any()
                .downcast_ref::<Time>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.fhir_type().to_string()))?;
            Ok(fp_time.to_string())
        }
        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}
